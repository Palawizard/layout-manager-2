use crate::domain::layout::BrowserKind;

pub fn build_browser_arguments(
    kind: BrowserKind,
    urls: &[String],
    profile: Option<&str>,
) -> Vec<String> {
    match kind {
        BrowserKind::Edge | BrowserKind::Chrome => build_chromium_arguments(urls, profile),
        BrowserKind::Firefox => build_firefox_arguments(urls, profile),
        BrowserKind::SystemDefault => urls.to_vec(),
    }
}

fn build_chromium_arguments(urls: &[String], profile: Option<&str>) -> Vec<String> {
    let mut arguments = vec!["--new-window".to_owned()];
    if let Some(profile_name) = profile.filter(|value| !value.trim().is_empty()) {
        arguments.push(format!("--profile-directory={profile_name}"));
    }
    arguments.extend(urls.iter().cloned());
    arguments
}

fn build_firefox_arguments(urls: &[String], profile: Option<&str>) -> Vec<String> {
    let mut arguments = Vec::new();
    if let Some(profile_name) = profile.filter(|value| !value.trim().is_empty()) {
        arguments.push("-P".to_owned());
        arguments.push(profile_name.to_owned());
    }

    match urls.len() {
        0 => {}
        1 => {
            arguments.push("-new-window".to_owned());
            arguments.push(urls[0].clone());
        }
        _ => {
            // With an existing Firefox instance, `-new-window` only applies to the first bare
            // URL; additional URLs open in the already-running window. Pair each URL with
            // `-url` so every tab lands in the same new window.
            arguments.push("-new-window".to_owned());
            for url in urls {
                arguments.push("-url".to_owned());
                arguments.push(url.clone());
            }
        }
    }

    arguments
}

#[cfg(test)]
mod tests {
    use super::build_browser_arguments;
    use crate::domain::layout::BrowserKind;

    #[test]
    fn builds_chromium_arguments_for_a_new_window() {
        assert_eq!(
            build_browser_arguments(
                BrowserKind::Edge,
                &[
                    "https://one.example".to_owned(),
                    "https://two.example".to_owned()
                ],
                None,
            ),
            vec![
                "--new-window".to_owned(),
                "https://one.example".to_owned(),
                "https://two.example".to_owned()
            ]
        );
    }

    #[test]
    fn builds_firefox_arguments_for_a_single_url() {
        assert_eq!(
            build_browser_arguments(
                BrowserKind::Firefox,
                &["https://example.com".to_owned()],
                Some("work"),
            ),
            vec![
                "-P".to_owned(),
                "work".to_owned(),
                "-new-window".to_owned(),
                "https://example.com".to_owned()
            ]
        );
    }

    #[test]
    fn builds_firefox_arguments_for_multiple_urls_in_one_new_window() {
        assert_eq!(
            build_browser_arguments(
                BrowserKind::Firefox,
                &[
                    "https://one.example".to_owned(),
                    "https://two.example".to_owned()
                ],
                None,
            ),
            vec![
                "-new-window".to_owned(),
                "-url".to_owned(),
                "https://one.example".to_owned(),
                "-url".to_owned(),
                "https://two.example".to_owned()
            ]
        );
    }

    #[test]
    fn keeps_urls_for_the_default_browser_adapter() {
        assert_eq!(
            build_browser_arguments(
                BrowserKind::SystemDefault,
                &["https://example.com".to_owned()],
                None,
            ),
            vec!["https://example.com".to_owned()]
        );
    }
}
