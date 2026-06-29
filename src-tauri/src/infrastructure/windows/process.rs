use std::path::Path;

use windows::{
    Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        },
    },
    core::PWSTR,
};

pub(super) fn process_metadata(process_id: u32) -> (Option<String>, Option<String>) {
    // SAFETY: The access mask is read-only and no handle inheritance is requested.
    let Ok(handle) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) })
    else {
        return (None, None);
    };
    let mut buffer = vec![0u16; 32_768];
    let mut length = buffer.len() as u32;
    // SAFETY: The buffer and length pointer remain valid for the duration of the call.
    let result = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &raw mut length,
        )
    };
    // SAFETY: `handle` was returned by OpenProcess and is closed exactly once here.
    let _ = unsafe { CloseHandle(handle) };
    if result.is_err() {
        return (None, None);
    }
    let path = String::from_utf16_lossy(&buffer[..length as usize]).replace('/', "\\");
    let process_name = Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    (Some(path), process_name)
}

#[cfg(test)]
mod tests {
    use super::process_metadata;

    #[test]
    fn resolves_the_current_test_process() {
        let (path, name) = process_metadata(std::process::id());
        assert!(path.is_some());
        assert!(name.is_some());
    }
}
