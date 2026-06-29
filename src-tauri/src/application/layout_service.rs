use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use uuid::Uuid;

use crate::{
    domain::{
        layout::{
            Layout, LayoutAction, LayoutActionId, LayoutId, LayoutSummary, validate_executable_path,
        },
        ports::LayoutRepository,
    },
    error::AppError,
    infrastructure::persistence::SqliteLayoutRepository,
};

pub struct LayoutService<'a> {
    repository: SqliteLayoutRepository<'a>,
}

impl<'a> LayoutService<'a> {
    #[must_use]
    pub fn new(repository: SqliteLayoutRepository<'a>) -> Self {
        Self { repository }
    }

    pub fn list_summaries(&self) -> Result<Vec<LayoutSummary>, AppError> {
        self.repository.list_summaries()
    }

    pub fn get(&self, id: &LayoutId) -> Result<Layout, AppError> {
        self.repository.get(id)
    }

    pub fn save(&self, mut layout: Layout) -> Result<Layout, AppError> {
        if layout.id.0.trim().is_empty() {
            layout.id = LayoutId(Uuid::new_v4().to_string());
            layout.created_at = current_timestamp();
        }
        layout.updated_at = current_timestamp();
        layout = layout.normalized()?;
        layout.validate(true)?;
        self.repository.save(&layout)?;
        Ok(layout)
    }

    pub fn duplicate(&self, id: &LayoutId) -> Result<Layout, AppError> {
        let source = self.repository.get(id)?;
        let mut duplicate = source;
        duplicate.id = LayoutId(Uuid::new_v4().to_string());
        duplicate.name = unique_duplicate_name(&duplicate.name, |name| {
            self.repository
                .list_summaries()
                .map(|summaries| summaries.iter().any(|summary| summary.name == name))
                .unwrap_or(false)
        });
        duplicate.actions = duplicate
            .actions
            .into_iter()
            .map(regenerate_action_id)
            .collect();
        duplicate.created_at = current_timestamp();
        duplicate.updated_at = duplicate.created_at;
        self.repository.save(&duplicate)?;
        Ok(duplicate)
    }

    pub fn delete(&self, id: &LayoutId) -> Result<(), AppError> {
        self.repository.delete(id)
    }

    pub fn validate_executable(&self, path: &str) -> Result<String, AppError> {
        validate_executable_path(path)?;
        let path = Path::new(path.trim());
        if !path.exists() {
            return Err(AppError::Validation(
                "Le fichier d’application est introuvable.".to_owned(),
            ));
        }
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_owned())
            .ok_or_else(|| {
                AppError::Validation("Choisissez un fichier d’application valide.".to_owned())
            })
    }
}

fn regenerate_action_id(action: LayoutAction) -> LayoutAction {
    let id = LayoutActionId(Uuid::new_v4().to_string());
    match action {
        LayoutAction::LaunchApplication {
            executable_path,
            arguments,
            working_directory,
            reuse_existing_window,
            window_matcher,
            placement,
            startup_timeout_ms,
            ..
        } => LayoutAction::LaunchApplication {
            id,
            executable_path,
            arguments,
            working_directory,
            reuse_existing_window,
            window_matcher,
            placement,
            startup_timeout_ms,
        },
        LayoutAction::PlaceExistingWindow {
            window_matcher,
            placement,
            ..
        } => LayoutAction::PlaceExistingWindow {
            id,
            window_matcher,
            placement,
        },
        LayoutAction::OpenBrowserWindow {
            browser_kind,
            executable_path,
            profile,
            urls,
            placement,
            startup_timeout_ms,
            ..
        } => LayoutAction::OpenBrowserWindow {
            id,
            browser_kind,
            executable_path,
            profile,
            urls,
            placement,
            startup_timeout_ms,
        },
    }
}

fn unique_duplicate_name(source_name: &str, exists: impl Fn(&str) -> bool) -> String {
    let base = format!("{source_name} (copie)");
    if !exists(&base) {
        return base;
    }
    let mut index = 2;
    loop {
        let candidate = format!("{source_name} (copie {index})");
        if !exists(&candidate) {
            return candidate;
        }
        index += 1;
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::LayoutService;
    use crate::{
        domain::{
            layout::{
                Layout, LayoutAction, LayoutActionId, LayoutId, LayoutOptions, WindowPlacement,
            },
            monitor::{MonitorFallback, MonitorId, MonitorSelector},
            window::WindowMatcher,
        },
        infrastructure::persistence::{SqliteLayoutRepository, open_in_memory_for_tests},
    };
    use uuid::Uuid;

    fn service() -> LayoutService<'static> {
        let database = Box::leak(Box::new(open_in_memory_for_tests()));
        LayoutService::new(SqliteLayoutRepository::new(database))
    }

    fn minimal_layout() -> Layout {
        Layout {
            id: LayoutId(String::new()),
            name: "Travail".to_owned(),
            description: None,
            actions: vec![LayoutAction::PlaceExistingWindow {
                id: LayoutActionId(Uuid::new_v4().to_string()),
                window_matcher: WindowMatcher {
                    process_name: Some("notepad.exe".to_owned()),
                    ..Default::default()
                },
                placement: WindowPlacement {
                    monitor_selector: MonitorSelector {
                        preferred_id: MonitorId("primary".to_owned()),
                        fallback: MonitorFallback::Primary,
                    },
                    bounds: crate::domain::geometry::NormalizedBounds::new(0.0, 0.0, 1.0, 1.0)
                        .expect("bounds"),
                    state: crate::domain::window::WindowState::Normal,
                },
            }],
            options: LayoutOptions::default(),
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn saves_duplicates_and_deletes_layouts() {
        let service = service();
        let saved = service.save(minimal_layout()).expect("save");
        let duplicate = service.duplicate(&saved.id).expect("duplicate");
        assert_eq!(duplicate.name, "Travail (copie)");
        assert_eq!(service.list_summaries().expect("list").len(), 2);
        service.delete(&duplicate.id).expect("delete");
        assert_eq!(service.list_summaries().expect("list").len(), 1);
    }
}
