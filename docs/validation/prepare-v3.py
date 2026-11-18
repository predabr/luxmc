from pathlib import Path
import json
import re

root = Path(__file__).resolve().parents[2]

def replace(path, before, after):
    file = root / path
    text = file.read_text(encoding="utf-8")
    if before not in text:
        raise ValueError(f"Expected text missing: {path}: {before}")
    file.write_text(text.replace(before, after), encoding="utf-8")

for path in ("package.json", "src-tauri/tauri.conf.json"):
    replace(path, '"version": "2.0.2"', '"version": "3.0.0"')
replace("package.json", "--project-name=luxmc", "--project-name=luxmc --branch=main")
replace("src-tauri/Cargo.toml", 'version = "2.0.2"', 'version = "3.0.0"')
replace("src-tauri/Cargo.lock", 'name = "luxmc"\nversion = "2.0.2"', 'name = "luxmc"\nversion = "3.0.0"')
replace("src/routes/+layout.svelte", '<script lang="ts">', '<script lang="ts">\nimport { APP_VERSION } from "$lib/version";')
replace("src/routes/+layout.svelte", '"Luxmc v2.0.2"', '`Luxmc v${APP_VERSION}`')
replace("src/routes/settings/+page.svelte", '<script lang="ts">', '<script lang="ts">\nimport { APP_VERSION } from "$lib/version";')
replace("src/routes/settings/+page.svelte", '"2.0.2"', 'APP_VERSION')
replace("website/app.js", '"v2.0.2"', '"v3.0.0"')
replace("website/lib/releases.js", 'Luxmc-Cloudflare-Pages/2.0.2', 'Luxmc-Cloudflare-Pages/3.0.0')
replace("website/install.sh", '${LATEST_TAG:-v2.0.2}', '${LATEST_TAG:-v3.0.0}')
replace("website/index.html", 'class="live-version-tag">Release oficial', 'class="live-version-tag">v3.0.0')
replace("packaging/arch/PKGBUILD", '2.0.0', '3.0.0')
replace("README.md", '2.0.1', '3.0.0')
replace("README.md", 'Luxmc_3.0.0_x64-setup.exe', 'Lux MC Launcher.exe')
replace("README.md", '/download/Lux MC Launcher.exe', '/download/Lux%20MC%20Launcher.exe')
replace("README.md", '100% dos seus dados, senhas e configurações permanecem armazenados localmente na sua máquina', 'Dados de instâncias ficam no computador; contas Luxmc, skins compartilhadas e recursos sociais usam o portal quando ativados')
replace("scripts/launcher-video-fixture.cjs", '2.0.2', '3.0.0')
replace("src-tauri/client-agent/io/github/luxmc/client/LuxmcClientAgent.java", '(v2.0.2)', '(v3.0.0)')
for path in ("src-tauri/src/core/share_codes.rs", "src-tauri/src/core/mods/pack_download.rs", "src-tauri/src/commands/modpack_update.rs", "src-tauri/src/commands/instances.rs"):
    file = root / path
    text = file.read_text(encoding="utf-8")
    text = re.sub(r'"Luxmc/2\.0\.[12]"', 'concat!("Luxmc/", env!("CARGO_PKG_VERSION"))', text)
    file.write_text(text, encoding="utf-8")

translations = {
    "pt-BR": {
        "title": "Luxmc 3.0 — Seu launcher atualizado",
        "summary": "Interface revisada, personalizador com NameMC, catálogos corrigidos e novidades oficiais do Minecraft.",
        "appearance": "Cartões maiores, wallpapers importados e contraste melhor no tema claro",
        "skins": "Catálogo NameMC, capa por skin e aplicação com sincronização Microsoft",
        "content": "Instalação de conteúdo em instâncias compatíveis e preparação do jogo com cache",
        "social": "Correções nas salas, nos códigos de convite e nos pedidos de amizade",
        "updates": "Notícias automáticas, idiomas do sistema e instalador Windows revisado"
    },
    "en": {
        "title": "Luxmc 3.0 — Your updated launcher",
        "summary": "A refreshed interface, NameMC appearance tools, corrected catalogs and official Minecraft news.",
        "appearance": "Larger cards, imported wallpapers and improved light theme contrast",
        "skins": "NameMC catalog, a cape per skin and application with Microsoft synchronization",
        "content": "Content installation in compatible instances and cached game preparation",
        "social": "Fixes for rooms, invite codes and friend requests",
        "updates": "Automatic news, system language detection and revised Windows installer"
    },
    "es": {
        "title": "Luxmc 3.0 — Tu launcher actualizado",
        "summary": "Interfaz renovada, personalizador con NameMC, catálogos corregidos y noticias oficiales de Minecraft.",
        "appearance": "Tarjetas más grandes, fondos importados y mejor contraste en el tema claro",
        "skins": "Catálogo NameMC, una capa por skin y aplicación con sincronización Microsoft",
        "content": "Instalación de contenido en instancias compatibles y preparación del juego con caché",
        "social": "Correcciones en salas, códigos de invitación y solicitudes de amistad",
        "updates": "Noticias automáticas, idioma del sistema e instalador Windows revisado"
    }
}
for locale, values in translations.items():
    file = root / f"src/lib/i18n/{locale}.json"
    data = json.loads(file.read_text(encoding="utf-8"))
    data["app"]["version"] = "v3.0.0"
    data["release3"] = values
    file.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

article = '''
        {
            id: "v3.0.0",
            title: uiText("release3.title"),
            tag: "3.0",
            tagColor: "text-brand-400 bg-brand-500/10 border-brand-500/30",
            date: "2026-10-05",
            version: "v3.0.0",
            image: "/news_1.jpg",
            summary: uiText("release3.summary"),
            highlights: ["appearance", "skins", "content", "social", "updates"].map(key => uiText(`release3.${key}`)),
            link: "https://github.com/predabr/luxmc/releases/tag/v3.0.0"
        },'''
replace("src/routes/news/+page.svelte", 'const launcherNews: LauncherArticle[] = $derived([', 'const launcherNews: LauncherArticle[] = $derived([' + article)
widget = '''
        {
            id: "luxmc-v3-0-0",
            title: uiText("release3.title"),
            body: uiText("release3.summary"),
            publishedAt: "2026-10-05",
            tag: "v3.0.0",
            url: "/news"
        },'''
replace("src/lib/components/home/NewsFeedWidget.svelte", 'const defaultNews: Release[] = $derived([', 'const defaultNews: Release[] = $derived([' + widget)
replace("src/lib/components/ui/ChangelogPanel.svelte", '{#each entries as e (e.version)}', '{#each entries as e (`${e.version}:${e.date}`)}')
changelog = '''
            ChangelogEntry {
                version: env!("CARGO_PKG_VERSION").into(),
                date: "2026-10-05".into(),
                title: "Luxmc 3.0".into(),
                highlights: vec![
                    "Refreshed instance cards and light theme contrast".into(),
                    "NameMC catalog, saved skin and cape pairs, Microsoft synchronization".into(),
                    "Compatible content catalogs, cached game preparation and multiplayer room fixes".into(),
                    "Official Minecraft news refresh and system language detection".into(),
                    "Updated portal, Windows installer and data-preserving uninstaller".into(),
                ],
            },'''
replace("src-tauri/src/commands/changelog.rs", 'Ok(vec![', 'Ok(vec![' + changelog)
print("Version 3.0.0 prepared; product identifiers and names preserved.")
