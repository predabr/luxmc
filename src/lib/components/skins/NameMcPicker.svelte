<script lang="ts">
    import { onDestroy } from "svelte";
    import { Globe2, ExternalLink, Link2, Upload, ArrowUpRight, ShieldCheck } from "lucide-svelte";
    import Modal from "$lib/components/ui/Modal.svelte";
    import Button from "$lib/components/ui/Button.svelte";
    import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { listenNameMcSelection, nameMcClosePicker, nameMcImportSkin, nameMcOpenPicker, type NameMcSkin } from "$lib/api/namemc";
    import { openUrl } from "@tauri-apps/plugin-opener";

    let { isOpen, onClose, onImport, onImportFile }: {
        isOpen: boolean;
        onClose: () => void;
        onImport: (skin: NameMcSkin) => Promise<void>;
        onImportFile: () => Promise<void>;
    } = $props();
    let skinLink = $state("");
    let importing = $state(false);
    let opening = $state(false);
    let showFallback = $state(false);
    let error = $state("");
    let generation = 0;

    async function importSkin(source = skinLink) {
        if (importing || !source.trim()) return;
        importing = true;
        error = "";
        const attempt = ++generation;
        try {
            const skin = await nameMcImportSkin(source.trim());
            if (attempt !== generation || !isOpen) return;
            await onImport(skin);
            if (attempt !== generation || !isOpen) return;
            await nameMcClosePicker().catch(() => {});
            onClose();
        } catch {
            if (attempt === generation && isOpen) { error = uiText("skinsStudio.nameMcFailed"); showFallback = true; }
        } finally {
            if (attempt === generation) importing = false;
        }
    }

    export async function browse() {
        if (opening) return;
        showFallback = false;
        opening = true;
        error = "";
        try {
            await nameMcOpenPicker();
        } catch {
            error = uiText("skinsStudio.nameMcBrowseFailed");
            showFallback = true;
        } finally {
            opening = false;
        }
    }

    export function showLinkImport() { showFallback = true; }

    function close() {
        generation++;
        importing = false;
        void nameMcClosePicker().catch(() => {});
        onClose();
    }

    $effect(() => {
        if (!isOpen) return;
        let disposed = false;
        let unsubscribe: (() => void) | undefined;
        void listenNameMcSelection(source => {
            if (disposed || !isOpen) return;
            skinLink = source;
            void importSkin(source);
        }).then(stop => {
            if (disposed) stop();
            else unsubscribe = stop;
        }).catch(() => {});
        return () => {
            disposed = true;
            generation++;
            importing = false;
            unsubscribe?.();
        };
    });

    onDestroy(() => { void nameMcClosePicker().catch(() => {}); });
</script>

<Modal isOpen={isOpen && showFallback} onClose={close} title={uiText("skinsStudio.nameMcTitle")} maxWidth="max-w-2xl">
    <div class="space-y-5">
        <div class="rounded-2xl border border-brand-400/20 bg-brand-500/10 p-5 flex flex-col gap-4 sm:flex-row sm:items-center">
            <div class="rounded-2xl border border-brand-400/20 bg-brand-500/10 p-4 self-start"><Globe2 class="h-7 w-7 text-brand-400" /></div>
            <div class="min-w-0 flex-1 space-y-1">
                <h3 class="font-semibold text-fg">NameMC</h3>
                <p class="text-sm leading-relaxed text-fg-muted">{uiText("skinsStudio.nameMcDescription")}</p>
            </div>
        </div>
        <div class="grid gap-3 sm:grid-cols-2">
            <Button variant="primary" size="xl" loading={opening} disabled={importing} onclick={browse}><Globe2 class="h-5 w-5" />{uiText("skinsStudio.nameMcBrowse")}<ArrowUpRight class="h-4 w-4" /></Button>
            <Button variant="secondary" size="xl" disabled={importing} onclick={() => openUrl("https://namemc.com/minecraft-skins").catch(() => { error = uiText("skinsStudio.nameMcBrowseFailed"); })}><ExternalLink class="h-4 w-4" />{uiText("skinsStudio.nameMcBrowser")}</Button>
        </div>
        <div class="border-t border-fg/10 pt-5 space-y-3">
            <label for="namemc-skin-link" class="flex items-center gap-2 text-sm font-semibold text-fg"><Link2 class="h-4 w-4 text-brand-400" />{uiText("skinsStudio.nameMcLink")}</label>
            <form class="flex flex-col gap-2 sm:flex-row" onsubmit={event => { event.preventDefault(); void importSkin(); }}>
                <input id="namemc-skin-link" type="url" bind:value={skinLink} placeholder="https://namemc.com/skin/…" required disabled={importing} class="min-w-0 flex-1 rounded-xl border border-fg/10 bg-fg/[0.04] px-4 py-3 text-sm text-fg placeholder:text-fg/35 focus:outline-none focus:ring-2 focus:ring-brand-500/40" />
                <Button type="submit" variant="secondary" size="xl" loading={importing} disabled={!skinLink.trim()}>{uiText("skinsStudio.nameMcImport")}</Button>
            </form>
            {#if importing}<p class="text-sm text-brand-400" role="status">{uiText("skinsStudio.nameMcImporting")}</p>{/if}
            {#if error}<p class="rounded-xl border border-danger/20 bg-danger/10 p-3 text-sm text-danger" role="alert">{error}</p>{/if}
            <p class="text-xs leading-relaxed text-fg-muted">{uiText("skinsStudio.nameMcFallback")}</p>
            <Button variant="ghost" size="lg" disabled={importing} onclick={async () => { close(); await onImportFile(); }}><Upload class="h-4 w-4" />{uiText("skinsStudio.importFile")}</Button>
        </div>
        <p class="flex items-start gap-2 rounded-xl border border-fg/10 bg-fg/[0.03] p-3 text-xs leading-relaxed text-fg-muted"><ShieldCheck class="h-4 w-4 shrink-0 text-brand-400" />{uiText("skinsStudio.nameMcReview")}</p>
    </div>
</Modal>
