use std::path::PathBuf;

use keyboard_types::{KeyboardEvent, Modifiers};

use crate::{Point, WindowInfo};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDelta {
    Lines { x: f32, y: f32 },

    Pixels { x: f32, y: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseEvent {
    CursorMoved { position: Point, modifiers: Modifiers },

    ButtonPressed { button: MouseButton, modifiers: Modifiers },

    ButtonReleased { button: MouseButton, modifiers: Modifiers },

    WheelScrolled { delta: ScrollDelta, modifiers: Modifiers },

    CursorEntered,

    CursorLeft,

    DragEntered { position: Point, modifiers: Modifiers, data: DropData },

    DragMoved { position: Point, modifiers: Modifiers, data: DropData },

    DragLeft,

    DragDropped { position: Point, modifiers: Modifiers, data: DropData },
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
    Resized(WindowInfo),
    Focused,
    Unfocused,
    WillClose,
}

#[derive(Debug, Clone)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Window(WindowEvent),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropEffect {
    Copy,
    Move,
    Link,
    Scroll,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DropData {
    None,
    Files(Vec<PathBuf>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventStatus {
    Captured,

    Ignored,

    AcceptDrop(DropEffect),
}
