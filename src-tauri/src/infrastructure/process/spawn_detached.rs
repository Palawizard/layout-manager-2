#![allow(unsafe_code)]

use std::ffi::OsStr;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

use crate::domain::ports::{LaunchedProcess, ProcessLaunchError, ProcessLaunchRequest};

use super::launch_environment::prepare_launch_environment;
use super::launch_executable::launch_working_directory;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{
    CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW,
    CREATE_UNICODE_ENVIRONMENT, CreateProcessW, DETACHED_PROCESS, GetProcessId,
    PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTF_USESTDHANDLES, STARTUPINFOW,
};
#[cfg(target_os = "windows")]
use windows::core::PWSTR;

#[cfg(target_os = "windows")]
const DETACHED_CREATION_FLAGS: PROCESS_CREATION_FLAGS = PROCESS_CREATION_FLAGS(
    CREATE_UNICODE_ENVIRONMENT.0
        | CREATE_NEW_PROCESS_GROUP.0
        | DETACHED_PROCESS.0
        | CREATE_NO_WINDOW.0,
);

pub fn spawn_detached(
    request: ProcessLaunchRequest,
) -> Result<LaunchedProcess, ProcessLaunchError> {
    let request = normalize_launch_request(request);
    if !std::path::Path::new(&request.executable_path).is_file() {
        return Err(ProcessLaunchError::ExecutableNotFound);
    }

    #[cfg(target_os = "windows")]
    {
        match spawn_create_process(&request, true) {
            Ok(process_id) => return Ok(LaunchedProcess { process_id }),
            Err(ProcessLaunchError::LaunchFailed(_)) => {}
            Err(error) => return Err(error),
        }
        match spawn_create_process(&request, false) {
            Ok(process_id) => return Ok(LaunchedProcess { process_id }),
            Err(ProcessLaunchError::LaunchFailed(_)) => {}
            Err(error) => return Err(error),
        }
        if let Ok(process_id) = spawn_via_shell_execute(&request) {
            return Ok(LaunchedProcess { process_id });
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = request;
    }

    Err(ProcessLaunchError::LaunchFailed(
        "process launch failed".to_owned(),
    ))
}

fn normalize_launch_request(mut request: ProcessLaunchRequest) -> ProcessLaunchRequest {
    if request.working_directory.is_none() {
        request.working_directory = launch_working_directory(&request.executable_path);
    }
    prepare_launch_environment(&mut request);
    request
}

#[cfg(target_os = "windows")]
fn spawn_create_process(
    request: &ProcessLaunchRequest,
    breakaway_from_job: bool,
) -> Result<u32, ProcessLaunchError> {
    let mut application = wide_null_terminated(&request.executable_path);
    let mut command_line = wide_null_terminated(&build_command_line(
        &request.executable_path,
        &request.arguments,
    ));
    let mut current_directory = request
        .working_directory
        .as_deref()
        .map(wide_null_terminated);

    let startup_info = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        dwFlags: STARTF_USESTDHANDLES,
        hStdInput: HANDLE(null_mut()),
        hStdOutput: HANDLE(null_mut()),
        hStdError: HANDLE(null_mut()),
        ..Default::default()
    };
    let mut process_info = PROCESS_INFORMATION::default();
    let mut flags = DETACHED_CREATION_FLAGS;
    if breakaway_from_job {
        flags = PROCESS_CREATION_FLAGS(flags.0 | CREATE_BREAKAWAY_FROM_JOB.0);
    }

    // SAFETY: Wide buffers and PROCESS_INFORMATION remain valid for the duration of the call.
    // Returned process and thread handles are closed exactly once below.
    let succeeded = unsafe {
        CreateProcessW(
            PWSTR(application.as_mut_ptr()),
            Some(PWSTR(command_line.as_mut_ptr())),
            None,
            None,
            false,
            flags,
            None,
            current_directory
                .as_mut()
                .map_or(PWSTR::null(), |value| PWSTR(value.as_mut_ptr())),
            &startup_info,
            &mut process_info,
        )
    }
    .is_ok();

    if !succeeded {
        return Err(ProcessLaunchError::LaunchFailed(
            "create process failed".to_owned(),
        ));
    }

    // SAFETY: Handles were returned by CreateProcessW and are closed exactly once here.
    let process_id = unsafe { GetProcessId(process_info.hProcess) };
    let _ = unsafe { CloseHandle(process_info.hThread) };
    let _ = unsafe { CloseHandle(process_info.hProcess) };
    if process_id == 0 {
        return Err(ProcessLaunchError::LaunchFailed(
            "create process returned no process id".to_owned(),
        ));
    }
    Ok(process_id)
}

#[cfg(target_os = "windows")]
fn spawn_via_shell_execute(request: &ProcessLaunchRequest) -> Result<u32, ProcessLaunchError> {
    use windows::{
        Win32::UI::Shell::{
            SEE_MASK_FLAG_NO_UI, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
        },
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
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    // SAFETY: All wide string buffers and the execute info struct remain valid for the call.
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
    Ok(process_id)
}

#[cfg(target_os = "windows")]
fn build_command_line(executable: &str, arguments: &[String]) -> String {
    let mut command_line = quote_argument(executable);
    for argument in arguments {
        command_line.push(' ');
        command_line.push_str(&quote_argument(argument));
    }
    command_line
}

#[cfg(target_os = "windows")]
fn quote_argument(argument: &str) -> String {
    if argument.is_empty() {
        return "\"\"".to_owned();
    }
    if !argument.chars().any(char::is_whitespace) && !argument.contains('"') {
        return argument.to_owned();
    }
    let mut quoted = String::from("\"");
    for character in argument.chars() {
        if character == '"' {
            quoted.push('\\');
        }
        quoted.push(character);
    }
    quoted.push('"');
    quoted
}

#[cfg(target_os = "windows")]
fn quote_arguments(arguments: &[String]) -> String {
    arguments
        .iter()
        .map(|argument| quote_argument(argument))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(target_os = "windows")]
fn wide_null_terminated(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}
