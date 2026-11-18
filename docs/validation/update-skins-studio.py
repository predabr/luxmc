from pathlib import Path
p=Path('src/routes/skins/+page.svelte')
s=p.read_text(encoding='utf-8')
s=s.replace('import Button from "$lib/components/ui/Button.svelte";', 'import Button from "$lib/components/ui/Button.svelte";\n    import NameMcPicker from "$lib/components/skins/NameMcPicker.svelte";\n    import type { NameMcSkin } from "$lib/api/namemc";')
s=s.replace('\t\tShirt\n', '\t\tShirt,\n        Globe2,\n        ArrowUpRight\n')
s=s.replace('let selectedCape = $state<CapeType>("luxmc");','let selectedCape = $state<CapeType>("none");\n    let showNameMc = $state(false);')
a=s.index('\tconst capeList:')
b=s.index('\n\tfunction hydrateAppearance()',a)
s=s[:a]+'''    const capeTypes: readonly CapeType[] = ["none", "custom", "luxmc", "optifine", "migrator", "cherry", "vanilla", "minecon2011", "minecon2012", "minecon2013", "minecon2015", "minecon2016", "tiktok", "twitch"];
    const currentCapePreview = $derived(selectedCape === "none" ? "" : selectedCape === "custom" ? customCapePreview : getCapePreviewDataUrl(selectedCape));
    const accountCapeUrl = $derived(account.value?.capeUrl || "");
''' +s[b:]
s=s.replace('capeList.some(item => item.id === cape)', 'capeTypes.includes(cape)')
s=s.replace('capeList.some(cape => cape.id === draft.cape)', 'capeTypes.includes(draft.cape)')
anchor='    let importController: AbortController | null = null;'
s=s.replace(anchor,'''    async function importNameMcSkin(skin: NameMcSkin) {
        importController?.abort();
        const controller = new AbortController();
        importController = controller;
        const image = await loadTextureImage(skin.skinUrl, controller.signal);
        if (controller.signal.aborted) return;
        const canvas = textureCanvas(image);
        if (classifyTexture(canvas) !== "skin") throw new Error(uiText("ui.4b23da27cc1f6901"));
        const model = inferSkinModelType(canvas);
        const existing = savedSkins.find(item => item.url === skin.skinUrl);
        const saved: SavedSkinItem = existing || { id: crypto.randomUUID(), name: `NameMC ${skin.skinId}`, url: skin.skinUrl, model };
        if (!existing) {
            const next = [saved, ...savedSkins];
            localStorage.setItem("luxmc_saved_skins", JSON.stringify(next));
            savedSkins = next;
        }
        selectValidatedSkin(saved, model);
        toast(uiText("skinsStudio.nameMcImported"), "success");
    }

''' +anchor)
s=s.replace('"Aplicando e sincronizando com Microsoft…"', 'uiText("skinsStudio.syncing")').replace('"Aplicando…"', 'uiText("skinsStudio.applying")')
a=s.index('            <section class="rounded-2xl p-4 space-y-3')
b=s.index('\n\t\t\t<section class="space-y-3">',a)
s=s[:a]+'''            <section class="rounded-2xl p-5 space-y-4 bg-bg-elevated border border-fg/10 shadow-sm" aria-label={uiText("ui.cd41a52072c213e4")}>
                <div class="flex flex-wrap items-center justify-between gap-4">
                    <div>
                        <h2 class="font-semibold text-fg">{uiText("ui.cd41a52072c213e4")}</h2>
                        <p class="mt-1 text-xs leading-relaxed text-fg-muted">{uiText("skinsStudio.capeDescription")}</p>
                    </div>
                    <Button variant="secondary" size="xl" onclick={handleCustomCapeUpload}><Upload class="h-5 w-5" />{uiText("skinsStudio.addCape")}</Button>
                </div>
                <div class="flex flex-wrap items-center gap-3">
                    {#if selectedCape !== "none"}
                        <div class="flex items-center gap-3 rounded-xl border border-fg/10 bg-fg/[0.03] px-4 py-3">
                            {#if currentCapePreview}<img class="h-14 w-9 object-contain [image-rendering:pixelated]" src={currentCapePreview} alt={uiText("skinsStudio.currentCape")} />{/if}
                            <span class="text-sm font-medium text-fg">{uiText("skinsStudio.currentCape")}</span>
                        </div>
                    {/if}
                    <Button variant={selectedCape === "none" ? "outline" : "ghost"} size="lg" aria-pressed={selectedCape === "none"} onclick={() => selectedCape = "none"}>{uiText("skinsStudio.removeCape")}</Button>
                    {#if customCapeDataUrl && selectedCape !== "custom"}
                        <Button variant="secondary" size="lg" onclick={() => selectedCape = "custom"}>{uiText("ui.b0dcf66f807f8b19")}</Button>
                    {/if}
                    {#if accountCapeUrl && accountCapeUrl !== customCapeDataUrl}
                        <Button variant="secondary" size="lg" onclick={() => { customCapeDataUrl = accountCapeUrl; selectedCape = "custom"; }}>{uiText("skinsStudio.accountCape")}</Button>
                    {/if}
                </div>
            </section>

            <div class="rounded-2xl border border-brand-400/20 bg-brand-500/[0.06] p-5 flex flex-col gap-4 sm:flex-row sm:items-center">
                <div class="min-w-0 flex-1">
                    <h2 class="font-semibold text-fg">{uiText("skinsStudio.collection")}</h2>
                    <p class="mt-1 text-sm leading-relaxed text-fg-muted">{uiText("skinsStudio.collectionDescription")}</p>
                </div>
                <Button variant="primary" size="xl" onclick={() => showNameMc = true}><Globe2 class="h-5 w-5" />{uiText("skinsStudio.nameMcBrowse")}<ArrowUpRight class="h-4 w-4" /></Button>
            </div>
''' +s[b:]
a=s.index('\n\t\t\t\t<div class="space-y-2">\n\t\t\t\t\t<div class="flex items-center justify-between">\n\t\t\t\t\t\t<span class="text-xs font-bold text-fg/70 block">{uiText("ui.d49f9b211515e655")}</span>')
b=s.index('\n\t\t\t\t<div class="flex items-center justify-end gap-2 pt-2',a)
s=s[:a]+s[b:]
s=s.replace('class="px-3 py-1 rounded-lg text-[11px] font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {activeAnimation === anim.id ? \'bg-brand-500 text-brand-foreground shadow-sm\' : \'text-fg/50 hover:text-fg hover:bg-fg/[0.06]\'}"','class={launcherButton({ variant: activeAnimation === anim.id ? "primary" : "ghost", size: "sm" })} aria-pressed={activeAnimation === anim.id}')
s=s.replace('class="w-full max-w-[200px] flex items-center justify-center gap-2"', 'class="w-full max-w-[280px] flex items-center justify-center gap-2"')
s=s.replace('<span>{uiText("ui.4cc3c3db5b5a582d")}</span>', '<span>{uiText("skinsStudio.model")}</span>')
s=s.replace('\n\t{#if showEditModal}', '\n    <NameMcPicker isOpen={showNameMc} onClose={() => showNameMc = false} onImport={importNameMcSkin} onImportFile={handleAddSkinFile} />\n\n\t{#if showEditModal}')
p.write_text(s,encoding='utf-8')
