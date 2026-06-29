use thiserror::Error;

use super::{
    geometry::PixelBounds,
    layout::{Layout, LayoutId, LayoutSummary},
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

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessLaunchRequest {
    pub executable_path: String,
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchedProcess {
    pub process_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProcessLaunchError {
    #[error("executable not found")]
    ExecutableNotFound,
    #[error("launch failed: {0}")]
    LaunchFailed(String),
}

pub trait ProcessLauncher: Send + Sync {
    fn launch(&self, request: ProcessLaunchRequest) -> Result<LaunchedProcess, ProcessLaunchError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserLaunchRequest {
    pub executable_path: String,
    pub arguments: Vec<String>,
}

pub trait BrowserLauncher: Send + Sync {
    fn launch(&self, request: BrowserLaunchRequest) -> Result<LaunchedProcess, ProcessLaunchError>;
}

pub trait LayoutRepository: Send + Sync {
    fn list_summaries(&self) -> Result<Vec<LayoutSummary>, AppError>;
    fn get(&self, id: &LayoutId) -> Result<Layout, AppError>;
    fn save(&self, layout: &Layout) -> Result<(), AppError>;
    fn delete(&self, id: &LayoutId) -> Result<(), AppError>;
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
