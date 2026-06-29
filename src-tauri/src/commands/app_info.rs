use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
    platform: &'static str,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Layout Manager 2",
        version: env!("CARGO_PKG_VERSION"),
        platform: "windows",
    }
}

#[cfg(test)]
mod tests {
    use super::get_app_info;

    #[test]
    fn returns_application_identity() {
        let info = get_app_info();

        assert_eq!(info.name, "Layout Manager 2");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.platform, "windows");
    }
}
