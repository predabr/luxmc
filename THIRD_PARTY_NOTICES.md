# Third-party notices — Luxmc

New distributions carrying the Luxmc Noncommercial License 2.0 apply it only to original Luxmc code owned by its licensors. Third-party code, dependencies, runtimes and artwork retain their own licenses. Previously distributed GPL copies retain their grants; the historical GPL text is archived under docs/licensing/archive/.

## Source dependencies

Rust dependency licenses are declared in their Cargo manifests; vendored components preserve their upstream license files under `src-tauri/vendor/`. Frontend dependency licenses and notices are distributed in the package contents. Build lockfiles identify the precise versions.

The first metadata review found permissive licenses and compatible alternative-license declarations in the current production dependency graph. The old `cmdk-svelte@0.0.1` package and its repository did not declare a license; it has been removed and the command palette now uses the already-used MIT-licensed Bits UI implementation.

The Java client agent includes ASM under its own license. Its notice is retained at `src-tauri/client-agent/dependencies/ASM-LICENSE.txt` and included in generated JARs by the build script.

The website uses [jose](https://github.com/panva/jose), distributed under the MIT license, to validate Microsoft OpenID Connect identity tokens. Its package retains the upstream license; `pnpm-lock.yaml` identifies the installed version.

## Game content and artwork

Minecraft names, textures, skins and official game imagery belong to their respective rights holders. Modpack and mod cover art belongs to the relevant project authors. Screenshots from the Complementary gallery are identified in `static/cinema/sources.json`; their attribution is not evidence of a new GPL license grant. These assets must be reviewed under their authors' applicable permissions during provider onboarding.

User-imported wallpapers, GIFs, skins and worlds are local user content and are not relicensed or included in signing submissions. No claim is made that a user's import is authorized for redistribution.

## Optional runtime components

Minecraft, modpack files, Java runtimes and optional network-provider downloads retain their upstream licenses. They are fetched separately when the corresponding feature is used, rather than signed as Luxmc-owned source. Wintun, WinPcap and Npcap license notices are retained under `src-tauri/licenses/` for relevant optional provider operations.

Microsoft WebView2 is a separately licensed Windows runtime. The installer may embed its official bootstrapper; its Microsoft signature and redistribution conditions must remain intact. Provider onboarding must explicitly assess whether bundled system-runtime components meet the program's system-library exception.

## Historical distribution

The prior Luxmc usage license is archived at `docs/licensing/archive/luxmc-license-1.0-2026.txt`. Existing release tags and previously published artifacts were not rewritten by this migration. Their license files and distribution records remain available in Git history. The 26 October 2026 change adopts the noncommercial terms for original source in subsequent distributions; it does not rewrite those artifacts. Eligibility for signing programs that require an Open Source license must be reassessed before applying under the new terms.

## Noncommercial license adoption, 26 October 2026

The maintainer explicitly confirmed ownership of the original Luxmc code and absence of unapproved external launcher code or contributions. Git history was reviewed for author identities. Cargo metadata for 891 resolved packages was inventoried: non-root packages exposing LGPL alternatives also offer MIT or Apache alternatives. This metadata review is not proof of authorship for every file or a legal opinion; upstream license files remain authoritative. Original-source terms do not override dependency or asset permissions. New external contributions require the explicit agreement described in CONTRIBUTING.md.
