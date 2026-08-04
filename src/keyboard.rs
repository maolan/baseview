#[cfg(unix)]
use keyboard_types::{Code, Location};

#[cfg(unix)]
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
