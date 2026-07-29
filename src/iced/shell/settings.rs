use crate::{Size, WindowOpenOptions, WindowScalePolicy};

pub struct IcedBaseviewSettings {
    pub window: WindowOpenOptions,

    pub ignore_non_modifier_keys: bool,

    pub always_redraw: bool,
}

impl Default for IcedBaseviewSettings {
    fn default() -> Self {
        Self {
            window: WindowOpenOptions {
                title: String::from("iced_baseview"),
                size: Size::new(500.0, 300.0),
                scale: WindowScalePolicy::SystemScaleFactor,
                #[cfg(feature = "rustanalyzer_opengl_workaround")]
                gl_config: None,
            },
            ignore_non_modifier_keys: false,
            always_redraw: false,
        }
    }
}

impl Clone for IcedBaseviewSettings {
    fn clone(&self) -> Self {
        Self {
            window: WindowOpenOptions {
                title: self.window.title.clone(),
                size: self.window.size,
                scale: self.window.scale,
                #[cfg(feature = "rustanalyzer_opengl_workaround")]
                gl_config: None,
            },
            ignore_non_modifier_keys: self.ignore_non_modifier_keys,
            always_redraw: self.always_redraw,
        }
    }
}
