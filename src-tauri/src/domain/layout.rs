use serde::{Deserialize, Serialize};

use super::{
    geometry::NormalizedBounds,
    monitor::MonitorSelector,
    window::{WindowMatcher, WindowState},
};
use crate::error::AppError;

pub const LAYOUT_NAME_MAX_LEN: usize = 80;
pub const LAYOUT_DESCRIPTION_MAX_LEN: usize = 300;
pub const STARTUP_TIMEOUT_MIN_MS: u32 = 1_000;
pub const STARTUP_TIMEOUT_MAX_MS: u32 = 120_000;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LayoutId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LayoutActionId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserKind {
    Edge,
    Chrome,
    Firefox,
    SystemDefault,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPlacement {
    pub monitor_selector: MonitorSelector,
    pub bounds: NormalizedBounds,
    pub state: WindowState,
}

impl WindowPlacement {
    pub fn validate(&self) -> Result<(), AppError> {
        NormalizedBounds::new(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.bounds.height,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutOptions {
    pub minimize_unmatched_windows: bool,
    pub continue_on_error: bool,
    pub restore_previous_state_on_cancel: bool,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            minimize_unmatched_windows: false,
            continue_on_error: true,
            restore_previous_state_on_cancel: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum LayoutAction {
    LaunchApplication {
        id: LayoutActionId,
        executable_path: String,
        arguments: Vec<String>,
        working_directory: Option<String>,
        reuse_existing_window: bool,
        window_matcher: WindowMatcher,
        placement: WindowPlacement,
        startup_timeout_ms: u32,
    },
    PlaceExistingWindow {
        id: LayoutActionId,
        window_matcher: WindowMatcher,
        placement: WindowPlacement,
    },
    OpenBrowserWindow {
        id: LayoutActionId,
        browser_kind: BrowserKind,
        executable_path: Option<String>,
        profile: Option<String>,
        urls: Vec<String>,
        placement: WindowPlacement,
        startup_timeout_ms: u32,
    },
}

impl LayoutAction {
    #[must_use]
    pub fn id(&self) -> &LayoutActionId {
        match self {
            Self::LaunchApplication { id, .. }
            | Self::PlaceExistingWindow { id, .. }
            | Self::OpenBrowserWindow { id, .. } => id,
        }
    }

    pub fn validate(&self) -> Result<(), AppError> {
        match self {
            Self::LaunchApplication {
                executable_path,
                arguments,
                working_directory,
                window_matcher,
                placement,
                startup_timeout_ms,
                ..
            } => {
                validate_executable_path(executable_path)?;
                validate_arguments(arguments)?;
                if let Some(directory) = working_directory {
                    validate_optional_path(directory, "Le répertoire de travail est invalide.")?;
                }
                window_matcher.validate()?;
                placement.validate()?;
                validate_startup_timeout(*startup_timeout_ms)?;
            }
            Self::PlaceExistingWindow {
                window_matcher,
                placement,
                ..
            } => {
                window_matcher.validate()?;
                placement.validate()?;
            }
            Self::OpenBrowserWindow {
                urls,
                placement,
                startup_timeout_ms,
                ..
            } => {
                if urls.is_empty() {
                    return Err(AppError::Validation(
                        "Ajoutez au moins une adresse web.".to_owned(),
                    ));
                }
                for url in urls {
                    validate_url(url)?;
                }
                placement.validate()?;
                validate_startup_timeout(*startup_timeout_ms)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    pub id: LayoutId,
    pub name: String,
    pub description: Option<String>,
    pub actions: Vec<LayoutAction>,
    pub options: LayoutOptions,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutSummary {
    pub id: LayoutId,
    pub name: String,
    pub description: Option<String>,
    pub action_count: usize,
    pub updated_at: i64,
}

impl Layout {
    pub fn validate(&self, require_actions: bool) -> Result<(), AppError> {
        validate_layout_name(&self.name)?;
        validate_layout_description(self.description.as_deref())?;
        if require_actions && self.actions.is_empty() {
            return Err(AppError::Validation(
                "Ajoutez au moins une action.".to_owned(),
            ));
        }
        let mut seen_ids = std::collections::HashSet::new();
        for action in &self.actions {
            if !seen_ids.insert(action.id().clone()) {
                return Err(AppError::Validation(
                    "Chaque action doit avoir un identifiant unique.".to_owned(),
                ));
            }
            action.validate()?;
        }
        Ok(())
    }

    #[must_use]
    pub fn summary(&self) -> LayoutSummary {
        LayoutSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            action_count: self.actions.len(),
            updated_at: self.updated_at,
        }
    }

    pub fn normalized(self) -> Result<Self, AppError> {
        Ok(Self {
            name: validate_layout_name(&self.name)?,
            description: validate_layout_description(self.description.as_deref())?,
            ..self
        })
    }
}

pub fn validate_layout_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Le nom est requis.".to_owned()));
    }
    if trimmed.chars().count() > LAYOUT_NAME_MAX_LEN {
        return Err(AppError::Validation(format!(
            "Le nom ne peut pas dépasser {LAYOUT_NAME_MAX_LEN} caractères."
        )));
    }
    Ok(trimmed.to_owned())
}

pub fn validate_layout_description(description: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(raw) = description else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > LAYOUT_DESCRIPTION_MAX_LEN {
        return Err(AppError::Validation(format!(
            "La description ne peut pas dépasser {LAYOUT_DESCRIPTION_MAX_LEN} caractères."
        )));
    }
    Ok(Some(trimmed.to_owned()))
}

pub fn validate_executable_path(path: &str) -> Result<(), AppError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Le chemin de l’application est requis.".to_owned(),
        ));
    }
    if !std::path::Path::new(trimmed).is_absolute() {
        return Err(AppError::Validation(
            "Choisissez un fichier d’application valide.".to_owned(),
        ));
    }
    Ok(())
}

pub fn validate_url(url: &str) -> Result<(), AppError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "L’adresse web est requise.".to_owned(),
        ));
    }
    let parsed = url::Url::parse(trimmed).map_err(|_| {
        AppError::Validation("L’adresse web est invalide.".to_owned())
    })?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => Err(AppError::Validation(
            "L’adresse web doit commencer par http:// ou https://.".to_owned(),
        )),
    }
}

fn validate_arguments(arguments: &[String]) -> Result<(), AppError> {
    for argument in arguments {
        if argument.contains('\0') {
            return Err(AppError::Validation(
                "Les arguments contiennent un caractère invalide.".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_optional_path(path: &str, message: &str) -> Result<(), AppError> {
    if path.trim().is_empty() || !std::path::Path::new(path.trim()).is_absolute() {
        return Err(AppError::Validation(message.to_owned()));
    }
    Ok(())
}

fn validate_startup_timeout(timeout_ms: u32) -> Result<(), AppError> {
    if !(STARTUP_TIMEOUT_MIN_MS..=STARTUP_TIMEOUT_MAX_MS).contains(&timeout_ms) {
        return Err(AppError::Validation(format!(
            "Le délai d’attente doit être compris entre {} et {} secondes.",
            STARTUP_TIMEOUT_MIN_MS / 1000,
            STARTUP_TIMEOUT_MAX_MS / 1000
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        BrowserKind, Layout, LayoutAction, LayoutActionId, LayoutId, LayoutOptions,
        WindowPlacement, validate_layout_name, validate_url,
    };
    use crate::domain::{
        geometry::NormalizedBounds,
        monitor::{MonitorFallback, MonitorId, MonitorSelector},
        window::WindowMatcher,
    };

    fn sample_placement() -> WindowPlacement {
        WindowPlacement {
            monitor_selector: MonitorSelector {
                preferred_id: MonitorId("primary".to_owned()),
                fallback: MonitorFallback::Primary,
            },
            bounds: NormalizedBounds::new(0.0, 0.0, 1.0, 1.0).expect("valid bounds"),
            state: crate::domain::window::WindowState::Normal,
        }
    }

    fn sample_layout() -> Layout {
        Layout {
            id: LayoutId("layout-1".to_owned()),
            name: "Travail".to_owned(),
            description: Some("Bureau principal".to_owned()),
            actions: vec![LayoutAction::PlaceExistingWindow {
                id: LayoutActionId("action-1".to_owned()),
                window_matcher: WindowMatcher {
                    process_name: Some("notepad.exe".to_owned()),
                    ..Default::default()
                },
                placement: sample_placement(),
            }],
            options: LayoutOptions::default(),
            created_at: 1,
            updated_at: 2,
        }
    }

    #[test]
    fn validates_layout_names_and_descriptions() {
        assert_eq!(
            validate_layout_name("  Travail  ").expect("name"),
            "Travail"
        );
        assert!(validate_layout_name(" ").is_err());
        assert!(validate_layout_name(&"a".repeat(81)).is_err());
    }

    #[test]
    fn validates_urls_for_browser_actions() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn rejects_duplicate_action_ids() {
        let mut layout = sample_layout();
        layout.actions.push(LayoutAction::OpenBrowserWindow {
            id: LayoutActionId("action-1".to_owned()),
            browser_kind: BrowserKind::Edge,
            executable_path: None,
            profile: None,
            urls: vec!["https://example.com".to_owned()],
            placement: sample_placement(),
            startup_timeout_ms: 10_000,
        });
        assert!(layout.validate(true).is_err());
    }

    #[test]
    fn accepts_a_complete_layout() {
        assert!(sample_layout().validate(true).is_ok());
    }
}
