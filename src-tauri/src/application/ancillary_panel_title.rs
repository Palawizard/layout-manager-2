use crate::application::transient_title::normalize_title_for_matching;

const ANCILLARY_PANEL_KEYWORDS: &[&str] = &[
    // Contact / friends side panels
    "contact list",
    "liste de contacts",
    "liste des contacts",
    "kontaktliste",
    "lista de contactos",
    "lista de contatos",
    "lista contatti",
    "friends list",
    "liste d'amis",
    "liste des amis",
    "freundesliste",
    "lista de amigos",
    // Chat / voice popouts
    "chat list",
    "liste de discussions",
    "discussion list",
    "voice chat",
    "voice channel",
    "canal vocal",
    // Generic overlays / tool panels
    "notification",
    "quick access",
    "mini profile",
    "pop-out",
    "popout",
    "pop out",
];

/// In-app tool strips and pop-out panels that share the main process but are not the client shell.
#[must_use]
pub fn has_ancillary_panel_title(title: &str) -> bool {
    let normalized = normalize_title_for_matching(title);
    if normalized.is_empty() {
        return false;
    }
    ANCILLARY_PANEL_KEYWORDS
        .iter()
        .any(|keyword| normalized.contains(keyword))
}

#[cfg(test)]
mod tests {
    use super::has_ancillary_panel_title;

    #[test]
    fn detects_steam_contact_list_titles() {
        assert!(has_ancillary_panel_title("Liste de contacts"));
        assert!(has_ancillary_panel_title("Contact list"));
    }

    #[test]
    fn keeps_main_application_titles() {
        assert!(!has_ancillary_panel_title("Steam"));
        assert!(!has_ancillary_panel_title("Friends - Discord"));
    }
}
