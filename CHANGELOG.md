# Journal des modifications

Toutes les évolutions notables du projet sont consignées dans ce fichier.

Le format s’inspire de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le versionnement suit [Semantic Versioning](https://semver.org/lang/fr/).

## [0.1.0-beta.1] - 2026-06-29

Première version bêta publique du MVP Layout Manager 2.

### Ajouté

- Création, édition, duplication et suppression de layouts persistants (SQLite local).
- Actions : lancer une application, placer une fenêtre existante, ouvrir une fenêtre navigateur avec plusieurs URL.
- Sélecteur de placement visuel avec préréglages (plein écran, moitiés, quarts, zone centrée, personnalisée).
- Exécution d’un layout avec progression, annulation et rapport partiel en cas d’erreur.
- Réduction optionnelle des fenêtres hors layout.
- Prise en charge multi-écrans et facteurs DPI mixtes, avec repli sur un autre écran.
- Navigateurs : Microsoft Edge, Google Chrome, Mozilla Firefox et navigateur système (limité).
- Réglages : navigateur préféré, délais, stratégie de repli d’écran, accès aux dossiers de données et de journaux.
- Installateur Windows NSIS (mode utilisateur courant, sans droits administrateur par défaut).

### Limites connues

- Windows 10 et 11 uniquement.
- Pas de signature de code : Windows SmartScreen peut afficher un avertissement à l’installation.
- Les applications exécutées avec des droits élevés ne peuvent pas être contrôlées.
- Le navigateur système par défaut ne garantit pas toujours une nouvelle fenêtre distincte.
- Pas de synchronisation cloud, import/export, bureaux virtuels ni déclencheurs automatiques.

[0.1.0-beta.1]: https://github.com/Palawizard/layout-manager-2/releases/tag/v0.1.0-beta.1
