use crate::iced::futures;
use crate::iced::graphics;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the futures executor could not be created")]
    ExecutorCreationFailed(futures::io::Error),

    #[error("the application window could not be created")]
    WindowCreationFailed,

    #[error("the application graphics context could not be created")]
    GraphicsCreationFailed(graphics::Error),
}

impl From<graphics::Error> for Error {
    fn from(error: graphics::Error) -> Error {
        Error::GraphicsCreationFailed(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_send_sync() {
        fn _assert<T: Send + Sync>() {}
        _assert::<Error>();
    }
}
