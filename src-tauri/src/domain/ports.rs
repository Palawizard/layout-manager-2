use thiserror::Error;

use super::{
    geometry::PixelBounds,
    monitor::Monitor,
    window::{DesktopWindow, NativeWindowHandle, WindowState},
};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NativeError {
    #[error("access denied")]
    AccessDenied,
    #[error("invalid window handle")]
    InvalidHandle,
    #[error("native operation failed: {0}")]
    OperationFailed(String),
}

pub trait MonitorProvider: Send + Sync {
    fn list_monitors(&self) -> Result<Vec<Monitor>, NativeError>;
}

pub trait WindowInventory: Send + Sync {
    fn list_windows(&self) -> Result<Vec<DesktopWindow>, NativeError>;
}

pub trait WindowController: Send + Sync {
    fn place_window(
        &self,
        handle: NativeWindowHandle,
        bounds: PixelBounds,
    ) -> Result<(), NativeError>;

    fn set_window_state(
        &self,
        handle: NativeWindowHandle,
        state: WindowState,
    ) -> Result<(), NativeError>;
}

#[cfg(test)]
pub mod fakes {
    use std::sync::Mutex;

    use super::{MonitorProvider, NativeError, WindowController, WindowInventory};
    use crate::domain::{
        geometry::PixelBounds,
        monitor::Monitor,
        window::{DesktopWindow, NativeWindowHandle, WindowState},
    };

    #[derive(Debug, Default)]
    pub struct FakeWindowSystem {
        pub monitors: Vec<Monitor>,
        pub windows: Vec<DesktopWindow>,
        pub placements: Mutex<Vec<(NativeWindowHandle, PixelBounds)>>,
        pub states: Mutex<Vec<(NativeWindowHandle, WindowState)>>,
    }

    impl MonitorProvider for FakeWindowSystem {
        fn list_monitors(&self) -> Result<Vec<Monitor>, NativeError> {
            Ok(self.monitors.clone())
        }
    }

    impl WindowInventory for FakeWindowSystem {
        fn list_windows(&self) -> Result<Vec<DesktopWindow>, NativeError> {
            Ok(self.windows.clone())
        }
    }

    impl WindowController for FakeWindowSystem {
        fn place_window(
            &self,
            handle: NativeWindowHandle,
            bounds: PixelBounds,
        ) -> Result<(), NativeError> {
            self.placements
                .lock()
                .expect("placements lock")
                .push((handle, bounds));
            Ok(())
        }

        fn set_window_state(
            &self,
            handle: NativeWindowHandle,
            state: WindowState,
        ) -> Result<(), NativeError> {
            self.states
                .lock()
                .expect("states lock")
                .push((handle, state));
            Ok(())
        }
    }
}
