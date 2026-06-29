use rusqlite::{OptionalExtension, params};

use crate::{
    domain::{
        layout::{Layout, LayoutAction, LayoutId, LayoutOptions, LayoutSummary},
        ports::LayoutRepository,
    },
    error::AppError,
    infrastructure::persistence::Database,
};

pub struct SqliteLayoutRepository<'a> {
    database: &'a Database,
}

impl<'a> SqliteLayoutRepository<'a> {
    #[must_use]
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }
}

impl LayoutRepository for SqliteLayoutRepository<'_> {
    fn list_summaries(&self) -> Result<Vec<LayoutSummary>, AppError> {
        self.database.with_connection(|connection| {
            let mut statement = connection
                .prepare(
                    "SELECT l.id, l.name, l.description, l.updated_at, COUNT(a.id)
                     FROM layouts l
                     LEFT JOIN layout_actions a ON a.layout_id = l.id
                     GROUP BY l.id
                     ORDER BY l.updated_at DESC, l.name COLLATE NOCASE ASC",
                )
                .map_err(|error| AppError::Storage(error.to_string()))?;
            let summaries = statement
                .query_map([], |row| {
                    Ok(LayoutSummary {
                        id: LayoutId(row.get(0)?),
                        name: row.get(1)?,
                        description: row.get(2)?,
                        action_count: row.get::<_, i64>(4)? as usize,
                        updated_at: row.get(3)?,
                    })
                })
                .map_err(|error| AppError::Storage(error.to_string()))?
                .filter_map(Result::ok)
                .collect();
            Ok(summaries)
        })
    }

    fn get(&self, id: &LayoutId) -> Result<Layout, AppError> {
        self.database.with_connection(|connection| {
            let layout = connection
                .query_row(
                    "SELECT id, name, description, minimize_unmatched_windows, continue_on_error,
                            restore_previous_state_on_cancel, created_at, updated_at
                     FROM layouts WHERE id = ?1",
                    [id.0.as_str()],
                    |row| {
                        Ok(Layout {
                            id: LayoutId(row.get(0)?),
                            name: row.get(1)?,
                            description: row.get(2)?,
                            options: LayoutOptions {
                                minimize_unmatched_windows: row.get::<_, i64>(3)? != 0,
                                continue_on_error: row.get::<_, i64>(4)? != 0,
                                restore_previous_state_on_cancel: row.get::<_, i64>(5)? != 0,
                            },
                            actions: Vec::new(),
                            created_at: row.get(6)?,
                            updated_at: row.get(7)?,
                        })
                    },
                )
                .optional()
                .map_err(|error| AppError::Storage(error.to_string()))?
                .ok_or(AppError::NotFound)?;

            let mut actions_statement = connection
                .prepare(
                    "SELECT payload FROM layout_actions
                     WHERE layout_id = ?1
                     ORDER BY position ASC",
                )
                .map_err(|error| AppError::Storage(error.to_string()))?;
            let actions = actions_statement
                .query_map([id.0.as_str()], |row| {
                    let payload: String = row.get(0)?;
                    serde_json::from_str(&payload).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })
                })
                .map_err(|error| AppError::Storage(error.to_string()))?
                .collect::<Result<Vec<LayoutAction>, _>>()
                .map_err(|error| AppError::Storage(error.to_string()))?;

            Ok(Layout { actions, ..layout })
        })
    }

    fn save(&self, layout: &Layout) -> Result<(), AppError> {
        layout.validate(false)?;
        self.database.with_connection(|connection| {
            let transaction = connection
                .unchecked_transaction()
                .map_err(|error| AppError::Storage(error.to_string()))?;
            transaction
                .execute(
                    "INSERT INTO layouts (
                        id, name, description, minimize_unmatched_windows, continue_on_error,
                        restore_previous_state_on_cancel, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(id) DO UPDATE SET
                        name = excluded.name,
                        description = excluded.description,
                        minimize_unmatched_windows = excluded.minimize_unmatched_windows,
                        continue_on_error = excluded.continue_on_error,
                        restore_previous_state_on_cancel = excluded.restore_previous_state_on_cancel,
                        updated_at = excluded.updated_at",
                    params![
                        layout.id.0,
                        layout.name,
                        layout.description,
                        i64::from(layout.options.minimize_unmatched_windows),
                        i64::from(layout.options.continue_on_error),
                        i64::from(layout.options.restore_previous_state_on_cancel),
                        layout.created_at,
                        layout.updated_at,
                    ],
                )
                .map_err(|error| AppError::Storage(error.to_string()))?;
            transaction
                .execute(
                    "DELETE FROM layout_actions WHERE layout_id = ?1",
                    [layout.id.0.as_str()],
                )
                .map_err(|error| AppError::Storage(error.to_string()))?;
            for (position, action) in layout.actions.iter().enumerate() {
                let payload = serde_json::to_string(action)
                    .map_err(|error| AppError::Storage(error.to_string()))?;
                transaction
                    .execute(
                        "INSERT INTO layout_actions (id, layout_id, position, kind, payload)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            action.id().0,
                            layout.id.0,
                            i64::try_from(position).map_err(|_| AppError::Internal)?,
                            action_kind(action),
                            payload,
                        ],
                    )
                    .map_err(|error| AppError::Storage(error.to_string()))?;
            }
            transaction
                .commit()
                .map_err(|error| AppError::Storage(error.to_string()))?;
            Ok(())
        })
    }

    fn delete(&self, id: &LayoutId) -> Result<(), AppError> {
        self.database.with_connection(|connection| {
            let deleted = connection
                .execute("DELETE FROM layouts WHERE id = ?1", [id.0.as_str()])
                .map_err(|error| AppError::Storage(error.to_string()))?;
            if deleted == 0 {
                return Err(AppError::NotFound);
            }
            Ok(())
        })
    }
}

fn action_kind(action: &LayoutAction) -> &'static str {
    match action {
        LayoutAction::LaunchApplication { .. } => "launch_application",
        LayoutAction::PlaceExistingWindow { .. } => "place_existing_window",
        LayoutAction::OpenBrowserWindow { .. } => "open_browser_window",
    }
}

#[cfg(test)]
mod tests {
    use super::SqliteLayoutRepository;
    use crate::{
        domain::{
            geometry::NormalizedBounds,
            layout::{
                BrowserKind, Layout, LayoutAction, LayoutActionId, LayoutId, LayoutOptions,
                WindowPlacement,
            },
            monitor::{MonitorFallback, MonitorId, MonitorSelector},
            ports::LayoutRepository,
            window::WindowMatcher,
        },
        infrastructure::persistence::open_in_memory_for_tests,
    };

    fn test_repository() -> SqliteLayoutRepository<'static> {
        let database = Box::leak(Box::new(open_in_memory_for_tests()));
        SqliteLayoutRepository::new(database)
    }

    fn sample_placement() -> WindowPlacement {
        WindowPlacement {
            monitor_selector: MonitorSelector {
                preferred_id: MonitorId("primary".to_owned()),
                fallback: MonitorFallback::Primary,
            },
            bounds: NormalizedBounds::new(0.0, 0.0, 0.5, 1.0).expect("bounds"),
            state: crate::domain::window::WindowState::Normal,
            center_scale: None,
        }
    }

    fn sample_layout() -> Layout {
        Layout {
            id: LayoutId("layout-1".to_owned()),
            name: "Travail".to_owned(),
            description: Some("Bureau".to_owned()),
            actions: vec![
                LayoutAction::LaunchApplication {
                    id: LayoutActionId("action-1".to_owned()),
                    executable_path: "C:\\Windows\\System32\\notepad.exe".to_owned(),
                    arguments: vec!["notes.txt".to_owned()],
                    working_directory: None,
                    reuse_existing_window: true,
                    window_matcher: WindowMatcher {
                        process_name: Some("notepad.exe".to_owned()),
                        ..Default::default()
                    },
                    placement: sample_placement(),
                    startup_timeout_ms: 15_000,
                },
                LayoutAction::PlaceExistingWindow {
                    id: LayoutActionId("action-2".to_owned()),
                    window_matcher: WindowMatcher {
                        process_name: Some("msedge.exe".to_owned()),
                        ..Default::default()
                    },
                    placement: sample_placement(),
                    executable_path: Some("C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe".to_owned()),
                    reopen_if_absent: true,
                    startup_timeout_ms: 15_000,
                },
                LayoutAction::OpenBrowserWindow {
                    id: LayoutActionId("action-3".to_owned()),
                    browser_kind: BrowserKind::Edge,
                    executable_path: None,
                    profile: None,
                    urls: vec![
                        "https://example.com".to_owned(),
                        "https://example.org".to_owned(),
                    ],
                    placement: sample_placement(),
                    startup_timeout_ms: 20_000,
                },
            ],
            options: LayoutOptions::default(),
            created_at: 10,
            updated_at: 20,
        }
    }

    #[test]
    fn persists_and_reads_layouts_in_action_order() {
        let repository = test_repository();
        let layout = sample_layout();
        repository.save(&layout).expect("save");
        let summaries = repository.list_summaries().expect("summaries");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].action_count, 3);

        let loaded = repository.get(&layout.id).expect("load");
        assert_eq!(loaded.name, layout.name);
        assert_eq!(loaded.actions.len(), 3);
        assert!(matches!(
            loaded.actions[0],
            LayoutAction::LaunchApplication { .. }
        ));
        assert!(matches!(
            loaded.actions[2],
            LayoutAction::OpenBrowserWindow { .. }
        ));
    }

    #[test]
    fn rolls_back_invalid_saves_without_partial_data() {
        let repository = test_repository();
        let mut invalid = sample_layout();
        invalid.actions[0] = LayoutAction::LaunchApplication {
            id: LayoutActionId("broken".to_owned()),
            executable_path: "relative.exe".to_owned(),
            arguments: vec![],
            working_directory: None,
            reuse_existing_window: false,
            window_matcher: WindowMatcher::default(),
            placement: sample_placement(),
            startup_timeout_ms: 15_000,
        };
        assert!(repository.save(&invalid).is_err());
        assert!(repository.list_summaries().expect("summaries").is_empty());
    }
}
