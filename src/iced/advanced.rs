//! Leverage advanced concepts like custom widgets.
pub mod subscription {
    //! Write your own subscriptions.
    pub use crate::iced::runtime::futures::subscription::{
        Event, EventStream, Hasher, Recipe, from_recipe, into_recipes,
    };
}

pub mod widget {
    //! Create custom widgets and operate on them.
    pub use crate::iced::core::widget::*;
    pub use crate::iced::runtime::task::widget as operate;
}

pub use crate::iced::core::Shell;
pub use crate::iced::core::clipboard::{self, Clipboard};
pub use crate::iced::core::image;
pub use crate::iced::core::input_method::{self, InputMethod};
pub use crate::iced::core::layout::{self, Layout};
pub use crate::iced::core::mouse;
pub use crate::iced::core::overlay::{self, Overlay};
pub use crate::iced::core::renderer::{self, Renderer};
pub use crate::iced::core::svg;
pub use crate::iced::core::text::{self, Text};
pub use crate::iced::renderer::graphics;

pub use widget::Widget;
