# Luxmc

Linux-first Minecraft launcher built with **Tauri 2** (Rust) and **SvelteKit** (Svelte 5 + TypeScript + Tailwind CSS).

## Status

| Phase | Description | Status |
|-------|-------------|--------|
| P0  | Scaffolding, layout, theme tokens, components | done |
| P1  | Settings persistence, i18n, store/sqlite | pending |
| P2  | Microsoft / Xbox authentication | pending |
| P3  | Profile CRUD + selector | pending |
| P4  | Minecraft vanilla install + launch | pending |
| P5  | Mods browser (Modrinth) + install + deps | pending |
| P6  | Fabric / Forge / NeoForge / Quilt installers | pending |
| P7  | CurseForge provider | pending |
| P8  | Logs UI, news, saves manager | pending |
| P9  | AppImage + .deb + CI | pending |
| P10 | Flatpak + AUR + Flathub | pending |

## Development

```sh
pnpm install
pnpm tauri dev
```

## Verification

```sh
pnpm check      # svelte-check (frontend)
pnpm build      # svelte-kit build
cargo check     # rust check (inside src-tauri/)
```

## Layout

- `src/` — SvelteKit frontend
  - `lib/components/ui/` — design system primitives
  - `lib/components/layout/` — Sidebar, Topbar
  - `lib/stores/` — account, profiles, settings (rune-based)
  - `lib/api/` — typed Tauri command wrappers
  - `lib/types/` — shared TS types
  - `routes/` — pages (Home, Profiles, Mods, News, Logs, Settings)
- `src-tauri/` — Rust backend
  - `commands/` — `#[tauri::command]` handlers (`system`, future: `auth`, `versions`, `mods`, `profiles`, `launch`, `settings`)
  - `core/` — domain logic (auth, minecraft, modloaders, mods, launcher)
  - `error.rs` — unified `AppError`

## Conventions

See `AGENTS.md`.
