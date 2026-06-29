#![allow(unsafe_code)]

use ::windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};

mod controller;
#[path = "windows.rs"]
mod inventory;
mod monitors;
mod process;

#[derive(Debug, Default)]
pub(crate) struct Win32WindowSystem;

impl Win32WindowSystem {
    #[must_use]
    pub(crate) fn new() -> Self {
        // SAFETY: The constant is a process-wide predefined DPI context and contains no borrowed data.
        if unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }
            .is_err()
        {
            tracing::debug!("DPI awareness was already configured by the application runtime");
        }
        Self
    }
}
