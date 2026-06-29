use serde::{Deserialize, Serialize};

use super::layout::BrowserKind;
use super::monitor::MonitorFallback;
use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub preferred_browser: BrowserKind,
    pub default_startup_timeout_ms: u32,
    pub monitor_fallback: MonitorFallback,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            preferred_browser: BrowserKind::Edge,
            default_startup_timeout_ms: 15_000,
            monitor_fallback: MonitorFallback::Primary,
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), AppError> {
        if !(1_000..=120_000).contains(&self.default_startup_timeout_ms) {
            return Err(AppError::Validation(
                "Le délai d’attente par défaut est invalide.".to_owned(),
            ));
        }
        Ok(())
    }
}
