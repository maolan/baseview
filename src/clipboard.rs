#[cfg(target_os = "windows")]
use crate::win as platform;
#[cfg(unix)]
use crate::x11 as platform;

pub fn copy_to_clipboard(data: &str) {
    platform::copy_to_clipboard(data)
}
