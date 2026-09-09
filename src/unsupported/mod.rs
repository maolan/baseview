//! Stub backend for platforms without windowing support (e.g. macOS).
//!
//! This backend only exists so that the crate compiles on unsupported
//! platforms. Opening a window panics at runtime; the GUI-related entry
//! points on `crate::Window` and `copy_to_clipboard` should not be reached.

use std::marker::PhantomData;

use raw_window_handle::{
    DisplayHandle as RwhDisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle,
    WindowHandle as RwhWindowHandle,
};

use crate::window::WindowHandler;
use crate::window_open_options::WindowOpenOptions;
use crate::{MouseCursor, Size};

fn unsupported() -> ! {
    panic!("baseview: GUI windows are not supported on this platform")
}

pub struct WindowHandle;

impl WindowHandle {
    pub fn close(&mut self) {}

    pub fn is_open(&self) -> bool {
        false
    }
}

impl HasWindowHandle for WindowHandle {
    fn window_handle(&self) -> Result<RwhWindowHandle<'_>, HandleError> {
        Err(HandleError::NotSupported)
    }
}

pub struct Window<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> Window<'a> {
    pub fn open_parented<P, H, B>(
        _parent: &P, _options: WindowOpenOptions, _build: B,
    ) -> WindowHandle
    where
        P: HasWindowHandle,
        H: WindowHandler + 'static,
        B: FnOnce(&mut crate::Window) -> H,
        B: Send + 'static,
    {
        unsupported()
    }

    pub fn open_blocking<H, B>(_options: WindowOpenOptions, _build: B)
    where
        H: WindowHandler + 'static,
        B: FnOnce(&mut crate::Window) -> H,
        B: Send + 'static,
    {
        unsupported()
    }

    pub fn close(&mut self) {}

    pub fn resize(&mut self, _size: Size) {}

    pub fn set_mouse_cursor(&mut self, _cursor: MouseCursor) {}

    pub fn has_focus(&mut self) -> bool {
        false
    }

    pub fn focus(&self) {}

    #[cfg(feature = "opengl")]
    pub fn gl_context(&self) -> Option<&crate::gl::GlContext> {
        None
    }
}

impl HasWindowHandle for Window<'_> {
    fn window_handle(&self) -> Result<RwhWindowHandle<'_>, HandleError> {
        Err(HandleError::NotSupported)
    }
}

impl HasDisplayHandle for Window<'_> {
    fn display_handle(&self) -> Result<RwhDisplayHandle<'_>, HandleError> {
        Err(HandleError::NotSupported)
    }
}

pub fn copy_to_clipboard(_data: &str) {}
