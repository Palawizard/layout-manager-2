use std::process::{Command, Stdio};

use crate::domain::ports::{LaunchedProcess, ProcessLaunchError, ProcessLaunchRequest};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
#[cfg(target_os = "windows")]
const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
#[cfg(target_os = "windows")]
const DETACHED_PROCESS: u32 = 0x0000_0008;

pub fn spawn_detached(request: ProcessLaunchRequest) -> Result<LaunchedProcess, ProcessLaunchError> {
    if !std::path::Path::new(&request.executable_path).is_file() {
        return Err(ProcessLaunchError::ExecutableNotFound);
    }

    match spawn_command(&request, true) {
        Ok(process_id) => Ok(LaunchedProcess { process_id }),
        Err(ProcessLaunchError::LaunchFailed(_)) => match spawn_command(&request, false) {
            Ok(process_id) => Ok(LaunchedProcess { process_id }),
            Err(ProcessLaunchError::LaunchFailed(_)) => spawn_via_shell_execute(request),
            Err(error) => Err(error),
        },
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
        let mut flags = CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS;
        if breakaway_from_job {
            flags |= CREATE_BREAKAWAY_FROM_JOB;
        }
        command.creation_flags(flags);
    }
}

#[cfg(target_os = "windows")]
fn spawn_via_shell_execute(request: ProcessLaunchRequest) -> Result<LaunchedProcess, ProcessLaunchError> {
    use windows::{
        Win32::Foundation::CloseHandle,
        Win32::System::Threading::GetProcessId,
        Win32::UI::Shell::{SEE_MASK_FLAG_NO_UI, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW},
        Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        core::PCWSTR,
    };

    let executable = wide_null_terminated(&request.executable_path);
    let parameters = wide_null_terminated(&quote_arguments(&request.arguments));
    let directory = request
        .working_directory
        .as_deref()
        .map(wide_null_terminated);
    let verb = wide_null_terminated("open");

    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_FLAG_NO_UI,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(executable.as_ptr()),
        lpParameters: if request.arguments.is_empty() {
            PCWSTR::null()
        } else {
            PCWSTR(parameters.as_ptr())
        },
        lpDirectory: directory
            .as_ref()
            .map_or(PCWSTR::null(), |value| PCWSTR(value.as_ptr())),
        nShow: SW_SHOWNORMAL.0 as i32,
        ..Default::default()
    };

    // SAFETY: All wide string buffers and the execute info struct remain valid for the call.
    // Returned process handles are closed exactly once below.
    let succeeded = unsafe { ShellExecuteExW(&mut info) }.is_ok();
    if !succeeded {
        return Err(ProcessLaunchError::LaunchFailed(
            "shell execute failed".to_owned(),
        ));
    }

    let process = info.hProcess;
    if process.is_invalid() {
        return Err(ProcessLaunchError::LaunchFailed(
            "shell execute returned no process handle".to_owned(),
        ));
    }

    // SAFETY: `process` was returned by ShellExecuteExW and is closed exactly once here.
    let process_id = unsafe { GetProcessId(process) };
    let _ = unsafe { CloseHandle(process) };
    if process_id == 0 {
        return Err(ProcessLaunchError::LaunchFailed(
            "shell execute process id unavailable".to_owned(),
        ));
    }
    let _ = process_access_check(process_id);
    Ok(LaunchedProcess { process_id })
}

#[cfg(target_os = "windows")]
fn process_access_check(process_id: u32) -> Result<(), ()> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    // SAFETY: Read-only access with a valid process id.
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }
        .map_err(|_| ())?;
    // SAFETY: `handle` was returned by OpenProcess and is closed exactly once here.
    let _ = unsafe { CloseHandle(handle) };
    Ok(())
}

#[cfg(target_os = "windows")]
fn wide_null_terminated(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

#[cfg(target_os = "windows")]
fn quote_arguments(arguments: &[String]) -> String {
    arguments
        .iter()
        .map(|argument| {
            if argument.chars().any(char::is_whitespace) {
                format!("\"{argument}\"")
            } else {
                argument.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(not(target_os = "windows"))]
fn spawn_via_shell_execute(_request: ProcessLaunchRequest) -> Result<LaunchedProcess, ProcessLaunchError> {
    Err(ProcessLaunchError::LaunchFailed(
        "shell execute is only available on Windows".to_owned(),
    ))
}

fn map_spawn_error(error: std::io::Error) -> ProcessLaunchError {
    if error.kind() == std::io::ErrorKind::NotFound {
        ProcessLaunchError::ExecutableNotFound
    } else {
        ProcessLaunchError::LaunchFailed(error.to_string())
    }
}

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::mem::size_of;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
