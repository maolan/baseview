use crate::iced::core::clipboard::Kind;
use raw_window_handle_06::HasDisplayHandle;

use tracing::warn;

pub struct Clipboard {
    state: State,
}

enum State {
    Connected { clipboard: window_clipboard::Clipboard },
    Unavailable,
}

impl Clipboard {
    pub unsafe fn connect<W: HasDisplayHandle>(window: &W) -> Clipboard {
        let clipboard = unsafe { window_clipboard::Clipboard::connect(window) };

        let state = match clipboard {
            Ok(clipboard) => State::Connected { clipboard },
            Err(_) => State::Unavailable,
        };

        Clipboard { state }
    }

    pub fn unconnected() -> Clipboard {
        Clipboard { state: State::Unavailable }
    }

    pub fn read(&self, kind: Kind) -> Option<String> {
        match &self.state {
            State::Connected { clipboard, .. } => match kind {
                Kind::Standard => clipboard.read().ok(),
                Kind::Primary => clipboard.read_primary().and_then(Result::ok),
            },
            State::Unavailable => None,
        }
    }

    pub fn write(&mut self, kind: Kind, contents: String) {
        match &mut self.state {
            State::Connected { clipboard, .. } => {
                let result = match kind {
                    Kind::Standard => clipboard.write(contents),
                    Kind::Primary => clipboard.write_primary(contents).unwrap_or(Ok(())),
                };

                match result {
                    Ok(()) => {}
                    Err(error) => {
                        warn!("error writing to clipboard: {error}");
                    }
                }
            }
            State::Unavailable => {}
        }
    }
}

impl crate::iced::core::Clipboard for Clipboard {
    fn read(&self, kind: Kind) -> Option<String> {
        self.read(kind)
    }

    fn write(&mut self, kind: Kind, contents: String) {
        self.write(kind, contents);
    }
}
