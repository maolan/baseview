use std::ffi::c_void;
use std::marker::PhantomData;

#[cfg(not(unix))]
use raw_window_handle::RawWindowHandle;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
use win as platform;

#[cfg(unix)]
pub(crate) mod x11;
#[cfg(unix)]
pub(crate) use self::x11 as platform;

#[derive(Clone, Debug)]
pub struct GlConfig {
    pub version: (u8, u8),
    pub profile: Profile,
    pub red_bits: u8,
    pub blue_bits: u8,
    pub green_bits: u8,
    pub alpha_bits: u8,
    pub depth_bits: u8,
    pub stencil_bits: u8,
    pub samples: Option<u8>,
    pub srgb: bool,
    pub double_buffer: bool,
    pub vsync: bool,
}

impl Default for GlConfig {
    fn default() -> Self {
        GlConfig {
            version: (3, 2),
            profile: Profile::Core,
            red_bits: 8,
            blue_bits: 8,
            green_bits: 8,
            alpha_bits: 8,
            depth_bits: 24,
            stencil_bits: 8,
            samples: None,
            srgb: true,
            double_buffer: true,
            vsync: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Compatibility,
    Core,
}

#[derive(Debug)]
pub enum GlError {
    InvalidWindowHandle,
    VersionNotSupported,
    CreationFailed(platform::CreationFailedError),
}

pub struct GlContext {
    context: platform::GlContext,
    phantom: PhantomData<*mut ()>,
}

impl GlContext {
    #[cfg(not(unix))]
    pub(crate) unsafe fn create(
        parent: &RawWindowHandle, config: GlConfig,
    ) -> Result<GlContext, GlError> {
        platform::GlContext::create(parent, config)
            .map(|context| GlContext { context, phantom: PhantomData })
    }

    #[cfg(unix)]
    pub(crate) fn new(context: platform::GlContext) -> GlContext {
        GlContext { context, phantom: PhantomData }
    }

    pub unsafe fn make_current(&self) {
        self.context.make_current();
    }

    pub unsafe fn make_not_current(&self) {
        self.context.make_not_current();
    }

    pub fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.context.get_proc_address(symbol)
    }

    pub fn swap_buffers(&self) {
        self.context.swap_buffers();
    }
}
