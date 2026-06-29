use std::path::Path;

use crate::domain::window::WindowMatcher;

/// The launched executable targets a different process than the window matcher
/// (for example launching `steam.exe` while matching `steamwebhelper.exe`).
#[must_use]
pub fn is_indirect_launch(launched_executable: &str, matcher: &WindowMatcher) -> bool {
    let Some(target) = matcher
        .process_name
        .as_deref()
        .map(normalize_process_stem)
    else {
        return false;
    };
    let launcher = normalize_process_stem(
        Path::new(launched_executable.trim())
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(launched_executable),
    );
    !launcher.is_empty() && launcher != target
}

fn normalize_process_stem(name: &str) -> String {
    name.trim()
        .strip_suffix(".exe")
        .or_else(|| name.strip_suffix(".EXE"))
        .unwrap_or(name.trim())
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::is_indirect_launch;
    use crate::domain::window::WindowMatcher;

    #[test]
    fn detects_launcher_helper_mismatch() {
        let matcher = WindowMatcher {
            process_name: Some("steamwebhelper.exe".to_owned()),
            ..Default::default()
        };
        assert!(is_indirect_launch(
            "C:\\Program Files (x86)\\Steam\\steam.exe",
            &matcher,
        ));
    }

    #[test]
    fn treats_direct_launch_as_not_indirect() {
        let matcher = WindowMatcher {
            process_name: Some("Discord.exe".to_owned()),
            ..Default::default()
        };
        assert!(!is_indirect_launch("C:\\Apps\\Discord\\Discord.exe", &matcher));
    }
}
