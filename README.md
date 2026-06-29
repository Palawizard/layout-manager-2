# Layout Manager 2

Application Windows pour enregistrer des layouts, lancer des applications et des pages web, puis placer leurs fenêtres sur les bons écrans.

## Stack

- Tauri 2 et Rust 2024
- React 19, TypeScript et Tailwind CSS
- SQLite pour les données locales
- pnpm 10

## How to run

Prérequis : Node.js 22+, pnpm 10, Rust stable, Microsoft C++ Build Tools avec `Desktop development with C++` et Microsoft Edge WebView2.

```powershell
pnpm install
pnpm tauri:dev
```

Vérifier le projet :

```powershell
pnpm check
```

Créer l’application installable :

```powershell
pnpm tauri:build
```
