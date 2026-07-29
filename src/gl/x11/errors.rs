use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use x11::xlib;

use std::cell::RefCell;
use std::error::Error;
use std::os::raw::{c_int, c_uchar, c_ulong};
use std::panic::AssertUnwindSafe;

thread_local! {


    static CURRENT_X11_ERROR: RefCell<Option<XLibError>> = const { RefCell::new(None) };
}

pub struct XErrorHandler<'a> {
    display: *mut xlib::Display,
    error: &'a RefCell<Option<XLibError>>,
}

impl<'a> XErrorHandler<'a> {
    pub fn check(&mut self) -> Result<(), XLibError> {
        unsafe {
            xlib::XSync(self.display, 0);
        }
        let error = self.error.borrow_mut().take();

        match error {
            None => Ok(()),
            Some(inner) => Err(inner),
        }
    }

    pub unsafe fn handle<T, F: FnOnce(&mut XErrorHandler) -> T>(
        display: *mut xlib::Display, handler: F,
    ) -> T {
        unsafe extern "C" fn error_handler(
            _dpy: *mut xlib::Display, err: *mut xlib::XErrorEvent,
        ) -> i32 {
            let err = &*err;

            CURRENT_X11_ERROR.with(|error| {
                let mut error = error.borrow_mut();
                match error.as_mut() {
                    Some(_) => 1,
                    None => {
                        *error = Some(XLibError::from_event(err));
                        0
                    }
                }
            })
        }

        unsafe {
            xlib::XSync(display, 0);
        }

        CURRENT_X11_ERROR.with(|error| {
            {
                *error.borrow_mut() = None;
            }

            let old_handler = unsafe { xlib::XSetErrorHandler(Some(error_handler)) };
            let panic_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                let mut h = XErrorHandler { display, error };
                handler(&mut h)
            }));

            unsafe { xlib::XSetErrorHandler(old_handler) };

            match panic_result {
                Ok(v) => v,
                Err(e) => std::panic::resume_unwind(e),
            }
        })
    }
}

pub struct XLibError {
    type_: c_int,
    resourceid: xlib::XID,
    serial: c_ulong,
    error_code: c_uchar,
    request_code: c_uchar,
    minor_code: c_uchar,

    display_name: Box<str>,
}

impl XLibError {
    unsafe fn from_event(error: &xlib::XErrorEvent) -> Self {
        Self {
            type_: error.type_,
            resourceid: error.resourceid,
            serial: error.serial,

            error_code: error.error_code,
            request_code: error.request_code,
            minor_code: error.minor_code,

            display_name: Self::get_display_name(error),
        }
    }

    unsafe fn get_display_name(error: &xlib::XErrorEvent) -> Box<str> {
        let mut buf = [0; 255];
        unsafe {
            xlib::XGetErrorText(
                error.display,
                error.error_code.into(),
                buf.as_mut_ptr().cast(),
                (buf.len() - 1) as i32,
            );
        }

        *buf.last_mut().unwrap() = 0;

        let cstr = unsafe { CStr::from_ptr(buf.as_mut_ptr().cast()) };

        cstr.to_string_lossy().into()
    }
}

impl Debug for XLibError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XLibError")
            .field("error_code", &self.error_code)
            .field("error_message", &self.display_name)
            .field("minor_code", &self.minor_code)
            .field("request_code", &self.request_code)
            .field("type", &self.type_)
            .field("resource_id", &self.resourceid)
            .field("serial", &self.serial)
            .finish()
    }
}

impl Display for XLibError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "XLib error: {} (error code {})", &self.display_name, self.error_code)
    }
}

impl Error for XLibError {}
