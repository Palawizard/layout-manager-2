use crate::domain::layout::BrowserKind;

use crate::domain::ports::{
    LaunchedProcess, ProcessLaunchError, ProcessLaunchRequest, ProcessLauncher,
};

use super::arguments::build_browser_arguments;
use super::detection::infer_browser_kind_from_executable;
use crate::infrastructure::process::spawn_detached;

#[derive(Debug, Default)]
pub struct WindowsBrowserLauncher;

impl WindowsBrowserLauncher {
    pub fn launch_browser(
        &self,
        kind: BrowserKind,
        executable_path: &str,
        urls: &[String],
        profile: Option<&str>,
    ) -> Result<LaunchedProcess, ProcessLaunchError> {
        if urls.is_empty() {
            return Err(ProcessLaunchError::LaunchFailed("missing url".to_owned()));
        }

        let arguments = if kind == BrowserKind::SystemDefault {
            let inferred = infer_browser_kind_from_executable(executable_path);
            build_browser_arguments(inferred, urls, profile)
        } else {
            build_browser_arguments(kind, urls, profile)
        };

        tracing::debug!(
            browser = ?kind,
            url_count = urls.len(),
            sample_url = %crate::logging::sanitize_for_log(
                urls.first().map(String::as_str).unwrap_or("")
            ),
            "launching browser window"
        );

        ProcessLauncher::launch(
            self,
            ProcessLaunchRequest {
                executable_path: executable_path.to_owned(),
                arguments,
                working_directory: crate::infrastructure::process::launch_working_directory(
                    executable_path,
                ),
            },
        )
    }
}

impl ProcessLauncher for WindowsBrowserLauncher {
    fn launch(&self, request: ProcessLaunchRequest) -> Result<LaunchedProcess, ProcessLaunchError> {
        spawn_detached(request)
    }
}

#[cfg(test)]
mod tests {
    use super::WindowsBrowserLauncher;
    use crate::{
        domain::layout::BrowserKind, infrastructure::browser::arguments::build_browser_arguments,
    };

    #[test]
    fn default_browser_adapter_only_passes_urls() {
        assert_eq!(
            build_browser_arguments(
                BrowserKind::SystemDefault,
                &["https://example.com".to_owned()],
                None,
            ),
            vec!["https://example.com".to_owned()]
        );
    }

    #[test]
    fn launcher_struct_is_constructible() {
        let _launcher = WindowsBrowserLauncher;
    }
}
