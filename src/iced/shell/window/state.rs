use crate::WindowScalePolicy;
use iced_core::keyboard::Modifiers;
use iced_core::theme;
use iced_program::Program;

use crate::iced::core::mouse;
use crate::iced::core::{Color, Size};
use crate::iced::graphics::Viewport;
use crate::iced::program;
use crate::iced::window;

use std::marker::PhantomData;

pub struct State<P: Program> {
    _title: String,
    viewport: Viewport,
    surface_version: u64,
    cursor_position: Option<iced_runtime::core::Point>,
    theme: Option<P::Theme>,
    theme_mode: theme::Mode,
    default_theme: P::Theme,
    style: theme::Style,
    program_scale_factor: f32,
    window_scale_factor: f32,
    scale_policy: WindowScalePolicy,
    pub modifiers: Modifiers,
    program: PhantomData<P>,
}

impl<P: Program> State<P> {
    pub fn new(
        program: &program::Instance<P>, window_id: window::Id, window_physical_size: Size<u32>,
        window_scale_factor: f32, scale_policy: WindowScalePolicy, system_theme: theme::Mode,
    ) -> Self {
        let title = program.title(window_id);
        let program_scale_factor = program.scale_factor(window_id);
        let theme = program.theme(window_id);
        let theme_mode = theme.as_ref().map(theme::Base::mode).unwrap_or_default();
        let default_theme = <P::Theme as theme::Base>::default(system_theme);
        let style = program.style(theme.as_ref().unwrap_or(&default_theme));

        let scale = match scale_policy {
            WindowScalePolicy::ScaleFactor(scale) => scale as f32 * program_scale_factor,
            WindowScalePolicy::SystemScaleFactor => window_scale_factor * program_scale_factor,
        };

        let viewport = Viewport::with_physical_size(window_physical_size, scale);

        Self {
            _title: title,
            viewport,
            surface_version: 0,
            cursor_position: None,
            theme,
            theme_mode,
            default_theme,
            style,
            program_scale_factor,
            window_scale_factor,
            scale_policy,
            modifiers: Default::default(),
            program: PhantomData,
        }
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn surface_version(&self) -> u64 {
        self.surface_version
    }

    pub fn physical_size(&self) -> Size<u32> {
        self.viewport.physical_size()
    }

    pub fn logical_size(&self) -> Size<f32> {
        self.viewport.logical_size()
    }

    pub fn window_scale_factor(&self) -> f32 {
        self.window_scale_factor
    }

    pub fn cursor(&self) -> mouse::Cursor {
        self.cursor_position.map(mouse::Cursor::Available).unwrap_or(mouse::Cursor::Unavailable)
    }

    pub fn theme(&self) -> &P::Theme {
        self.theme.as_ref().unwrap_or(&self.default_theme)
    }

    pub fn background_color(&self) -> Color {
        self.style.background_color
    }

    pub fn text_color(&self) -> Color {
        self.style.text_color
    }

    pub fn update(&mut self, event: &crate::Event) {
        match event {
            crate::Event::Window(crate::WindowEvent::Resized(window_info)) => {
                self.window_scale_factor = window_info.scale() as f32;

                let scale = match self.scale_policy {
                    WindowScalePolicy::ScaleFactor(scale) => {
                        scale as f32 * self.program_scale_factor
                    }
                    WindowScalePolicy::SystemScaleFactor => {
                        self.window_scale_factor * self.program_scale_factor
                    }
                };

                self.viewport = Viewport::with_physical_size(
                    Size::new(
                        window_info.physical_size().width,
                        window_info.physical_size().height,
                    ),
                    scale,
                );

                self.surface_version += 1;
            }
            crate::Event::Mouse(crate::MouseEvent::CursorMoved { position, modifiers: _ }) => {
                self.cursor_position =
                    Some(crate::iced::core::Point { x: position.x as f32, y: position.y as f32 });
            }
            crate::Event::Mouse(crate::MouseEvent::CursorLeft) => {
                self.cursor_position = None;
            }
            crate::Event::Keyboard(_event) => {}
            _ => {}
        }
    }

    pub fn synchronize(&mut self, program: &program::Instance<P>, window_id: window::Id) {
        let new_title = program.title(window_id);
        if self._title != new_title {
            self._title = new_title;
        }

        let new_program_scale_factor = program.scale_factor(window_id);
        if self.program_scale_factor != new_program_scale_factor {
            self.program_scale_factor = new_program_scale_factor;

            let scale = match self.scale_policy {
                WindowScalePolicy::ScaleFactor(scale) => scale as f32 * self.program_scale_factor,
                WindowScalePolicy::SystemScaleFactor => {
                    self.window_scale_factor * self.program_scale_factor
                }
            };

            self.viewport = Viewport::with_physical_size(self.viewport.physical_size(), scale);
        }

        self.theme = program.theme(window_id);
        self.style = program.style(self.theme());

        let new_mode = self.theme.as_ref().map(theme::Base::mode).unwrap_or_default();

        if self.theme_mode != new_mode {
            self.theme_mode = if new_mode == theme::Mode::None {
                theme::Base::mode(&self.default_theme)
            } else {
                new_mode
            };
        }
    }

    pub fn set_system_theme(
        &mut self, window_id: window::Id, system_theme: theme::Mode, program: &program::Instance<P>,
    ) {
        let theme = program.theme(window_id);
        self.theme_mode = theme.as_ref().map(theme::Base::mode).unwrap_or_default();
        self.default_theme = <P::Theme as theme::Base>::default(system_theme);
        self.style = program.style(theme.as_ref().unwrap_or(&self.default_theme));
    }
}
