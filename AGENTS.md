# AGENTS.md

> Project: **Luxmc** — Linux-first Minecraft launcher.
> Stack: Tauri 2 (Rust) + SvelteKit (Svelte 5 + TS) + Tailwind CSS.

## Build & verification

| Action | Command |
| --- | --- |
| Install deps | `pnpm install` |
| Dev (Tauri + SvelteKit) | `pnpm tauri dev` |
| Frontend check | `pnpm check` (svelte-check) |
| Frontend build | `pnpm build` |
| Tauri release bundle | `pnpm tauri build` |
| Rust check | `cargo check` (inside `src-tauri/`) |

Always run `pnpm check` after editing `.svelte` / `.ts` files, and `cargo check` after editing files under `src-tauri/`.

## Project layout (high level)

- `src/` — SvelteKit frontend (Svelte 5, runes, Tailwind).
- `src-tauri/` — Rust backend, Tauri commands.
- `static/` — static assets served as-is.
- See `src/lib/` for components, stores, API wrappers.

## Conventions

- No comments unless explicitly requested.
- TypeScript strict mode is on.
- Use Svelte 5 runes (`$state`, `$derived`, `$effect`), not legacy stores.
- Tailwind tokens are defined in `tailwind.config.ts`; never hardcode hex values.
- All Tauri side-effects go through `#[tauri::command]` handlers; frontend uses typed wrappers in `src/lib/api/`.
