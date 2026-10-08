<script lang="ts">
    import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button } from "$lib/components/ui/button";
    import { onMount } from "svelte";
    import { fade, scale } from "svelte/transition";
    import { quintOut } from "svelte/easing";
    import { DownloadCloud, ArrowRight, X, Loader2, ExternalLink, Copy, ShieldCheck, AlertCircle } from "lucide-svelte";
    import { marked } from "marked";
    import DOMPurify from "dompurify";
    import { focusTrap } from "$lib/utils/focusTrap";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { updaterStore } from "$lib/stores/updater.svelte";
    import { settings } from "$lib/stores/settings.svelte";

    import { releaseImages, releaseSections } from '$lib/utils/releasePresentation';
    const images = $derived(releaseImages(updaterStore.releaseNotes));
    const sections = $derived(releaseSections(updaterStore.releaseNotes));
    const notes = $derived(DOMPurify.sanitize(marked.parse(updaterStore.releaseNotes.replace(/^#\s+Luxmc[^\r\n]*(?:\r?\n)*/i, ""), { async: false }), { ALLOWED_TAGS: ["p", "ul", "ol", "li", "strong", "em", "code", "h2", "h3", "h4", "br"], ALLOWED_ATTR: [] }));
    function close() { if (!updaterStore.isUpdating) updaterStore.showModal = false; }
    onMount(() => {
        const timer = setTimeout(() => { if (settings.value.autoCheckUpdates) void updaterStore.check(false); }, 3000);
        const interval = setInterval(() => { if (settings.value.autoCheckUpdates) void updaterStore.check(false); }, 1800000);
        return () => { clearTimeout(timer); clearInterval(interval); };
    });
</script>

{#if updaterStore.showModal}
    <div class="fixed inset-0 z-[100] flex items-center justify-center bg-bg-overlay/75 p-4 backdrop-blur-sm" role="dialog" aria-modal="true" aria-labelledby="update-title" use:focusTrap tabindex="-1" onkeydown={(event) => { if (event.key === "Escape") { event.stopPropagation(); close(); } }} onclick={(event) => { if (event.target === event.currentTarget) close(); }} transition:fade={{ duration: settings.value.animations ? 180 : 0 }}>
        <section class="flex max-h-[90dvh] w-full max-w-2xl flex-col overflow-hidden rounded-3xl border border-border bg-bg-elevated shadow-elevated" in:scale={{ start: 0.98, easing: quintOut, duration: settings.value.animations ? 200 : 0 }}>
            <header class="flex items-start gap-4 border-b border-border p-6">
                <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl border border-brand-400/25 bg-brand-500/10 text-brand-400">{#if updaterStore.isUpdating}<Loader2 class="h-6 w-6 animate-spin" />{:else}<DownloadCloud class="h-6 w-6" />{/if}</div>
                <div class="min-w-0 flex-1"><p class="text-[10px] font-semibold uppercase tracking-widest text-fg-muted">{uiText("updateDesign.label")}</p><h2 id="update-title" class="mt-1 font-sans text-xl font-bold tracking-tight text-fg">{updaterStore.isUpdating ? uiText("ui.4977706263fc4df6") : uiText("updateDesign.title")}</h2><p class="mt-2 text-sm leading-relaxed text-fg-muted">{uiText("updateDesign.subtitle")}</p></div>
                {#if !updaterStore.isUpdating}<button type="button" class={button({variant:"ghost",size:"icon"})} aria-label={uiText("common.close")} onclick={close}><X class="h-5 w-5" /></button>{/if}
            </header>
            <div class="min-h-0 space-y-5 overflow-y-auto p-6">
                <div class="grid grid-cols-[1fr_auto_1fr] items-center gap-3">
                    <div class="rounded-2xl border border-border bg-bg-subtle p-4"><p class="text-[10px] font-semibold uppercase tracking-wider text-fg-muted">{uiText("ui.078d1fdbabd0769e")}</p><p class="mt-1 text-lg font-semibold text-fg">v{updaterStore.currentVersion}</p></div>
                    <ArrowRight class="h-4 w-4 text-fg-muted" />
                    <div class="rounded-2xl border border-brand-400/25 bg-brand-500/10 p-4"><p class="text-[10px] font-semibold uppercase tracking-wider text-brand-400">{uiText("ui.f79e00e9eecb4a42")}</p><p class="mt-1 text-lg font-semibold text-brand-400">v{updaterStore.latestVersion}</p></div>
                </div>
                {#if updaterStore.isUpdating}
                    <div role="status" aria-live="polite" class="space-y-3 rounded-2xl border border-border bg-bg-subtle p-4">
                        <div class="flex items-center justify-between gap-3 text-sm"><span class="text-fg">{updaterStore.statusText}</span><span class="font-semibold tabular-nums text-brand-400">{updaterStore.progressPercent}%</span></div>
                        <div role="progressbar" aria-label={uiText("updateDesign.label")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={updaterStore.progressPercent} class="h-2 overflow-hidden rounded-full bg-border"><div class="h-full rounded-full bg-brand-500 transition-[width] duration-200" style:width={`${updaterStore.progressPercent}%`}></div></div>
                        <p class="text-xs tabular-nums text-fg-muted">{(updaterStore.transferredBytes / 1048576).toFixed(1)} MB{updaterStore.totalBytes > 0 ? ` / ${(updaterStore.totalBytes / 1048576).toFixed(1)} MB` : ""}</p>
                    </div>
                {:else}
                    {#if updaterStore.updateError}<div role="alert" class="flex gap-3 rounded-2xl border border-danger/30 bg-danger/10 p-4"><AlertCircle class="h-5 w-5 shrink-0 text-danger" /><div><p class="text-sm font-semibold text-danger">{uiText("ui.7cc311f43f53578b")}</p><p class="mt-1 break-words text-sm text-fg-muted">{updaterStore.updateError}</p></div></div>{/if}
                    {#if images.length}<div class="grid gap-3 sm:grid-cols-2" aria-label={uiText('workshop.preview')}>{#each images as image}<img src={image} alt={uiText('workshop.preview')} loading="lazy" referrerpolicy="no-referrer" class="max-h-56 w-full rounded-2xl border border-border object-contain" />{/each}</div>{/if}
                    {#if sections.length}<div class="grid gap-3 sm:grid-cols-2">{#each sections as section}<section class="rounded-2xl border border-border bg-brand-500/5 p-4"><h3 class="text-xs font-semibold text-brand-400">{section.title || uiText('workshop.summary')}</h3><ul class="mt-3 space-y-2">{#each section.items as item}<li class="flex gap-2 text-xs leading-relaxed text-fg-muted"><span class="mt-1.5 h-1 w-1 shrink-0 rounded-full bg-brand-400"></span>{item}</li>{/each}</ul></section>{/each}</div>{/if}
                    <section aria-label={uiText("ui.8b4b6e51a3a788e8")}><h3 class="mb-3 text-sm font-semibold text-fg">{uiText("ui.8b4b6e51a3a788e8")}</h3><div class="update-changelog max-h-[34dvh] overflow-y-auto rounded-2xl border border-border bg-bg-subtle p-4 font-sans text-sm leading-relaxed text-fg-muted">{@html notes}</div></section>
                    {#if updaterStore.terminalCommand}<div class="rounded-2xl border border-border p-4"><p class="text-xs font-semibold text-fg-muted">{uiText("ui.dc297a8bbc37b86b")}</p><code class="mt-2 block break-all text-xs text-fg">{updaterStore.terminalCommand}</code><button type="button" class={button({variant:"ghost",size:"sm",class:"mt-3"})} onclick={() => navigator.clipboard.writeText(updaterStore.terminalCommand)}><Copy class="h-4 w-4" />{uiText("ui.ffd3736638c756e3")}</button></div>{/if}
                {/if}
                <p class="flex items-start gap-2 text-xs leading-relaxed text-fg-muted"><ShieldCheck class="mt-0.5 h-4 w-4 shrink-0 text-brand-400" />{uiText("updateDesign.preserve")}</p>
            </div>
            {#if !updaterStore.isUpdating}<footer class="flex flex-wrap items-center justify-between gap-3 border-t border-border bg-bg-subtle/50 px-6 py-4"><button type="button" class={button({variant:"ghost",size:"sm"})} onclick={close}>{uiText("ui.060ea5eae9c1ff2f")}</button><div class="flex flex-wrap gap-2"><button type="button" class={button({variant:"secondary",size:"sm"})} onclick={() => openUrl(updaterStore.releaseUrl || "https://github.com/predabr/luxmc/releases/latest")}><ExternalLink class="h-4 w-4" />GitHub</button>{#if updaterStore.downloadUrl}<button type="button" class={button({variant:"primary",size:"sm"})} onclick={() => updaterStore.startUpdate()}><DownloadCloud class="h-4 w-4" />{updaterStore.updateError ? uiText("ui.45824b20097cc608") : uiText("updateDesign.install")}</button>{/if}</div></footer>{/if}
        </section>
    </div>
{/if}

<style>
    .update-changelog :global(p + p), .update-changelog :global(p + ul), .update-changelog :global(ul + p), .update-changelog :global(h2), .update-changelog :global(h3) { margin-top: 0.9rem; }
    .update-changelog :global(ul), .update-changelog :global(ol) { padding-left: 1.2rem; }
    .update-changelog :global(ul) { list-style: disc; }
    .update-changelog :global(ol) { list-style: decimal; }
    .update-changelog :global(li + li) { margin-top: 0.65rem; }
    .update-changelog :global(strong), .update-changelog :global(h2), .update-changelog :global(h3), .update-changelog :global(h4) { color: rgb(var(--fg)); font-weight: 600; }
</style>
