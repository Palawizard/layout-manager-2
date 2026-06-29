use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{geometry::PixelBounds, monitor::MonitorId};
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeWindowHandle(pub isize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowState {
    Normal,
    Maximized,
    Minimized,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopWindow {
    #[serde(skip)]
    pub handle: NativeWindowHandle,
    pub process_id: u32,
    pub executable_path: Option<String>,
    pub process_name: Option<String>,
    pub title: String,
    pub class_name: String,
    pub bounds: PixelBounds,
    pub state: WindowState,
    pub monitor_id: Option<MonitorId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowMatcher {
    pub executable_path: Option<String>,
    pub process_name: Option<String>,
    pub class_name: Option<String>,
    pub title_pattern: Option<String>,
    pub instance_index: Option<usize>,
}

impl WindowMatcher {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.executable_path.is_none()
            && self.process_name.is_none()
            && self.class_name.is_none()
            && self.title_pattern.is_none()
        {
            return Err(AppError::Validation(
                "Ajoutez au moins un critère pour retrouver la fenêtre.".to_owned(),
            ));
        }
        if let Some(pattern) = &self.title_pattern {
            Regex::new(pattern).map_err(|_| {
                AppError::Validation("Le titre à retrouver est invalide.".to_owned())
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::WindowMatcher;

    #[test]
    fn requires_at_least_one_stable_criterion() {
        assert!(WindowMatcher::default().validate().is_err());
    }

    #[test]
    fn rejects_invalid_title_patterns() {
        let matcher = WindowMatcher {
            title_pattern: Some("[".to_owned()),
            ..Default::default()
        };
        assert!(matcher.validate().is_err());
    }
}
