use std::time::Instant;
use std::{cell::RefCell, pin::Pin, rc::Rc};

use crate::{Event, EventStatus, Window, WindowHandler};
use iced_core::{mouse, theme};
use iced_program::Program;
use iced_runtime::Task;
pub use iced_runtime::core::window::Id;
use iced_runtime::futures::futures::{
    self,
    channel::mpsc::{self, SendError},
};
pub use iced_runtime::window::{close_events, close_requests, events, open_events, resize_events};
use iced_widget::core::Size;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use tracing::error;

use crate::iced::graphics::Compositor;
use crate::iced::shell::RuntimeEvent;
use crate::iced::shell::conversion::WindowWrapper;

pub(super) mod state;

pub(super) struct InstanceWindow<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    pub state: state::State<P>,
    pub mouse_interaction: mouse::Interaction,
    pub surface: C::Surface,
    pub surface_version: u64,
    pub compositor: C,
    pub renderer: P::Renderer,

    pub queue: WindowQueue,
    pub window06: WindowWrapper,
    pub id: iced_core::window::Id,
    #[allow(unused)]
    pub always_redraw: bool,
    pub ignore_non_modifier_keys: bool,
    pub redraw_requested: bool,
    pub redraw_at: Option<Instant>,
}

pub(super) struct IcedWindowHandler<P: Program> {
    pub sender: mpsc::UnboundedSender<RuntimeEvent<P::Message>>,
    pub instance: Pin<Box<dyn futures::Future<Output = ()>>>,
    pub runtime_context: futures::task::Context<'static>,
    pub runtime_rx: mpsc::UnboundedReceiver<iced_runtime::Action<P::Message>>,
    pub window_queue_rx: mpsc::UnboundedReceiver<WindowCommand>,
    pub event_status: Rc<RefCell<EventStatus>>,
    pub processed_close_signal: bool,
}

impl<P> IcedWindowHandler<P>
where
    P: Program + 'static,
{
    fn drain_window_commands(&mut self, window: &mut Window<'_>) {
        while let Ok(cmd) = self.window_queue_rx.try_recv() {
            match cmd {
                WindowCommand::CloseWindow => {
                    window.close();
                }
                WindowCommand::ResizeWindow(size) => {
                    window.resize(crate::Size {
                        width: size.width as f64,
                        height: size.height as f64,
                    });
                }
                WindowCommand::Focus => {
                    window.focus();
                }
                WindowCommand::SetCursorIcon(cursor) => {
                    window.set_mouse_cursor(cursor);
                }
            }
        }
    }
}

impl<P: Program + 'static> WindowHandler for IcedWindowHandler<P> {
    fn on_frame(&mut self, window: &mut Window<'_>) {
        if self.processed_close_signal {
            return;
        }

        self.sender.start_send(RuntimeEvent::Poll).expect("Send event");

        let _ = self.instance.as_mut().poll(&mut self.runtime_context);

        while let Ok(message) = self.runtime_rx.try_recv() {
            self.sender.start_send(RuntimeEvent::UserEvent(message)).expect("Send event");
        }

        self.sender.start_send(RuntimeEvent::OnFrame).expect("Send event");

        let _ = self.instance.as_mut().poll(&mut self.runtime_context);

        self.drain_window_commands(window);
    }

    fn on_event(&mut self, window: &mut Window<'_>, event: Event) -> EventStatus {
        if self.processed_close_signal {
            return EventStatus::Ignored;
        }

        #[cfg(not(target_os = "linux"))]
        if matches!(event, Event::Mouse(crate::MouseEvent::ButtonPressed { .. }))
            && !window.has_focus()
        {
            window.focus();
        }

        let status = if requests_exit(&event) {
            self.processed_close_signal = true;

            self.sender.start_send(RuntimeEvent::WillClose).expect("Send event");

            let _ = self.instance.as_mut().poll(&mut self.runtime_context);

            EventStatus::Ignored
        } else {
            self.sender.start_send(RuntimeEvent::Baseview((event, true))).expect("Send event");

            let _ = self.instance.as_mut().poll(&mut self.runtime_context);

            *self.event_status.borrow()
        };

        if !self.processed_close_signal {
            self.drain_window_commands(window);
        }

        status
    }
}

pub fn close<T>() -> Task<T> {
    iced_runtime::window::close(Id::unique())
}

pub fn resize<T>(new_size: Size) -> Task<T> {
    iced_runtime::window::resize(Id::unique(), new_size)
}

pub fn gain_focus<T>() -> Task<T> {
    iced_runtime::window::gain_focus(Id::unique())
}

pub fn requests_exit(event: &crate::Event) -> bool {
    matches!(event, crate::Event::Window(crate::WindowEvent::WillClose))
}

#[allow(missing_debug_implementations)]
pub struct WindowHandle<Message: 'static + Send> {
    bv_handle: crate::WindowHandle,
    tx: mpsc::UnboundedSender<RuntimeEvent<Message>>,
}

impl<Message: 'static + Send> WindowHandle<Message> {
    pub(crate) fn new(
        bv_handle: crate::WindowHandle, tx: mpsc::UnboundedSender<RuntimeEvent<Message>>,
    ) -> Self {
        Self { bv_handle, tx }
    }

    pub fn send_baseview_event(&mut self, event: crate::Event) -> Result<(), SendError> {
        self.tx.start_send(RuntimeEvent::Baseview((event, false)))
    }

    pub fn send_message(&mut self, msg: Message) -> Result<(), SendError> {
        self.tx.start_send(RuntimeEvent::UserEvent(iced_runtime::Action::Output(msg)))
    }

    pub fn close_window(&mut self) {
        self.bv_handle.close();
    }

    pub fn is_open(&self) -> bool {
        self.bv_handle.is_open()
    }
}

impl<Message: 'static + Send> Drop for WindowHandle<Message> {
    fn drop(&mut self) {
        self.close_window();
    }
}

unsafe impl<Message: 'static + Send> HasRawWindowHandle for WindowHandle<Message> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.bv_handle.raw_window_handle()
    }
}

pub enum WindowCommand {
    CloseWindow,
    ResizeWindow(crate::iced::core::Size),
    Focus,
    SetCursorIcon(crate::MouseCursor),
}

pub struct WindowQueue {
    tx: mpsc::UnboundedSender<WindowCommand>,
}

impl WindowQueue {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<WindowCommand>) {
        let (tx, rx) = mpsc::unbounded();

        (Self { tx }, rx)
    }

    pub fn send(&mut self, command: WindowCommand) {
        if let Err(e) = self.tx.start_send(command) {
            error!("Failed to send command to window: {}", e);
        }
    }
}
