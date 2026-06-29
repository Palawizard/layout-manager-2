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
    let mut arguments = vec!["-new-window".to_owned()];
    if let Some(profile_name) = profile.filter(|value| !value.trim().is_empty()) {
        arguments.push("-P".to_owned());
        arguments.push(profile_name.to_owned());
    }
    arguments.extend(urls.iter().cloned());
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
    fn builds_firefox_arguments_for_a_new_window() {
        assert_eq!(
            build_browser_arguments(
                BrowserKind::Firefox,
                &["https://example.com".to_owned()],
                Some("work"),
            ),
            vec![
                "-new-window".to_owned(),
                "-P".to_owned(),
                "work".to_owned(),
                "https://example.com".to_owned()
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
