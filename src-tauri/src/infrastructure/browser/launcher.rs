use std::process::Command;

use crate::domain::layout::BrowserKind;

use crate::domain::ports::{
    LaunchedProcess, ProcessLaunchError, ProcessLaunchRequest, ProcessLauncher,
};

use super::arguments::build_browser_arguments;

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
        let arguments = build_browser_arguments(kind, urls, profile);
        if kind == BrowserKind::SystemDefault {
            launch_default_browser(executable_path, &arguments)
        } else {
            ProcessLauncher::launch(
                self,
                ProcessLaunchRequest {
                    executable_path: executable_path.to_owned(),
                    arguments,
                    working_directory: None,
                },
            )
        }
    }
}

impl ProcessLauncher for WindowsBrowserLauncher {
    fn launch(
        &self,
        request: ProcessLaunchRequest,
    ) -> Result<LaunchedProcess, ProcessLaunchError> {
        if !std::path::Path::new(&request.executable_path).is_file() {
            return Err(ProcessLaunchError::ExecutableNotFound);
        }
        let child = Command::new(&request.executable_path)
            .args(&request.arguments)
            .spawn()
            .map_err(|error| ProcessLaunchError::LaunchFailed(error.to_string()))?;
        Ok(LaunchedProcess {
            process_id: child.id(),
        })
    }
}

fn launch_default_browser(
    executable_path: &str,
    urls: &[String],
) -> Result<LaunchedProcess, ProcessLaunchError> {
    if urls.is_empty() {
        return Err(ProcessLaunchError::LaunchFailed(
            "missing url".to_owned(),
        ));
    }
    let child = Command::new(executable_path)
        .arg(urls[0].as_str())
        .spawn()
        .map_err(|error| ProcessLaunchError::LaunchFailed(error.to_string()))?;
    for url in urls.iter().skip(1) {
        let _ = Command::new(executable_path).arg(url.as_str()).spawn();
    }
    Ok(LaunchedProcess {
        process_id: child.id(),
    })
}

#[cfg(test)]
mod tests {
    use super::WindowsBrowserLauncher;
    use crate::{
        domain::layout::BrowserKind,
        infrastructure::browser::arguments::build_browser_arguments,
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
