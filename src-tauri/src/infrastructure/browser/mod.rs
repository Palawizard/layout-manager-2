mod arguments;
mod detection;
mod launcher;

pub use detection::{InstalledBrowser, detect_installed_browsers, resolve_browser_executable};
pub use launcher::WindowsBrowserLauncher;
