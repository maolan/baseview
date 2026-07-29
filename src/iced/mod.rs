#[cfg(not(any(
    target_arch = "wasm32",
    feature = "thread-pool",
    feature = "tokio",
    feature = "smol"
)))]
compile_error!(
    "No futures executor has been enabled! You must enable an executor feature.\nAvailable \
     options: thread-pool, tokio, or smol."
);

#[cfg(all(target_family = "unix", not(feature = "x11")))]
compile_error!(
    "No Unix display server backend has been enabled. You must enable a display server \
     feature.\nAvailable options: x11, wayland."
);

pub use crate as baseview;
pub use iced_program as program;
use iced_widget::graphics;
use iced_widget::renderer;
pub use program::core;
pub use program::runtime;

pub use iced_futures::futures;
pub use iced_futures::stream;

#[cfg(feature = "app-builder")]
pub mod application;

pub mod shell;
pub mod time;

mod error;

#[cfg(feature = "highlighter")]
pub use iced_highlighter as highlighter;

#[cfg(feature = "wgpu")]
pub use iced_renderer::wgpu::wgpu;

#[cfg(feature = "advanced")]
pub mod advanced;

pub use crate::iced::core::alignment;
pub use crate::iced::core::animation;
pub use crate::iced::core::border;
pub use crate::iced::core::color;
pub use crate::iced::core::gradient;
pub use crate::iced::core::padding;
pub use crate::iced::core::theme;
pub use crate::iced::core::{
    Alignment, Animation, Background, Border, Color, ContentFit, Degrees, Function, Gradient,
    Length, Never, Padding, Pixels, Point, Radians, Rectangle, Rotation, Settings, Shadow, Size,
    Theme, Transformation, Vector, never,
};
pub use crate::iced::program::Preset;
pub use crate::iced::program::message;
pub use crate::iced::runtime::exit;
pub use iced_futures::Subscription;

pub use Alignment::Center;
pub use Length::{Fill, FillPortion, Shrink};
pub use alignment::Horizontal::{Left, Right};
pub use alignment::Vertical::{Bottom, Top};

pub mod debug {

    pub use iced_debug::{Span, time, time_with};
}

pub mod task {

    pub use crate::iced::runtime::task::{Handle, Task};

    #[cfg(feature = "sipper")]
    pub use crate::iced::runtime::task::{Never, Sipper, Straw, sipper, stream};
}

pub mod clipboard {

    pub use crate::iced::runtime::clipboard::{read, read_primary, write, write_primary};
}

pub mod executor {

    pub use iced_futures::Executor;
    pub use iced_futures::backend::default::Executor as Default;
}

pub mod font {

    pub use crate::iced::core::font::*;
    pub use crate::iced::runtime::font::*;
}

pub mod event {

    pub use crate::iced::core::event::{Event, Status};
    pub use iced_futures::event::{listen, listen_raw, listen_with};
}

pub mod keyboard {

    pub use crate::iced::core::keyboard::key;
    pub use crate::iced::core::keyboard::{Event, Key, Location, Modifiers};
    pub use iced_futures::keyboard::listen;
}

pub mod mouse {

    pub use crate::iced::core::mouse::{Button, Cursor, Event, Interaction, ScrollDelta};
}

#[cfg(feature = "sysinfo")]
pub mod system {

    pub use crate::iced::runtime::system::{Information, information};
}

pub mod overlay {

    pub type Element<'a, Message, Theme = crate::iced::Renderer, Renderer = crate::iced::Renderer> =
        crate::iced::core::overlay::Element<'a, Message, Theme, Renderer>;

    pub use iced_widget::overlay::*;
}

pub mod touch {

    pub use crate::iced::core::touch::{Event, Finger};
}

#[allow(hidden_glob_reexports)]
pub mod widget {

    pub use iced_runtime::widget::*;
    pub use iced_widget::*;

    #[cfg(feature = "image")]
    pub mod image {

        pub use iced_runtime::image::{Allocation, Error, allocate};
        pub use iced_widget::image::*;
    }

    mod core {}
    mod graphics {}
    mod renderer {}
}

pub mod window {
    pub use crate::iced::core::window::*;
    pub use crate::iced::runtime::window::*;
}

#[cfg(feature = "app-builder")]
pub use application::Application;

pub use error::Error;
pub use event::Event;
pub use executor::Executor;
pub use font::Font;
pub use program::Program;
pub use renderer::Renderer;
pub use shell::{
    PollSubNotifier, open_blocking, open_parented, poll_events, settings::IcedBaseviewSettings,
};
pub use task::Task;
pub use window::Window;

#[cfg(feature = "app-builder")]
#[doc(inline)]
pub use application::application;

pub type Element<'a, Message, Theme = crate::iced::Theme, Renderer = crate::iced::Renderer> =
    crate::iced::core::Element<'a, Message, Theme, Renderer>;

pub type Result = std::result::Result<(), Error>;
