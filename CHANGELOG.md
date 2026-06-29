# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-beta.1] - 2026-06-29

Première release beta publique du MVP Layout Manager 2.

### Added

- Création, édition, duplication et suppression de layouts persistants (SQLite local).
- Actions : lancer une application, placer une fenêtre existante, ouvrir une fenêtre navigateur avec plusieurs URL.
- Sélecteur de placement visuel avec presets (plein écran, moitiés, quarts, zone centrée, personnalisée).
- Exécution d'un layout avec progression, annulation et rapport partiel en cas d'erreur.
- Réduction optionnelle des fenêtres hors layout.
- Support multi-écrans et facteurs DPI mixtes, avec fallback d'écran.
- Navigateurs : Microsoft Edge, Google Chrome, Mozilla Firefox et navigateur système (limité).
- Réglages : navigateur préféré, délais, fallback d'écran, accès aux dossiers données et journaux.
- Installateur Windows NSIS (mode utilisateur courant, sans droits administrateur par défaut).

### Known limitations

- Windows 10 et 11 uniquement.
- Pas de signature de code : Windows SmartScreen peut afficher un avertissement à l'installation.
- Les applications exécutées avec des droits élevés ne peuvent pas être contrôlées.
- Le navigateur système par défaut ne garantit pas toujours une nouvelle fenêtre distincte.
- Pas de synchronisation cloud, import/export, bureaux virtuels ni déclencheurs automatiques.

[0.1.0-beta.1]: https://github.com/Palawizard/layout-manager-2/releases/tag/v0.1.0-beta.1
