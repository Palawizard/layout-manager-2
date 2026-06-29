use tauri::State;

use crate::{
    application::layout_service::LayoutService,
    domain::layout::{Layout, LayoutId, LayoutSummary},
    error::PublicError,
    infrastructure::persistence::{Database, SqliteLayoutRepository},
};

#[tauri::command]
pub fn list_layouts(database: State<'_, Database>) -> Result<Vec<LayoutSummary>, PublicError> {
    LayoutService::new(SqliteLayoutRepository::new(&database))
        .list_summaries()
        .map_err(PublicError::from)
}

#[tauri::command]
pub fn get_layout(database: State<'_, Database>, layout_id: String) -> Result<Layout, PublicError> {
    LayoutService::new(SqliteLayoutRepository::new(&database))
        .get(&LayoutId(layout_id))
        .map_err(PublicError::from)
}

#[tauri::command]
pub fn save_layout(database: State<'_, Database>, layout: Layout) -> Result<Layout, PublicError> {
    LayoutService::new(SqliteLayoutRepository::new(&database))
        .save(layout)
        .map_err(PublicError::from)
}

#[tauri::command]
pub fn duplicate_layout(
    database: State<'_, Database>,
    layout_id: String,
) -> Result<Layout, PublicError> {
    LayoutService::new(SqliteLayoutRepository::new(&database))
        .duplicate(&LayoutId(layout_id))
        .map_err(PublicError::from)
}

#[tauri::command]
pub fn delete_layout(database: State<'_, Database>, layout_id: String) -> Result<(), PublicError> {
    LayoutService::new(SqliteLayoutRepository::new(&database))
        .delete(&LayoutId(layout_id))
        .map_err(PublicError::from)
}

#[tauri::command]
pub fn validate_executable(
    database: State<'_, Database>,
    path: String,
) -> Result<String, PublicError> {
    LayoutService::new(SqliteLayoutRepository::new(&database))
        .validate_executable(&path)
        .map_err(PublicError::from)
}

#[cfg(test)]
mod tests {
    use crate::{
        application::layout_service::LayoutService,
        domain::{
            layout::{
                Layout, LayoutAction, LayoutActionId, LayoutId, LayoutOptions, WindowPlacement,
            },
            monitor::{MonitorFallback, MonitorId, MonitorSelector},
            window::WindowMatcher,
        },
        infrastructure::persistence::{SqliteLayoutRepository, open_in_memory_for_tests},
    };

    #[test]
    fn exposes_layout_management_without_storage_schema_details() {
        let database = open_in_memory_for_tests();
        let service = LayoutService::new(SqliteLayoutRepository::new(&database));
        let layout = Layout {
            id: LayoutId(String::new()),
            name: "Bureau".to_owned(),
            description: None,
            actions: vec![LayoutAction::PlaceExistingWindow {
                id: LayoutActionId("action-1".to_owned()),
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
                    center_scale: None,
                },
            }],
            options: LayoutOptions::default(),
            created_at: 0,
            updated_at: 0,
        };
        let saved = service.save(layout).expect("save");
        let summaries = service.list_summaries().expect("list");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, saved.id);
    }
}
