# Layout Manager 2

Application Windows pour enregistrer des layouts, lancer des applications et des pages web, puis placer leurs fenêtres sur les bons écrans.

## Stack

- Tauri 2 et Rust 2024
- React 19, TypeScript et Tailwind CSS
- SQLite pour les données locales
- pnpm 10

## How to run

Prérequis : Windows 10 ou 11, Node.js 22+, pnpm 10, Rust stable, Microsoft C++ Build Tools avec `Desktop development with C++` et Microsoft Edge WebView2.

```powershell
pnpm install
pnpm tauri:dev
```

Vérifier le projet :

```powershell
pnpm check
```

Créer l’installateur Windows :

```powershell
pnpm tauri:build
```

L’installateur est généré dans `src-tauri/target/release/bundle/`. Les données utilisateur et les journaux sont stockés dans le dossier de données de l’application, accessible depuis les réglages.
