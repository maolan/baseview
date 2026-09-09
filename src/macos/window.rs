use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::ffi::c_void;
use std::rc::Rc;

use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSBackingStoreBuffered,
    NSPasteboard, NSView, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{BOOL, NO, YES, id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use core_foundation::runloop::{
    __CFRunLoopTimer, CFRunLoop, CFRunLoopTimer, CFRunLoopTimerContext, kCFRunLoopDefaultMode,
};
use keyboard_types::KeyboardEvent;
use objc::class;
use objc::{msg_send, runtime::Object, sel, sel_impl};
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, DisplayHandle as RwhDisplayHandle, HandleError,
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle as RwhWindowHandle,
};

use crate::{
    Event, EventStatus, MouseCursor, Size, WindowHandler, WindowInfo, WindowOpenOptions,
    WindowScalePolicy,
};

use super::keyboard::KeyboardState;
use super::view::{BASEVIEW_STATE_IVAR, create_view};

#[cfg(feature = "opengl")]
use crate::gl::{GlConfig, GlContext};

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRunLoopGetMain() -> *mut c_void;
    fn CFRunLoopWakeUp(rl: *mut c_void);
    fn CFRunLoopSourceCreate(
        allocator: *mut c_void, order: i64, context: *const CFRunLoopSourceContext,
    ) -> *mut c_void;
    fn CFRunLoopSourceSignal(source: *mut c_void);
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *mut c_void);
}

#[repr(C)]
struct CFRunLoopSourceContext {
    version: i64,
    info: *mut c_void,
    retain: Option<extern "C" fn(*const c_void) -> *const c_void>,
    release: Option<extern "C" fn(*const c_void)>,
    copy_description: Option<extern "C" fn(*const c_void) -> *mut c_void>,
    equal: Option<extern "C" fn(*const c_void, *const c_void) -> bool>,
    hash: Option<extern "C" fn(*const c_void) -> usize>,
    schedule: Option<extern "C" fn(*mut c_void, *mut c_void, *mut c_void)>,
    cancel: Option<extern "C" fn(*mut c_void, *mut c_void, *mut c_void)>,
    perform: Option<extern "C" fn(*mut c_void)>,
}

type ErasedJob = Box<dyn FnOnce() -> SendBox + Send>;

/// Result of a main-thread job, transferred to the waiting thread through
/// a mutex/condvar handshake that guarantees an exclusive handoff (written
/// once on the main thread, read once by the waiter after being woken), so
/// wrapping it as `Send` is sound even if the concrete type is not `Send`.
struct SendBox(Box<dyn std::any::Any>);
unsafe impl Send for SendBox {}

struct MainJob {
    job: std::sync::Mutex<Option<ErasedJob>>,
    outcome: std::sync::Mutex<Option<std::thread::Result<SendBox>>>,
    done: std::sync::Condvar,
}

/// Opaque `CFRunLoopSourceRef`. The handle is created once, retained by the
/// main run loop it is added to, and only ever passed to CoreFoundation
/// calls, which are thread-safe.
struct SourcePtr(*mut c_void);
unsafe impl Send for SourcePtr {}

struct MainQueue {
    pending: std::sync::Mutex<std::collections::VecDeque<std::sync::Arc<MainJob>>>,
    source: std::sync::Mutex<SourcePtr>,
}

static MAIN_QUEUE: std::sync::OnceLock<MainQueue> = std::sync::OnceLock::new();

/// Runs all jobs queued by worker threads. Executed on the main run loop.
extern "C" fn main_queue_perform(_info: *mut c_void) {
    let Some(queue) = MAIN_QUEUE.get() else { return };
    loop {
        let next = queue.pending.lock().expect("main queue lock").pop_front();
        let Some(job) = next else { break };
        let outcome = {
            let mut job_guard = job.job.lock().expect("main job lock");
            let f = job_guard.take().expect("main job already taken");
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        };
        *job.outcome.lock().expect("main outcome lock") = Some(outcome);
        job.done.notify_all();
    }
}

/// Runs `f` on the AppKit main thread, blocking the caller until it
/// completes. AppKit requires window and application objects to be used
/// from the process main thread; callers (e.g. audio plugin UIs) invoke
/// this from their own worker threads.
///
/// Requires the process main thread to run a CoreFoundation run loop
/// (e.g. `CFRunLoopRunInMode` or `NSApplicationMain`) in the default
/// mode, otherwise this deadlocks.
///
/// `pub` (hidden) so that cross-crate monomorphizations of the generic
/// window constructors can link against it.
#[doc(hidden)]
pub fn run_on_main<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: 'static,
{
    unsafe {
        let is_main: BOOL = msg_send![class!(NSThread), isMainThread];
        if is_main == YES {
            return f();
        }
    }

    let job = enqueue_main_job(Box::new(move || SendBox(Box::new(f()))));

    let mut outcome = job.outcome.lock().expect("main outcome lock");
    while outcome.is_none() {
        outcome = job.done.wait(outcome).expect("main condvar wait");
    }
    match outcome.take().expect("main job outcome") {
        Ok(value) => *value.0.downcast().expect("main job result type mismatch"),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// Queues `f` to run on the AppKit main thread without blocking the caller.
/// Used for deferred teardown that must not run while an event handler is
/// borrowing the window state.
#[doc(hidden)]
pub fn run_on_main_async<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    let _job = enqueue_main_job(Box::new(move || {
        f();
        SendBox(Box::new(()))
    }));
}

fn enqueue_main_job(f: ErasedJob) -> std::sync::Arc<MainJob> {
    let queue = MAIN_QUEUE.get_or_init(|| {
        let queue = MainQueue {
            pending: std::sync::Mutex::new(std::collections::VecDeque::new()),
            source: std::sync::Mutex::new(SourcePtr(std::ptr::null_mut())),
        };
        let context = CFRunLoopSourceContext {
            version: 0,
            info: std::ptr::null_mut(),
            retain: None,
            release: None,
            copy_description: None,
            equal: None,
            hash: None,
            schedule: None,
            cancel: None,
            perform: Some(main_queue_perform),
        };
        unsafe {
            let source = CFRunLoopSourceCreate(std::ptr::null_mut(), 0, &context);
            assert!(!source.is_null(), "failed to create main run loop source");
            CFRunLoopAddSource(CFRunLoopGetMain(), source, kCFRunLoopDefaultMode as *mut c_void);
            *queue.source.lock().expect("main source lock") = SourcePtr(source);
        }
        queue
    });

    let job = std::sync::Arc::new(MainJob {
        job: std::sync::Mutex::new(Some(f)),
        outcome: std::sync::Mutex::new(None),
        done: std::sync::Condvar::new(),
    });
    queue.pending.lock().expect("main queue lock").push_back(std::sync::Arc::clone(&job));
    unsafe {
        CFRunLoopSourceSignal(queue.source.lock().expect("main source lock").0);
        CFRunLoopWakeUp(CFRunLoopGetMain());
    }
    job
}

pub struct WindowHandle {
    state: Rc<WindowState>,
}

impl WindowHandle {
    pub fn close(&mut self) {
        self.state.window_inner.close();
    }

    pub fn is_open(&self) -> bool {
        self.state.window_inner.open.get()
    }
}

impl HasWindowHandle for WindowHandle {
    fn window_handle(&self) -> Result<RwhWindowHandle<'_>, HandleError> {
        Ok(unsafe { RwhWindowHandle::borrow_raw(self.state.window_inner.raw_window_handle()) })
    }
}

pub(super) struct WindowInner {
    open: Cell<bool>,

    /// Only set if we created the parent window, i.e. we are running in
    /// parentless mode
    ns_app: Cell<Option<id>>,
    /// Only set if we created the parent window, i.e. we are running in
    /// parentless mode
    ns_window: Cell<Option<id>>,
    /// Our subclassed NSView
    ns_view: id,

    #[cfg(feature = "opengl")]
    gl_context: Option<GlContext>,
}

impl WindowInner {
    pub(super) fn close(&self) {
        if self.open.get() {
            self.open.set(false);
            if self.ns_app.get().is_some() {
                unsafe {
                    // In parentless mode, `app.run()` (which runs on the
                    // main thread) returns after this stop request, and
                    // `open_blocking` performs the AppKit teardown once no
                    // event handler borrows the window state anymore.
                    let app = self.ns_app.get().unwrap_or(nil);
                    app.stop_(app);
                }
            } else {
                // Parented mode: defer the teardown to the main run loop so
                // it runs after any in-flight event handler.
                let ns_view = self.ns_view as usize;
                run_on_main_async(move || unsafe {
                    let ns_view = ns_view as id;
                    let state_ptr: *const c_void = *(*ns_view).get_ivar(BASEVIEW_STATE_IVAR);
                    let window_state = Rc::from_raw(state_ptr as *mut WindowState);

                    if let Some(frame_timer) = window_state.frame_timer.take() {
                        CFRunLoop::get_current().remove_timer(&frame_timer, kCFRunLoopDefaultMode);
                    }

                    let notification_center: id =
                        msg_send![class!(NSNotificationCenter), defaultCenter];
                    let () = msg_send![notification_center, removeObserver: ns_view];

                    if let Some(ns_window) = window_state.window_inner.ns_window.take() {
                        ns_window.close();
                    }

                    ns_view.removeFromSuperview();
                    let () = msg_send![ns_view as id, release];

                    drop(window_state);
                });
            }
        }
    }

    fn raw_window_handle(&self) -> RawWindowHandle {
        let handle = AppKitWindowHandle::new(
            std::ptr::NonNull::new(self.ns_view as *mut c_void).expect("ns_view is null"),
        );

        RawWindowHandle::AppKit(handle)
    }
}

pub struct Window<'a> {
    inner: &'a WindowInner,
}

impl<'a> Window<'a> {
    pub fn open_parented<P, H, B>(parent: &P, options: WindowOpenOptions, build: B) -> WindowHandle
    where
        P: HasWindowHandle,
        H: WindowHandler + 'static,
        B: FnOnce(&mut crate::Window) -> H,
        B: Send + 'static,
    {
        // Resolve the parent handle on the calling thread; only the
        // window creation runs on the main thread.
        let parent_ns_view = match parent.window_handle().expect("parent window handle").as_raw() {
            RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr() as usize,
            _ => panic!("Not a macOS window"),
        };
        run_on_main(move || Self::open_parented_on_main::<H, B>(parent_ns_view, options, build))
    }

    fn open_parented_on_main<H, B>(
        parent_ns_view: usize, options: WindowOpenOptions, build: B,
    ) -> WindowHandle
    where
        H: WindowHandler + 'static,
        B: FnOnce(&mut crate::Window) -> H,
        B: Send + 'static,
    {
        let pool = unsafe { NSAutoreleasePool::new(nil) };

        let scaling = match options.scale {
            WindowScalePolicy::ScaleFactor(scale) => scale,
            WindowScalePolicy::SystemScaleFactor => 1.0,
        };

        let window_info = WindowInfo::from_logical_size(options.size, scaling);

        let ns_view = unsafe { create_view(&options) };

        let window_inner = WindowInner {
            open: Cell::new(true),
            ns_app: Cell::new(None),
            ns_window: Cell::new(None),
            ns_view,

            #[cfg(feature = "opengl")]
            gl_context: options
                .gl_config
                .map(|gl_config| Self::create_gl_context(None, ns_view, gl_config)),
        };

        let window_handle = Self::init(window_inner, window_info, build);

        unsafe {
            let _: id = msg_send![parent_ns_view as *mut Object, addSubview: ns_view];

            let () = msg_send![pool, drain];
        }

        window_handle
    }

    pub fn open_blocking<H, B>(options: WindowOpenOptions, build: B)
    where
        H: WindowHandler + 'static,
        B: FnOnce(&mut crate::Window) -> H,
        B: Send + 'static,
    {
        run_on_main(|| Self::open_blocking_on_main::<H, B>(options, build));
    }

    fn open_blocking_on_main<H, B>(options: WindowOpenOptions, build: B)
    where
        H: WindowHandler + 'static,
        B: FnOnce(&mut crate::Window) -> H,
        B: Send + 'static,
    {
        let pool = unsafe { NSAutoreleasePool::new(nil) };

        let app = unsafe { NSApp() };

        unsafe {
            app.setActivationPolicy_(NSApplicationActivationPolicyRegular);
        }

        let scaling = match options.scale {
            WindowScalePolicy::ScaleFactor(scale) => scale,
            WindowScalePolicy::SystemScaleFactor => 1.0,
        };

        let window_info = WindowInfo::from_logical_size(options.size, scaling);

        let rect = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(window_info.logical_size().width, window_info.logical_size().height),
        );

        let ns_window = unsafe {
            let ns_window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                rect,
                NSWindowStyleMask::NSTitledWindowMask
                    | NSWindowStyleMask::NSClosableWindowMask
                    | NSWindowStyleMask::NSMiniaturizableWindowMask,
                NSBackingStoreBuffered,
                NO,
            );
            ns_window.center();

            let title = NSString::alloc(nil).init_str(&options.title).autorelease();
            ns_window.setTitle_(title);

            ns_window.makeKeyAndOrderFront_(nil);

            ns_window
        };

        let ns_view = unsafe { create_view(&options) };

        let window_inner = WindowInner {
            open: Cell::new(true),
            ns_app: Cell::new(Some(app)),
            ns_window: Cell::new(Some(ns_window)),
            ns_view,

            #[cfg(feature = "opengl")]
            gl_context: options
                .gl_config
                .map(|gl_config| Self::create_gl_context(Some(ns_window), ns_view, gl_config)),
        };

        let _window_handle = Self::init(window_inner, window_info, build);

        unsafe {
            ns_window.setContentView_(ns_view);
            ns_window.setDelegate_(ns_view);

            let () = msg_send![pool, drain];

            app.run();

            // Teardown: the window has been closed and `app.run()` has
            // returned, so no event handler borrows the window state
            // anymore. The `ns_view` pointer is still valid: our initial
            // reference outlives the window's.
            let state_ptr: *const c_void = *(*ns_view).get_ivar(BASEVIEW_STATE_IVAR);
            let window_state = Rc::from_raw(state_ptr as *mut WindowState);

            if let Some(frame_timer) = window_state.frame_timer.take() {
                CFRunLoop::get_current().remove_timer(&frame_timer, kCFRunLoopDefaultMode);
            }

            let notification_center: id = msg_send![class!(NSNotificationCenter), defaultCenter];
            let () = msg_send![notification_center, removeObserver: ns_view];

            ns_window.close();
            let () = msg_send![ns_view as id, release];

            drop(window_state);
        }
    }

    fn init<H, B>(window_inner: WindowInner, window_info: WindowInfo, build: B) -> WindowHandle
    where
        H: WindowHandler + 'static,
        B: FnOnce(&mut crate::Window) -> H,
        B: Send + 'static,
    {
        let mut window = crate::Window::new(Window { inner: &window_inner });
        let window_handler = Box::new(build(&mut window));

        let ns_view = window_inner.ns_view;

        let window_state = Rc::new(WindowState {
            window_inner,
            window_handler: RefCell::new(window_handler),
            keyboard_state: KeyboardState::new(),
            frame_timer: Cell::new(None),
            window_info: Cell::new(window_info),
            deferred_events: RefCell::default(),
        });

        let window_state_ptr = Rc::into_raw(Rc::clone(&window_state));

        unsafe {
            (*ns_view).set_ivar(BASEVIEW_STATE_IVAR, window_state_ptr as *const c_void);

            WindowState::setup_timer(window_state_ptr);
        }

        WindowHandle { state: window_state }
    }

    pub fn close(&mut self) {
        self.inner.close();
    }

    pub fn has_focus(&mut self) -> bool {
        unsafe {
            let view = self.inner.ns_view.as_mut().unwrap();
            let window: id = msg_send![view, window];
            if window == nil {
                return false;
            };
            let first_responder: id = msg_send![window, firstResponder];
            let is_key_window: BOOL = msg_send![window, isKeyWindow];
            let is_focused: BOOL = msg_send![view, isEqual: first_responder];
            is_key_window == YES && is_focused == YES
        }
    }

    pub fn focus(&mut self) {
        unsafe {
            let view = self.inner.ns_view.as_mut().unwrap();
            let window: id = msg_send![view, window];
            if window != nil {
                msg_send![window, makeFirstResponder:view]
            }
        }
    }

    pub fn resize(&mut self, size: Size) {
        if self.inner.open.get() {
            let size = NSSize::new(size.width.round(), size.height.round());

            unsafe { NSView::setFrameSize(self.inner.ns_view, size) };
            unsafe {
                let _: () = msg_send![self.inner.ns_view, setNeedsDisplay: YES];
            }

            #[cfg(feature = "opengl")]
            if let Some(gl_context) = &self.inner.gl_context {
                gl_context.resize(size);
            }

            if let Some(ns_window) = self.inner.ns_window.get() {
                unsafe { NSWindow::setContentSize_(ns_window, size) };
            }
        }
    }

    pub fn set_mouse_cursor(&mut self, _mouse_cursor: MouseCursor) {
        todo!()
    }

    #[cfg(feature = "opengl")]
    pub fn gl_context(&self) -> Option<&GlContext> {
        self.inner.gl_context.as_ref()
    }

    #[cfg(feature = "opengl")]
    fn create_gl_context(ns_window: Option<id>, ns_view: id, config: GlConfig) -> GlContext {
        let handle = AppKitWindowHandle::new(
            std::ptr::NonNull::new(ns_view as *mut c_void).expect("ns_view is null"),
        );
        let handle = RawWindowHandle::AppKit(handle);

        unsafe { GlContext::create(&handle, config).expect("Could not create OpenGL context") }
    }
}

pub(super) struct WindowState {
    pub(super) window_inner: WindowInner,
    window_handler: RefCell<Box<dyn WindowHandler>>,
    keyboard_state: KeyboardState,
    frame_timer: Cell<Option<CFRunLoopTimer>>,
    /// The last known window info for this window.
    pub window_info: Cell<WindowInfo>,

    /// Events that will be triggered at the end of `window_handler`'s borrow.
    deferred_events: RefCell<VecDeque<Event>>,
}

impl WindowState {
    /// Gets the `WindowState` held by a given `NSView`.
    ///
    /// This method returns a cloned `Rc<WindowState>` rather than just a `&WindowState`, since the
    /// original `Rc<WindowState>` owned by the `NSView` can be dropped at any time
    /// (including during an event handler).
    pub(super) unsafe fn from_view(view: &Object) -> Rc<WindowState> {
        let state_ptr: *const c_void = unsafe { *view.get_ivar(BASEVIEW_STATE_IVAR) };

        let state_rc = unsafe { Rc::from_raw(state_ptr as *const WindowState) };
        let state = Rc::clone(&state_rc);
        let _ = Rc::into_raw(state_rc);

        state
    }

    /// Trigger the event immediately and return the event status.
    /// Will panic if `window_handler` is already borrowed (see `trigger_deferrable_event`).
    pub(super) fn trigger_event(&self, event: Event) -> EventStatus {
        let mut window = crate::Window::new(Window { inner: &self.window_inner });
        let mut window_handler = self.window_handler.borrow_mut();
        let status = window_handler.on_event(&mut window, event);
        self.send_deferred_events(window_handler.as_mut());
        status
    }

    /// Trigger the event immediately if `window_handler` can be borrowed mutably,
    /// otherwise add the event to a queue that will be cleared once `window_handler`'s mutable borrow ends.
    /// As this method might result in the event triggering asynchronously, it can't reliably return the event status.
    pub(super) fn trigger_deferrable_event(&self, event: Event) {
        if let Ok(mut window_handler) = self.window_handler.try_borrow_mut() {
            let mut window = crate::Window::new(Window { inner: &self.window_inner });
            window_handler.on_event(&mut window, event);
            self.send_deferred_events(window_handler.as_mut());
        } else {
            self.deferred_events.borrow_mut().push_back(event);
        }
    }

    pub(super) fn trigger_frame(&self) {
        let mut window = crate::Window::new(Window { inner: &self.window_inner });
        let mut window_handler = self.window_handler.borrow_mut();
        window_handler.on_frame(&mut window);
        self.send_deferred_events(window_handler.as_mut());
    }

    pub(super) fn keyboard_state(&self) -> &KeyboardState {
        &self.keyboard_state
    }

    pub(super) fn process_native_key_event(&self, event: *mut Object) -> Option<KeyboardEvent> {
        self.keyboard_state.process_native_event(event)
    }

    unsafe fn setup_timer(window_state_ptr: *const WindowState) {
        extern "C" fn timer_callback(_: *mut __CFRunLoopTimer, window_state_ptr: *mut c_void) {
            unsafe {
                let window_state = &*(window_state_ptr as *const WindowState);

                window_state.trigger_frame();
            }
        }

        let mut timer_context = CFRunLoopTimerContext {
            version: 0,
            info: window_state_ptr as *mut c_void,
            retain: None,
            release: None,
            copyDescription: None,
        };

        let timer = CFRunLoopTimer::new(0.0, 0.015, 0, 0, timer_callback, &mut timer_context);

        unsafe { CFRunLoop::get_current().add_timer(&timer, kCFRunLoopDefaultMode) };

        unsafe { (*window_state_ptr).frame_timer.set(Some(timer)) };
    }

    fn send_deferred_events(&self, window_handler: &mut dyn WindowHandler) {
        let mut window = crate::Window::new(Window { inner: &self.window_inner });
        loop {
            let next_event = self.deferred_events.borrow_mut().pop_front();
            if let Some(event) = next_event {
                window_handler.on_event(&mut window, event);
            } else {
                break;
            }
        }
    }
}

impl<'a> HasWindowHandle for Window<'a> {
    fn window_handle(&self) -> Result<RwhWindowHandle<'_>, HandleError> {
        Ok(unsafe { RwhWindowHandle::borrow_raw(self.inner.raw_window_handle()) })
    }
}

impl<'a> HasDisplayHandle for Window<'a> {
    fn display_handle(&self) -> Result<RwhDisplayHandle<'_>, HandleError> {
        Ok(unsafe {
            RwhDisplayHandle::borrow_raw(RawDisplayHandle::AppKit(AppKitDisplayHandle::new()))
        })
    }
}

pub fn copy_to_clipboard(string: &str) {
    unsafe {
        let pb = NSPasteboard::generalPasteboard(nil);

        let ns_str = NSString::alloc(nil).init_str(string);

        pb.clearContents();
        pb.setString_forType(ns_str, cocoa::appkit::NSPasteboardTypeString);
    }
}
