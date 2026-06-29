/// Keywords that commonly appear in splash, updater and bootstrap window titles.
/// Matching uses accent-insensitive normalization.
const TRANSIENT_TITLE_KEYWORDS: &[&str] = &[
    // English
    "updater",
    "updating",
    "update available",
    "installing",
    "installation",
    "splash",
    "loading",
    "please wait",
    "wait",
    "setup",
    "patching",
    "maintenance",
    "checking for updates",
    "initializing",
    "preparing",
    "preparation",
    "starting",
    "launching",
    "booting",
    "connecting",
    "syncing",
    "synchronizing",
    "restoring",
    "restarting",
    "processing",
    // French
    "chargement",
    "patienter",
    "veuillez patienter",
    "demarrage",
    "initialisation",
    "preparation",
    "mise a jour",
    "installation en cours",
    "connexion",
    "demarrage en cours",
    // Login / connect bootstrap dialogs
    "sign in",
    "log in",
    "login",
    "se connecter",
    "connect to",
    "connectez-vous",
    "anmelden",
    "einloggen",
    "iniciar sesion",
    "iniciar sesión",
    "accedi",
    "connexion a",
    "connexion à",
    "verbinden mit",
    "zaloguj",
    "войти",
    // German
    "wird geladen",
    "laden",
    "bitte warten",
    "aktualisierung",
    "aktualisiert",
    "wird gestartet",
    "startet",
    "vorbereitung",
    "verbindung",
    // Spanish
    "cargando",
    "espere",
    "por favor espere",
    "actualizacion",
    "actualizando",
    "instalacion",
    "iniciando",
    "preparando",
    "conectando",
    // Italian
    "caricamento",
    "attendere",
    "aggiornamento",
    "aggiornando",
    "installazione",
    "avvio",
    "preparazione",
    "connessione",
    // Portuguese
    "carregando",
    "aguarde",
    "atualizacao",
    "atualizando",
    "instalacao",
    "iniciando",
    "preparacao",
    "conectando",
    // Dutch
    "even geduld",
    "bijwerken",
    "installeren",
    "opstarten",
    "voorbereiden",
    "verbinden",
    // Polish
    "ladowanie",
    "prosze czekac",
    "aktualizacja",
    "instalacja",
    "uruchamianie",
    "przygotowywanie",
    // Russian (transliterated)
    "zagruzka",
    "zhdite",
    "obnovlenie",
    "ustanovka",
    "zapusk",
    // Japanese (romaji fragments)
    "yomikomi",
    "shori chuu",
    "koshin",
    // Chinese (pinyin fragments)
    "jiazai",
    "qing shaodeng",
    "gengxin",
    "anzhuang",
    "qidong",
    // Korean
    "로딩",
    "불러오는",
    "업데이트",
    "설치",
    "시작",
];

#[must_use]
pub fn has_transient_title(title: &str) -> bool {
    let normalized = normalize_title_for_matching(title);
    if normalized.is_empty() {
        return false;
    }
    if looks_like_progress_title(&normalized) {
        return true;
    }
    TRANSIENT_TITLE_KEYWORDS
        .iter()
        .any(|keyword| normalized.contains(keyword))
}

fn looks_like_progress_title(normalized: &str) -> bool {
    if normalized.len() > 96 {
        return false;
    }
    let ends_with_ellipsis = normalized.ends_with("...")
        || normalized.ends_with("…")
        || normalized.ends_with("..");
    if !ends_with_ellipsis {
        return false;
    }
    const PROGRESS_MARKERS: &[&str] = &[
        "load", "wait", "start", "launch", "boot", "prep", "init", "install", "update",
        "connect", "sync", "charg", "patient", "demarr", "laden", "warten", "carg", "espere",
        "caric", "attend", "carreg", "aguarde", "ladow", "zagruz", "jiaz", "gengx",
    ];
    PROGRESS_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
}

#[must_use]
pub fn normalize_title_for_matching(title: &str) -> String {
    fold_diacritics(&title.trim().to_lowercase())
}

fn fold_diacritics(text: &str) -> String {
    text.chars().map(fold_char).collect()
}

fn fold_char(character: char) -> char {
    match character {
        'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' | 'æ' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'ö' | 'õ' | 'ø' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        'ß' => 's',
        'ł' => 'l',
        'ą' => 'a',
        'ć' => 'c',
        'ę' => 'e',
        'ń' => 'n',
        'ś' => 's',
        'ź' | 'ż' => 'z',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::has_transient_title;

    #[test]
    fn detects_english_and_french_loading_titles() {
        assert!(has_transient_title("Loading Steam..."));
        assert!(has_transient_title("Chargement de Steam..."));
        assert!(has_transient_title("Démarrage de l’application..."));
    }

    #[test]
    fn detects_other_common_languages() {
        assert!(has_transient_title("Wird geladen..."));
        assert!(has_transient_title("Cargando aplicación..."));
        assert!(has_transient_title("Caricamento in corso..."));
    }

    #[test]
    fn keeps_regular_application_titles() {
        assert!(!has_transient_title("VIVE Hub 2.5.5"));
        assert!(!has_transient_title("Friends - Discord"));
        assert!(!has_transient_title("Steam"));
    }

    #[test]
    fn treats_login_bootstrap_titles_as_transient() {
        assert!(has_transient_title("Se connecter à Steam"));
        assert!(has_transient_title("Sign in to Steam"));
    }
}
