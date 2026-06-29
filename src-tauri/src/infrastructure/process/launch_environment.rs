use std::path::{Path, PathBuf};

use crate::domain::ports::ProcessLaunchRequest;

use super::launch_executable::launch_working_directory;

/// Handles launchers that ship an `expect-updater` marker (for example Caffeinated).
///
/// - If the marker points at another existing executable, launch that binary instead.
/// - Otherwise remove the marker so the bootstrap does not recursively relaunch itself.
pub fn prepare_launch_environment(request: &mut ProcessLaunchRequest) {
    let executable = Path::new(request.executable_path.trim());
    let Some(directory) = executable.parent() else {
        return;
    };
    let expect_updater = directory.join("expect-updater");
    if !expect_updater.is_file() {
        return;
    }
    let Ok(content) = std::fs::read_to_string(&expect_updater) else {
        let _ = std::fs::remove_file(&expect_updater);
        return;
    };
    let target = PathBuf::from(content.trim().trim_matches('"'));
    if target.is_file() && !paths_equal(&target, executable) {
        request.executable_path = target.to_string_lossy().into_owned();
        request.working_directory = launch_working_directory(&request.executable_path);
        return;
    }
    let _ = std::fs::remove_file(&expect_updater);
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::prepare_launch_environment;
    use crate::domain::ports::ProcessLaunchRequest;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("layout-manager-{name}-{stamp}"))
    }

    #[test]
    fn removes_a_broken_expect_updater_marker() {
        let root = temp_dir("expect-updater");
        fs::create_dir_all(&root).expect("directory");
        let executable = root.join("Casterlabs-Caffeinated.exe");
        fs::write(&executable, "stub").expect("executable");
        fs::write(
            root.join("expect-updater"),
            "C:\\Program Files\\Missing App\\App.exe",
        )
        .expect("expect-updater");

        let mut request = ProcessLaunchRequest {
            executable_path: executable.to_string_lossy().into_owned(),
            arguments: Vec::new(),
            working_directory: None,
        };
        prepare_launch_environment(&mut request);

        assert!(!root.join("expect-updater").exists());
        assert_eq!(request.executable_path, executable.to_string_lossy());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn removes_a_self_referential_expect_updater_marker() {
        let root = temp_dir("expect-updater-self");
        fs::create_dir_all(&root).expect("directory");
        let executable = root.join("Casterlabs-Caffeinated.exe");
        fs::write(&executable, "stub").expect("executable");
        fs::write(&root.join("expect-updater"), executable.to_string_lossy().as_ref())
            .expect("expect-updater");

        let mut request = ProcessLaunchRequest {
            executable_path: executable.to_string_lossy().into_owned(),
            arguments: Vec::new(),
            working_directory: None,
        };
        prepare_launch_environment(&mut request);

        assert!(!root.join("expect-updater").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn redirects_to_a_valid_external_launcher() {
        let root = temp_dir("expect-updater-valid");
        fs::create_dir_all(&root).expect("directory");
        let executable = root.join("App.exe");
        let canonical = root.join("Canonical.exe");
        fs::write(&executable, "stub").expect("executable");
        fs::write(&canonical, "canonical").expect("canonical");
        fs::write(
            root.join("expect-updater"),
            canonical.to_string_lossy().as_ref(),
        )
        .expect("expect");

        let mut request = ProcessLaunchRequest {
            executable_path: executable.to_string_lossy().into_owned(),
            arguments: Vec::new(),
            working_directory: None,
        };
        prepare_launch_environment(&mut request);

        assert_eq!(request.executable_path, canonical.to_string_lossy());
        assert!(root.join("expect-updater").exists());
        let _ = fs::remove_dir_all(root);
    }
}
