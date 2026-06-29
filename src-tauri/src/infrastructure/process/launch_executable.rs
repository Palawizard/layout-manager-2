use std::path::{Path, PathBuf};

const HELPER_STEM_SUFFIXES: &[&str] = &[
    "webhelper",
    "helper",
    "renderer",
    "crashhandler",
    "crashpad",
    "gpu",
    "broker",
    "utility",
    "service",
    "console",
];

const HELPER_STEM_MARKERS: &[&str] = &[
    "webhelper",
    "helper",
    "renderer",
    "crashpad",
    "gpu",
    "broker",
    "utility",
    "service",
    "console",
];

const MAX_DIRECTORY_WALK_DEPTH: usize = 8;

/// Resolves the executable that should be spawned to (re)open an application from a
/// window's process path. Helper and child binaries are mapped to their main launcher
/// when one can be found in the install tree.
#[must_use]
pub fn resolve_launch_executable(executable_path: &str) -> String {
    let path = Path::new(executable_path.trim());
    if !is_windows_executable(path) {
        return executable_path.to_owned();
    }

    resolve_launcher(path).unwrap_or_else(|| executable_path.to_owned())
}

/// Recovers a launch executable from a stored path, including legacy values that
/// incorrectly pointed at resource files (.pak, .dll, …) instead of the process binary.
#[must_use]
pub fn recover_launch_executable(stored_path: &str, process_name: Option<&str>) -> String {
    let trimmed = stored_path.trim();
    if is_windows_executable(Path::new(trimmed)) {
        return resolve_launch_executable(trimmed);
    }

    let Some(process_name) = process_name.filter(|name| {
        Path::new(name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
    }) else {
        return trimmed.to_owned();
    };

    let bad_path = Path::new(trimmed);
    let Some(directory) = bad_path.parent() else {
        return trimmed.to_owned();
    };

    let candidate = directory.join(process_name);
    if candidate.is_file() {
        return resolve_launch_executable(&candidate.to_string_lossy());
    }

    trimmed.to_owned()
}

#[must_use]
pub fn is_windows_executable(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
}

#[must_use]
pub fn launch_working_directory(executable_path: &str) -> Option<String> {
    let path = Path::new(executable_path.trim());
    if !is_windows_executable(path) {
        return None;
    }
    path.parent()
        .map(|directory| directory.to_string_lossy().into_owned())
}

fn resolve_launcher(helper_executable: &Path) -> Option<String> {
    if !is_helper_like_executable(helper_executable) {
        return None;
    }

    let mut directory = helper_executable.parent()?;
    for _ in 0..MAX_DIRECTORY_WALK_DEPTH {
        if let Some(candidate) = find_launcher_in_directory(helper_executable, directory) {
            return Some(candidate.to_string_lossy().into_owned());
        }
        directory = directory.parent()?;
    }
    None
}

fn find_launcher_in_directory(source_executable: &Path, directory: &Path) -> Option<PathBuf> {
    if let Some(candidate) = resolve_from_helper_suffix_in_directory(source_executable, directory) {
        return Some(candidate);
    }
    if let Some(candidate) = resolve_from_folder_name_in_directory(source_executable, directory) {
        return Some(candidate);
    }
    resolve_from_electron_layout_in_directory(source_executable, directory)
}

fn resolve_from_helper_suffix_in_directory(
    source_executable: &Path,
    directory: &Path,
) -> Option<PathBuf> {
    let stem = source_executable.file_stem()?.to_str()?.to_lowercase();
    for suffix in HELPER_STEM_SUFFIXES {
        let Some(base) = stem.strip_suffix(suffix) else {
            continue;
        };
        if base.len() < 2 {
            continue;
        }
        if let Some(candidate) = find_exe_with_stem(directory, base)
            && candidate != source_executable
        {
            return Some(candidate);
        }
    }
    None
}

fn resolve_from_folder_name_in_directory(
    source_executable: &Path,
    directory: &Path,
) -> Option<PathBuf> {
    let folder_key = directory
        .file_name()
        .and_then(|name| name.to_str())
        .map(normalize_key)?;

    for entry in std::fs::read_dir(directory).ok()?.flatten() {
        let candidate = entry.path();
        if !is_windows_executable(&candidate) || candidate == source_executable {
            continue;
        }
        let candidate_stem = candidate
            .file_stem()
            .and_then(|name| name.to_str())
            .map(normalize_key)?;
        if is_helper_like_stem(&candidate_stem) {
            continue;
        }
        if folder_key == candidate_stem
            || folder_key.contains(&candidate_stem)
            || candidate_stem.contains(&folder_key)
        {
            return Some(candidate);
        }
    }

    None
}

fn resolve_from_electron_layout_in_directory(
    source_executable: &Path,
    directory: &Path,
) -> Option<PathBuf> {
    let directory_name = directory.file_name()?.to_str()?;
    if !directory_name.starts_with("app-") {
        return None;
    }
    let app_root = directory.parent()?;
    let app_name = app_root.file_name()?.to_str()?;
    let candidate = app_root.join(format!("{app_name}.exe"));
    if is_windows_executable(&candidate) && candidate != source_executable {
        return Some(candidate);
    }
    None
}

fn find_exe_with_stem(directory: &Path, stem: &str) -> Option<PathBuf> {
    let target = stem.to_lowercase();
    for entry in std::fs::read_dir(directory).ok()?.flatten() {
        let path = entry.path();
        if !is_windows_executable(&path) {
            continue;
        }
        let candidate_stem = path.file_stem()?.to_str()?;
        if candidate_stem.eq_ignore_ascii_case(&target) {
            return Some(path);
        }
    }
    None
}

fn is_helper_like_executable(path: &Path) -> bool {
    path.file_stem()
        .and_then(|name| name.to_str())
        .is_some_and(|stem| is_helper_like_stem(&stem.to_lowercase()))
}

fn is_helper_like_stem(stem: &str) -> bool {
    HELPER_STEM_MARKERS
        .iter()
        .any(|marker| stem.contains(marker))
}

fn normalize_key(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        is_windows_executable, launch_working_directory, recover_launch_executable,
        resolve_launch_executable,
    };
    use std::fs;
    use std::path::{Path, PathBuf};

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "layout-manager-launch-test-{}",
            uuid::Uuid::new_v4()
        ))
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write file");
    }

    fn temp_install(files: &[(&str, &str)]) -> PathBuf {
        temp_install_in("App", files)
    }

    fn temp_install_in(folder_name: &str, files: &[(&str, &str)]) -> PathBuf {
        let root = temp_root();
        let _ = fs::remove_dir_all(&root);
        let install = root.join(folder_name);
        for (name, contents) in files {
            write_file(&install.join(name), contents);
        }
        install
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn maps_webhelper_binaries_to_their_main_launcher() {
        let root = temp_root();
        let install = root.join("Steam");
        write_file(&install.join("steam.exe"), "main");
        write_file(&install.join("steamwebhelper.exe"), "helper");
        let helper = install.join("steamwebhelper.exe");
        assert_eq!(
            resolve_launch_executable(&helper.to_string_lossy()),
            install.join("steam.exe").to_string_lossy()
        );
        cleanup(&root);
    }

    #[test]
    fn walks_up_the_install_tree_for_nested_helpers() {
        let root = temp_root();
        let install = root.join("Steam");
        write_file(&install.join("steam.exe"), "main");
        write_file(
            &install
                .join("bin")
                .join("cef")
                .join("cef.win64")
                .join("steamwebhelper.exe"),
            "helper",
        );
        write_file(
            &install
                .join("bin")
                .join("cef")
                .join("cef.win64")
                .join("chrome_100_percent.pak"),
            "pak",
        );
        let helper = install
            .join("bin")
            .join("cef")
            .join("cef.win64")
            .join("steamwebhelper.exe");
        assert_eq!(
            resolve_launch_executable(&helper.to_string_lossy()),
            install.join("steam.exe").to_string_lossy()
        );
        cleanup(&root);
    }

    #[test]
    fn maps_console_binaries_to_the_install_folder_launcher() {
        let root = temp_root();
        let install = root.join("VIVE Hub");
        write_file(&install.join("VIVEHub.exe"), "main");
        write_file(&install.join("VHConsole").join("VHConsole.exe"), "console");
        write_file(&install.join("VHConsole").join("ApkParser.dll"), "dll");
        let console = install.join("VHConsole").join("VHConsole.exe");
        assert_eq!(
            resolve_launch_executable(&console.to_string_lossy()),
            install.join("VIVEHub.exe").to_string_lossy()
        );
        cleanup(&root);
    }

    #[test]
    fn ignores_non_executable_resources_in_the_same_folder() {
        let install = temp_install_in(
            "cef.win64",
            &[
                ("steamwebhelper.exe", "helper"),
                ("chrome_100_percent.pak", "pak"),
            ],
        );
        assert_eq!(
            resolve_launch_executable(&install.join("steamwebhelper.exe").to_string_lossy()),
            install.join("steamwebhelper.exe").to_string_lossy()
        );
        cleanup(install.parent().expect("parent"));
    }

    #[test]
    fn keeps_direct_launchers_unchanged() {
        let install = temp_install(&[("Discord.exe", "main")]);
        let launcher = install.join("Discord.exe");
        assert_eq!(
            resolve_launch_executable(&launcher.to_string_lossy()),
            launcher.to_string_lossy()
        );
        cleanup(install.parent().expect("parent"));
    }

    #[test]
    fn only_treats_exe_files_as_launchable() {
        assert!(!is_windows_executable(Path::new(
            "C:\\Apps\\Steam\\chrome_100_percent.pak"
        )));
        assert!(is_windows_executable(Path::new(
            "C:\\Apps\\Steam\\steam.exe"
        )));
    }

    #[test]
    fn derives_working_directory_from_executable_parent() {
        assert_eq!(
            launch_working_directory("C:\\Apps\\Steam\\steam.exe"),
            Some("C:\\Apps\\Steam".to_owned())
        );
        assert_eq!(launch_working_directory("C:\\Apps\\Steam\\file.pak"), None);
    }

    #[test]
    fn recovers_launcher_from_legacy_resource_paths() {
        let root = temp_root();
        let install = root.join("Steam");
        write_file(&install.join("steam.exe"), "main");
        let helper_dir = install.join("bin").join("cef").join("cef.win64");
        write_file(&helper_dir.join("steamwebhelper.exe"), "helper");
        write_file(&helper_dir.join("chrome_100_percent.pak"), "pak");
        assert_eq!(
            recover_launch_executable(
                &helper_dir.join("chrome_100_percent.pak").to_string_lossy(),
                Some("steamwebhelper.exe"),
            ),
            install.join("steam.exe").to_string_lossy()
        );

        let vive = root.join("VIVE Hub");
        write_file(&vive.join("VIVEHub.exe"), "main");
        let console = vive.join("VHConsole");
        write_file(&console.join("VHConsole.exe"), "console");
        write_file(&console.join("ApkParser.dll"), "dll");
        assert_eq!(
            recover_launch_executable(
                &console.join("ApkParser.dll").to_string_lossy(),
                Some("VHConsole.exe"),
            ),
            vive.join("VIVEHub.exe").to_string_lossy()
        );
        cleanup(&root);
    }
}
