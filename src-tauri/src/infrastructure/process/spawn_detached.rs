use std::process::{Command, Stdio};

use crate::domain::ports::{LaunchedProcess, ProcessLaunchError, ProcessLaunchRequest};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
#[cfg(target_os = "windows")]
const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;

pub fn spawn_detached(request: ProcessLaunchRequest) -> Result<LaunchedProcess, ProcessLaunchError> {
    if !std::path::Path::new(&request.executable_path).is_file() {
        return Err(ProcessLaunchError::ExecutableNotFound);
    }

    match spawn_command(&request, true) {
        Ok(process_id) => Ok(LaunchedProcess { process_id }),
        Err(ProcessLaunchError::LaunchFailed(_)) => spawn_command(&request, false).map(|process_id| {
            LaunchedProcess { process_id }
        }),
        Err(error) => Err(error),
    }
}

fn spawn_command(
    request: &ProcessLaunchRequest,
    breakaway_from_job: bool,
) -> Result<u32, ProcessLaunchError> {
    let mut command = Command::new(&request.executable_path);
    command.args(&request.arguments);
    if let Some(directory) = &request.working_directory {
        command.current_dir(directory);
    }
    configure_detached(&mut command, breakaway_from_job);

    let child = command.spawn().map_err(map_spawn_error)?;
    let process_id = child.id();
    drop(child);
    Ok(process_id)
}

fn configure_detached(command: &mut Command, breakaway_from_job: bool) {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        let mut flags = CREATE_NEW_PROCESS_GROUP;
        if breakaway_from_job {
            flags |= CREATE_BREAKAWAY_FROM_JOB;
        }
        command.creation_flags(flags);
    }
}

fn map_spawn_error(error: std::io::Error) -> ProcessLaunchError {
    if error.kind() == std::io::ErrorKind::NotFound {
        ProcessLaunchError::ExecutableNotFound
    } else {
        ProcessLaunchError::LaunchFailed(error.to_string())
    }
}
