pub use crate::iced::core::time::*;

#[allow(unused_imports)]
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio", feature = "smol", target_arch = "wasm32"))))]
pub use iced_futures::backend::default::time::*;

use crate::iced::task::Task;

pub fn now() -> Task<Instant> {
    Task::future(async { Instant::now() })
}
