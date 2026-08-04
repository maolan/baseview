use std::marker::PhantomData;

use raw_window_handle::{
    DisplayHandle as RwhDisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle,
    WindowHandle as RwhWindowHandle,
};

use crate::event::{Event, EventStatus};
use crate::window_open_options::WindowOpenOptions;
use crate::{MouseCursor, Size};

#[cfg(target_os = "windows")]
use crate::win as platform;
#[cfg(unix)]
use crate::x11 as platform;

pub struct WindowHandle {
    window_handle: platform::WindowHandle,

    phantom: PhantomData<*mut ()>,
}

impl WindowHandle {
    fn new(window_handle: platform::WindowHandle) -> Self {
        Self { window_handle, phantom: PhantomData }
    }

    pub fn close(&mut self) {
        self.window_handle.close();
    }

    pub fn is_open(&self) -> bool {
        self.window_handle.is_open()
    }
}

impl HasWindowHandle for WindowHandle {
    fn window_handle(&self) -> Result<RwhWindowHandle<'_>, HandleError> {
        self.window_handle.window_handle()
    }
}

pub trait WindowHandler {
    fn on_frame(&mut self, window: &mut Window);
    fn on_event(&mut self, window: &mut Window, event: Event) -> EventStatus;
}

pub struct Window<'a> {
    window: platform::Window<'a>,

    phantom: PhantomData<*mut ()>,
}

impl<'a> Window<'a> {
    #[cfg(target_os = "windows")]
    pub(crate) fn new(window: platform::Window<'a>) -> Window<'a> {
        Window { window, phantom: PhantomData }
    }

    #[cfg(not(target_os = "windows"))]
    pub(crate) fn new(window: platform::Window) -> Window {
        Window { window, phantom: PhantomData }
    }

    pub fn open_parented<P, H, B>(parent: &P, options: WindowOpenOptions, build: B) -> WindowHandle
    where
        P: HasWindowHandle,
        H: WindowHandler + 'static,
        B: FnOnce(&mut Window) -> H,
        B: Send + 'static,
    {
        let window_handle = platform::Window::open_parented::<P, H, B>(parent, options, build);
        WindowHandle::new(window_handle)
    }

    pub fn open_blocking<H, B>(options: WindowOpenOptions, build: B)
    where
        H: WindowHandler + 'static,
        B: FnOnce(&mut Window) -> H,
        B: Send + 'static,
    {
        platform::Window::open_blocking::<H, B>(options, build)
    }

    pub fn close(&mut self) {
        self.window.close();
    }

    pub fn resize(&mut self, size: Size) {
        self.window.resize(size);
    }

    pub fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        self.window.set_mouse_cursor(cursor);
    }

    pub fn has_focus(&mut self) -> bool {
        self.window.has_focus()
    }

    pub fn focus(&mut self) {
        self.window.focus()
    }

    #[cfg(feature = "opengl")]
    pub fn gl_context(&self) -> Option<&crate::gl::GlContext> {
        self.window.gl_context()
    }
}

impl<'a> HasWindowHandle for Window<'a> {
    fn window_handle(&self) -> Result<RwhWindowHandle<'_>, HandleError> {
        self.window.window_handle()
    }
}

impl<'a> HasDisplayHandle for Window<'a> {
    fn display_handle(&self) -> Result<RwhDisplayHandle<'_>, HandleError> {
        self.window.display_handle()
    }
}
