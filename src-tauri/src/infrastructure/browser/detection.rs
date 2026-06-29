use serde::Serialize;

use crate::domain::layout::BrowserKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledBrowser {
    pub kind: BrowserKind,
    pub executable_path: String,
    pub label: String,
}

pub fn detect_installed_browsers() -> Vec<InstalledBrowser> {
    let mut browsers = Vec::new();
    if let Some(path) = find_executable(&edge_paths()) {
        browsers.push(installed(BrowserKind::Edge, path, "Microsoft Edge"));
    }
    if let Some(path) = find_executable(&chrome_paths()) {
        browsers.push(installed(BrowserKind::Chrome, path, "Google Chrome"));
    }
    if let Some(path) = find_executable(&firefox_paths()) {
        browsers.push(installed(BrowserKind::Firefox, path, "Mozilla Firefox"));
    }
    if let Some(path) = detect_default_browser_executable() {
        browsers.push(installed(
            BrowserKind::SystemDefault,
            path,
            "Navigateur par défaut",
        ));
    }
    browsers
}

#[must_use]
pub fn infer_browser_kind_from_executable(executable_path: &str) -> BrowserKind {
    let file_name = std::path::Path::new(executable_path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    match file_name.as_str() {
        "firefox.exe" => BrowserKind::Firefox,
        "chrome.exe" => BrowserKind::Chrome,
        "msedge.exe" => BrowserKind::Edge,
        _ => BrowserKind::SystemDefault,
    }
}

pub fn resolve_browser_executable(
    kind: BrowserKind,
    override_path: Option<&str>,
) -> Option<String> {
    if let Some(path) = override_path.filter(|path| std::path::Path::new(path).is_file()) {
        return Some(path.to_owned());
    }
    detect_installed_browsers()
        .into_iter()
        .find(|browser| browser.kind == kind)
        .map(|browser| browser.executable_path)
}

fn installed(kind: BrowserKind, executable_path: String, label: &str) -> InstalledBrowser {
    InstalledBrowser {
        kind,
        executable_path,
        label: label.to_owned(),
    }
}

fn find_executable(paths: &[String]) -> Option<String> {
    paths
        .iter()
        .find(|path| std::path::Path::new(path).is_file())
        .cloned()
}

fn edge_paths() -> Vec<String> {
    vec![
        program_files_path("Microsoft\\Edge\\Application\\msedge.exe"),
        program_files_x86_path("Microsoft\\Edge\\Application\\msedge.exe"),
    ]
}

fn chrome_paths() -> Vec<String> {
    let mut paths = vec![program_files_path(
        "Google\\Chrome\\Application\\chrome.exe",
    )];
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        paths.push(
            std::path::Path::new(&local)
                .join("Google\\Chrome\\Application\\chrome.exe")
                .to_string_lossy()
                .into_owned(),
        );
    }
    paths
}

fn firefox_paths() -> Vec<String> {
    vec![
        program_files_path("Mozilla Firefox\\firefox.exe"),
        program_files_x86_path("Mozilla Firefox\\firefox.exe"),
    ]
}

fn program_files_path(relative: &str) -> String {
    std::env::var_os("ProgramFiles")
        .map(|root| {
            std::path::Path::new(&root)
                .join(relative)
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_default()
}

fn program_files_x86_path(relative: &str) -> String {
    std::env::var_os("ProgramFiles(x86)")
        .map(|root| {
            std::path::Path::new(&root)
                .join(relative)
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_default()
}

fn detect_default_browser_executable() -> Option<String> {
    let output = std::process::Command::new("cmd")
        .args(["/C", "assoc", ".htm"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let association = String::from_utf8_lossy(&output.stdout);
    let prog_id = association
        .split('=')
        .nth(1)?
        .trim()
        .trim_end_matches('\r')
        .trim_end_matches('\n');
    let ftype_output = std::process::Command::new("cmd")
        .args(["/C", "ftype", prog_id])
        .output()
        .ok()?;
    if !ftype_output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&ftype_output.stdout);
    let command = line.split('=').nth(1)?.trim();
    extract_executable_from_command(command)
}

fn extract_executable_from_command(command: &str) -> Option<String> {
    let trimmed = command.trim();
    if let Some(stripped) = trimmed.strip_prefix('"') {
        let end = stripped.find('"')?;
        return Some(stripped[..end].to_owned());
    }
    let path = trimmed.split_whitespace().next()?;
    Some(path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{detect_installed_browsers, extract_executable_from_command};

    #[test]
    fn extracts_a_quoted_executable_from_a_file_type_command() {
        assert_eq!(
            extract_executable_from_command("\"C:\\Program Files\\Browser\\browser.exe\" \"%1\""),
            Some("C:\\Program Files\\Browser\\browser.exe".to_owned())
        );
    }

    #[test]
    fn infers_browser_kind_from_executable_name() {
        use super::infer_browser_kind_from_executable;
        use crate::domain::layout::BrowserKind;

        assert_eq!(
            infer_browser_kind_from_executable("C:\\Program Files\\Mozilla Firefox\\firefox.exe"),
            BrowserKind::Firefox
        );
        assert_eq!(
            infer_browser_kind_from_executable(
                "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"
            ),
            BrowserKind::Chrome
        );
        assert_eq!(
            infer_browser_kind_from_executable("C:\\Program Files\\Unknown\\browser.exe"),
            BrowserKind::SystemDefault
        );
    }

    #[test]
    fn detection_returns_only_existing_browsers() {
        use crate::domain::layout::BrowserKind;

        let browsers = detect_installed_browsers();
        for browser in browsers {
            if browser.kind == BrowserKind::SystemDefault {
                continue;
            }
            assert!(std::path::Path::new(&browser.executable_path).is_file());
        }
    }
}
