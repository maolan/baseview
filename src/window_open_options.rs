use crate::Size;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowScalePolicy {
    SystemScaleFactor,

    ScaleFactor(f64),
}

pub struct WindowOpenOptions {
    pub title: String,

    pub size: Size,

    pub scale: WindowScalePolicy,

    #[cfg(feature = "opengl")]
    pub gl_config: Option<crate::gl::GlConfig>,
}
