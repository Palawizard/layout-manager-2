<div align="center">

# Layout Manager 2

### Organisez et appliquez vos espaces de travail Windows

[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=for-the-badge&logo=tauri)](https://v2.tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB?style=for-the-badge&logo=react)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-stable-000000?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-local-003B57?style=for-the-badge&logo=sqlite)](https://www.sqlite.org/)
[![License](https://img.shields.io/badge/Licence-MIT-green?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Plateforme-Windows_10%2F11-0078D4?style=for-the-badge&logo=windows)](https://www.microsoft.com/windows)

[Fonctionnalités](#-fonctionnalités) • [Téléchargement](#-téléchargement) • [Démarrage rapide](#-démarrage-rapide) • [Architecture](#-architecture) • [Vérification](#-vérification)

</div>

---

## ◆ Fonctionnalités

<table>
<tr>
<td width="50%">

### » Layouts

- **Création et édition** : nom, description et liste ordonnée d'actions
- **Duplication et suppression** : copies indépendantes avec confirmation
- **Persistance locale** : SQLite dans le dossier de données utilisateur
- **Trois types d'actions** : application, fenêtre existante, fenêtre navigateur
- **Presets de placement** : plein écran, moitiés, quarts, zone centrée, personnalisée
- **Aperçu multi-écrans** : sélection visuelle avec support des coordonnées négatives

</td>
<td width="50%">

### » Exécution

- **Lancement d'applications** : exécutable, arguments et répertoire de travail
- **Réutilisation intelligente** : évite les doublons si une fenêtre compatible existe
- **Placement Win32** : déplacement, redimensionnement et état final (normal, agrandi, réduit)
- **Navigateurs** : Edge, Chrome, Firefox et navigateur système (fallback limité)
- **Progression en direct** : statut par action, annulation et rapport partiel
- **Réduction optionnelle** : minimise les fenêtres hors layout en dernière phase

</td>
</tr>
<tr>
<td>

### » Fenêtres et écrans

- **Inventaire de bureau** : liste filtrée des fenêtres utiles (sans bruit système)
- **Matching déterministe** : exécutable, processus, classe, titre (regex) et index
- **Multi-écrans et DPI** : bounds normalisés, fallback d'écran et clamp dans la zone de travail
- **Sélecteur intégré** : recherche, rafraîchissement et matcher stable (jamais de handle persisté)
- **Timeouts bornés** : attente annulable pour les applications lentes

</td>
<td>

### » Expérience

- **Interface en français** : navigation `Layouts` et `Réglages`
- **Thème clair / sombre** : suit les préférences système
- **Réglages persistants** : navigateur préféré, délais et stratégie de fallback
- **Diagnostics** : journaux locaux avec rotation, dossiers données et journaux accessibles
- **Hors ligne** : aucun compte, cloud ni télémétrie

</td>
</tr>
<tr>
<td colspan="2">

### » Limites connues (beta)

- Windows 10 et 11 uniquement — pas de macOS ni Linux
- Installateur **non signé** : SmartScreen peut demander une confirmation supplémentaire
- Applications **élevées** (admin) non contrôlables par une app standard
- Navigateur système par défaut : ouverture possible sans garantie de nouvelle fenêtre distincte
- Pas d'import/export, bureaux virtuels, raccourcis globaux ni déclencheurs automatiques

</td>
</tr>
</table>

---

## ◆ Téléchargement

Les releases sont publiées sur [GitHub Releases](https://github.com/Palawizard/layout-manager-2/releases).

| Fichier                            | Description                                     |
| ---------------------------------- | ----------------------------------------------- |
| `Layout Manager 2_*_x64-setup.exe` | Installateur NSIS Windows (utilisateur courant) |
| `release-checksums.txt`            | Empreintes SHA-256 des artefacts                |

**Prérequis runtime** : Microsoft Edge WebView2 (installé automatiquement par l'installateur si nécessaire).

---

## ▶ Démarrage rapide

### Prérequis (développement)

- Windows 10 (1803+) ou Windows 11
- [Node.js](https://nodejs.org/) 22+
- [pnpm](https://pnpm.io/) 10
- [Rust](https://www.rust-lang.org/) stable (MSVC)
- Microsoft C++ Build Tools avec `Desktop development with C++`
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)

### Installation des dépendances

```powershell
pnpm install
```

### Lancer en développement

```powershell
pnpm tauri:dev
```

### Vérifier le projet

```powershell
pnpm check
```

Exécute formatage, lint, typecheck, tests frontend, build frontend, rustfmt, Clippy et tests Rust.

### Créer l'installateur Windows

```powershell
pnpm tauri:build
```

L'installateur est généré dans `src-tauri/target/release/bundle/`.

### Données utilisateur

Layouts, réglages et base SQLite sont stockés dans le dossier de données de l'application Tauri, accessible depuis **Réglages** → **Ouvrir le dossier des données**.

---

## ◆ Scripts utiles

<div align="center">

| Commande           | Description                            |
| ------------------ | -------------------------------------- |
| `pnpm dev`         | Serveur Vite seul (frontend)           |
| `pnpm tauri:dev`   | Application complète en développement  |
| `pnpm check`       | Suite qualité complète                 |
| `pnpm test`        | Tests unitaires et composants (Vitest) |
| `pnpm test:e2e`    | Parcours e2e mocké                     |
| `pnpm tauri:build` | Build production + installateur NSIS   |
| `pnpm rust:test`   | Tests Rust backend                     |

</div>

---

## ◆ Architecture

```
layout-manager-2/
├── src/                          → Frontend React (features, composants, bridge Tauri)
│   ├── app/                      → Routeur et providers
│   ├── features/
│   │   ├── layouts/              → Liste, éditeur et schémas de validation
│   │   ├── execution/            → Progression et rapport d'exécution
│   │   ├── settings/             → Réglages persistants
│   │   └── windows/              → Sélecteur de fenêtres de bureau
│   ├── components/ui/            → Primitives shadcn/ui partagées
│   └── lib/tauri/                → Commandes et événements typés
├── src-tauri/src/
│   ├── domain/                   → Modèles, règles et ports (sans Win32 ni SQLite)
│   ├── application/              → Use cases : layouts, exécution, matching
│   ├── infrastructure/           → Win32, processus, navigateurs, SQLite
│   └── commands/                 → Boundary Tauri et DTO
├── src-tauri/migrations/         → Schéma SQLite versionné
├── tests/e2e/                    → Parcours UI mocké
├── scripts/                      → Outils locaux (lecture base SQLite)
└── .github/workflows/            → CI qualité et pipeline release
```

### Flux d'exécution

```
Validation → Snapshot fenêtres → Lancement (apps + navigateurs)
    → Placement → Réduction optionnelle → Rapport
```

Le frontend communique uniquement via des commandes Tauri typées. Aucun accès direct à Win32 ou SQLite depuis l'interface.

---

## ◆ Stack technique

<div align="center">

| Couche            | Technologie                                                     |
| ----------------- | --------------------------------------------------------------- |
| **Shell desktop** | Tauri 2                                                         |
| **Backend**       | Rust 2024, crate `windows` (Win32)                              |
| **Frontend**      | React 19, TypeScript strict, Vite                               |
| **Styles**        | Tailwind CSS, shadcn/ui, Lucide                                 |
| **État UI**       | Zustand, React Hook Form, Zod                                   |
| **Persistance**   | SQLite (`rusqlite`), migrations versionnées                     |
| **Tests**         | Vitest, Testing Library, tests Rust + natifs opt-in `#[ignore]` |
| **Qualité**       | ESLint, Prettier, rustfmt, Clippy, GitHub Actions               |

</div>

### Pourquoi Tauri et Rust ?

- → Appels Win32 directs pour inventaire, placement et processus
- → Backend typé avec erreurs contrôlées et logs structurés
- → Interface web rapide à itérer sans embarquer Chromium complet
- → Installateur léger et permissions Tauri minimales

---

## ◆ Commandes Tauri exposées

| Domaine     | Commandes                                                                                   |
| ----------- | ------------------------------------------------------------------------------------------- |
| Application | `get_app_info`                                                                              |
| Layouts     | `list_layouts`, `get_layout`, `save_layout`, `duplicate_layout`, `delete_layout`            |
| Système     | `list_desktop_windows`, `list_monitors`, `validate_executable`, `resolve_launch_executable` |
| Navigateur  | `list_installed_browsers`                                                                   |
| Exécution   | `run_layout`, `cancel_layout_run`                                                           |
| Réglages    | `get_settings`, `save_settings`, `open_data_directory`                                      |

Événements de progression : `layout-run://started`, `layout-run://action-started`, `layout-run://action-completed`, `layout-run://completed`.

---

## ◆ Sécurité et confidentialité

- Application **locale** : pas de télémétrie ni de compte dans le MVP
- Capacités Tauri **minimales** ; pas de shell générique exposé au frontend
- Chemins, arguments et URL **validés côté Rust** avant exécution
- Aucun handle Windows (`HWND`) **persisté** en base
- Journaux avec **rotation** ; pas d'URL complètes ni d'arguments sensibles par défaut
- Installateur en mode **utilisateur courant** — pas d'élévation admin par défaut

---

## ◆ Vérification

```powershell
# Suite complète (recommandé avant contribution)
pnpm check

# Tests natifs Windows (modifient le bureau — opt-in)
cargo test --manifest-path src-tauri/Cargo.toml -- --ignored

# Parcours e2e mocké
pnpm test:e2e
```

> Le placement réel sur le bureau et l'installateur sur machine vierge doivent être validés manuellement.

---

## ◆ Contribution et release

- Branche d'intégration : `dev`
- Commits : `type(scope): thing done` (anglais, sans body)
- Releases : tag `v*` sur `main` déclenche le workflow GitHub Actions
- Changelog : voir [CHANGELOG.md](CHANGELOG.md)

---

## ◆ Licence

Distribué sous licence [MIT](LICENSE).

---

<div align="center">

Layout Manager 2 · Version beta · Windows 10/11

</div>
