use std::path::PathBuf;

use tauri::AppHandle;
use tauri::Manager;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const LOG_FILE_PREFIX: &str = "layout-manager";

pub(crate) fn initialize(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let log_directory = log_directory(app)?;
    std::fs::create_dir_all(&log_directory)?;

    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .max_log_files(7)
        .filename_prefix(LOG_FILE_PREFIX)
        .build(&log_directory)?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("layout_manager_2=info,warn"));

    let file_layer = fmt::layer()
        .with_target(false)
        .with_ansi(false)
        .compact()
        .with_writer(file_appender);

    let stdout_layer = fmt::layer()
        .with_target(false)
        .compact()
        .with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .try_init()?;

    Ok(())
}

pub fn log_directory(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(app.path().app_data_dir()?.join("logs"))
}

pub fn sanitize_for_log(value: &str) -> String {
    if let Some((base, _query)) = value.split_once('?')
        && value.contains("://")
    {
        return format!("{base}?…");
    }
    if value.len() > 120 {
        return format!("{}…", &value[..120]);
    }
    value.to_owned()
}

#[cfg(test)]
mod tests {
    use super::sanitize_for_log;

    #[test]
    fn redacts_url_query_strings() {
        assert_eq!(
            sanitize_for_log("https://example.com/path?token=secret"),
            "https://example.com/path?…"
        );
    }

    #[test]
    fn truncates_long_values() {
        let value = "a".repeat(200);
        let sanitized = sanitize_for_log(&value);
        assert!(sanitized.ends_with('…'));
        assert!(sanitized.len() < value.len());
    }
}
