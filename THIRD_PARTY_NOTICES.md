# Third-party notices — Luxmc

The GPL-3.0-only grant applies to original Luxmc code. It does not replace third-party licenses or claim ownership of third-party content.

## Source dependencies

Rust dependency licenses are declared in their Cargo manifests; vendored components preserve their upstream license files under `src-tauri/vendor/`. Frontend dependency licenses and notices are distributed in the package contents. Build lockfiles identify the precise versions.

The first metadata review found permissive licenses and compatible alternative-license declarations in the current production dependency graph. The old `cmdk-svelte@0.0.1` package and its repository did not declare a license; it has been removed and the command palette now uses the already-used MIT-licensed Bits UI implementation.

The Java client agent includes ASM under its own license. Its notice is retained at `src-tauri/client-agent/dependencies/ASM-LICENSE.txt` and included in generated JARs by the build script.

## Game content and artwork

Minecraft names, textures, skins and official game imagery belong to their respective rights holders. Modpack and mod cover art belongs to the relevant project authors. Screenshots from the Complementary gallery are identified in `static/cinema/sources.json`; their attribution is not evidence of a new GPL license grant. These assets must be reviewed under their authors' applicable permissions during provider onboarding.

User-imported wallpapers, GIFs, skins and worlds are local user content and are not relicensed or included in signing submissions. No claim is made that a user's import is authorized for redistribution.

## Optional runtime components

Minecraft, modpack files, Java runtimes and optional network-provider downloads retain their upstream licenses. They are fetched separately when the corresponding feature is used, rather than signed as Luxmc-owned source. Wintun, WinPcap and Npcap license notices are retained under `src-tauri/licenses/` for relevant optional provider operations.

Microsoft WebView2 is a separately licensed Windows runtime. The installer may embed its official bootstrapper; its Microsoft signature and redistribution conditions must remain intact. Provider onboarding must explicitly assess whether bundled system-runtime components meet the program's system-library exception.

## Historical distribution

The prior Luxmc usage license is archived at `docs/licensing/archive/luxmc-license-1.0-2026.txt`. Existing release tags and previously published artifacts were not rewritten by this migration. Their license files and distribution records remain available in Git history. Future signing candidates are built from the source commit carrying the GPLv3 migration.
