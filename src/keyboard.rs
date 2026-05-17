//! Keyboard types.

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
use keyboard_types::{Code, Location};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
/// Map key code to location.
///
/// The logic for this is adapted from InitKeyEvent in TextInputHandler (in the Mozilla
/// mac port).
///
/// Note: in the original, this is based on kVK constants, but since we don't have those
/// readily available, we use the mapping to code (which should be effectively lossless).
pub fn code_to_location(code: Code) -> Location {
    match code {
        Code::MetaLeft | Code::ShiftLeft | Code::AltLeft | Code::ControlLeft => Location::Left,
        Code::MetaRight | Code::ShiftRight | Code::AltRight | Code::ControlRight => Location::Right,
        Code::Numpad0
        | Code::Numpad1
        | Code::Numpad2
        | Code::Numpad3
        | Code::Numpad4
        | Code::Numpad5
        | Code::Numpad6
        | Code::Numpad7
        | Code::Numpad8
        | Code::Numpad9
        | Code::NumpadAdd
        | Code::NumpadComma
        | Code::NumpadDecimal
        | Code::NumpadDivide
        | Code::NumpadEnter
        | Code::NumpadEqual
        | Code::NumpadMultiply
        | Code::NumpadSubtract => Location::Numpad,
        _ => Location::Standard,
    }
}
