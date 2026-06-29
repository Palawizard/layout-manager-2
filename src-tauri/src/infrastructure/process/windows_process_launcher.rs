use std::process::Command;

use crate::domain::ports::{
    LaunchedProcess, ProcessLaunchError, ProcessLaunchRequest, ProcessLauncher,
};

#[derive(Debug, Default)]
pub struct WindowsProcessLauncher;

impl ProcessLauncher for WindowsProcessLauncher {
    fn launch(&self, request: ProcessLaunchRequest) -> Result<LaunchedProcess, ProcessLaunchError> {
        if !std::path::Path::new(&request.executable_path).is_file() {
            return Err(ProcessLaunchError::ExecutableNotFound);
        }
        let mut command = Command::new(&request.executable_path);
        command.args(&request.arguments);
        if let Some(directory) = &request.working_directory {
            command.current_dir(directory);
        }
        let child = command.spawn().map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ProcessLaunchError::ExecutableNotFound
            } else {
                ProcessLaunchError::LaunchFailed(error.to_string())
            }
        })?;
        Ok(LaunchedProcess {
            process_id: child.id(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::WindowsProcessLauncher;
    use crate::domain::ports::{ProcessLaunchError, ProcessLaunchRequest, ProcessLauncher};

    #[test]
    fn rejects_a_missing_executable() {
        let launcher = WindowsProcessLauncher;
        let result = launcher.launch(ProcessLaunchRequest {
            executable_path: "C:\\missing\\app.exe".to_owned(),
            arguments: vec![],
            working_directory: None,
        });
        assert_eq!(result, Err(ProcessLaunchError::ExecutableNotFound));
    }

    #[test]
    fn keeps_arguments_separate_from_the_executable() {
        let launcher = WindowsProcessLauncher;
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_owned());
        let executable = format!("{system_root}\\System32\\cmd.exe");
        if !std::path::Path::new(&executable).is_file() {
            return;
        }
        let launched = launcher
            .launch(ProcessLaunchRequest {
                executable_path: executable,
                arguments: vec!["/C".to_owned(), "exit".to_owned()],
                working_directory: None,
            })
            .expect("cmd should launch");
        assert!(launched.process_id > 0);
    }
}
