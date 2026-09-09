#[cfg(target_os = "macos")]
use crate::macos as platform;
#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
)))]
use crate::unsupported as platform;
#[cfg(target_os = "windows")]
use crate::win as platform;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use crate::x11 as platform;

pub fn copy_to_clipboard(data: &str) {
    platform::copy_to_clipboard(data)
}
