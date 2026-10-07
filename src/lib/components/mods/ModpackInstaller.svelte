<script lang="ts">
import { translateUi as uiText, currentUiLocale } from "$lib/i18n/useTranslation.svelte";
    import { focusTrap } from "$lib/utils/focusTrap";
    import { button } from "$lib/components/ui/button";
    import LazyImage from "$lib/components/ui/LazyImage.svelte";
    import SourceBadge from "./SourceBadge.svelte";
    import { PackagePlus, Download, Loader2, X, Cpu, Check, FolderCheck } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import type { ModSearchResultItem } from "$lib/api";
    let { modpack, instanceName = $bindable(""), ramMb = $bindable(4096), isInstalling = false, progressText = "", progressPercent = 0, onConfirm, onClose, onCancel, cancelling = false }: {
        modpack: ModSearchResultItem; instanceName?: string; ramMb?: number; isInstalling?: boolean; progressText?: string; progressPercent?: number; onConfirm: () => void; onClose: () => void; onCancel: () => void; cancelling?: boolean;
    } = $props();
    const percent = $derived(Math.max(0, Math.min(100, progressPercent)));
    const indeterminate = $derived(progressPercent < 0);
    const status = $derived(progressText.toLowerCase());
    const stage = $derived(/finaliz|conclu|verific/.test(status) ? 3 : /mods|extraindo|instalando|configurando/.test(status) && !/buscando/.test(status) ? 2 : /baixando pacote|download|baixando arquivo/.test(status) ? 1 : 0);
    const stages = $derived([{ label: uiText("ui.edbadf0d55e84195"), icon: PackagePlus }, { label: uiText("ui.d7e616e73950c566"), icon: Download }, { label: uiText("ui.a52fd8beef3c087f"), icon: FolderCheck }, { label: uiText("ui.80a5037a33b80200"), icon: Check }]);
    const downloads = $derived(new Intl.NumberFormat(currentUiLocale(), { notation: "compact" }).format(modpack.downloads));
</script>

<div class="fixed inset-0 z-[999999] flex items-center justify-center bg-bg-overlay/90 p-4" transition:fade={{ duration: 140 }}>
    <button type="button" class="absolute inset-0 cursor-default" tabindex="-1" aria-label={uiText("ui.2de7443df5f6a877")} disabled={isInstalling} onclick={onClose}></button>
    <div role="dialog" aria-modal="true" aria-label={uiText("ui.ae9010b55c74629b")} tabindex="-1" use:focusTrap class="relative flex max-h-[92vh] w-full max-w-2xl flex-col overflow-hidden font-sans rounded-2xl border border-border-strong bg-bg-elevated shadow-elevated" onkeydown={(event) => { if (event.key === "Escape" && !isInstalling) { event.stopPropagation(); onClose(); } }}>
        <header class="relative shrink-0 overflow-hidden border-b border-border bg-bg-subtle/50 p-6">

            <div class="relative flex items-start gap-4">
                <div class="h-20 w-20 shrink-0 overflow-hidden rounded-2xl border border-border-strong bg-bg-subtle p-1"><LazyImage src={modpack.iconUrl || '/grass_block.png'} alt="" loading="eager" class="object-contain" /></div>
                <div class="min-w-0 flex-1"><div class="mb-2 flex items-center gap-2"><SourceBadge source={modpack.source} /><span class="text-[10px] font-semibold uppercase tracking-widest text-brand-400">{uiText("ui.456abc53629fc5af")}</span></div><h2 class="text-xl font-semibold leading-snug text-fg break-words">{modpack.title}</h2><p class="mt-1 text-xs text-fg-muted">{modpack.author || modpack.slug} · {downloads} {uiText("ui.4eed41d911d75723")}</p></div>
                {#if !isInstalling}<button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={uiText("ui.2de7443df5f6a877")} onclick={onClose}><X class="h-5 w-5" /></button>{/if}
            </div>
            <p class="relative mt-4 line-clamp-2 text-xs leading-relaxed text-fg-muted">{modpack.description}</p>
        </header>
        <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain custom-scrollbar p-6 space-y-6">
            {#if !isInstalling}
                <div><h3 class="text-base font-semibold text-fg">{uiText("ui.d271c5645474da6d")}</h3><p class="mt-1 text-xs text-fg-muted">{uiText("ui.dd01b055a7d23328")}</p></div>
                <label class="block space-y-2 text-xs font-semibold text-fg" for="modpack-inst-name"><span>{uiText("instances.namePlaceholder")}</span><input id="modpack-inst-name" type="text" bind:value={instanceName} placeholder={modpack.title} class="w-full rounded-xl border border-border-strong bg-bg-subtle px-4 py-3 text-sm text-fg focus:border-brand-400 focus:outline-none" /></label>
                <div class="space-y-4 rounded-2xl border border-border bg-bg-subtle/50 p-4"><div class="flex items-center gap-3"><Cpu class="h-5 w-5 text-brand-400" /><div class="flex-1"><label for="modpack-ram" class="text-xs font-semibold text-fg">{uiText("ui.27d363405331c831")}</label><p class="mt-1 text-xs text-fg-muted">{uiText("ui.7e24067202e46e23")}</p></div><output for="modpack-ram" class="text-lg font-bold tabular-nums text-brand-400">{ramMb / 1024} <span class="text-xs">GB</span></output></div>
                    <input id="modpack-ram" type="range" min="1024" max="16384" step="512" bind:value={ramMb} class="w-full accent-brand-500" />
                    <div class="flex flex-wrap items-center gap-2">{#each [4096, 6144, 8192, 12288] as value}<button type="button" aria-pressed={ramMb === value} class={button({ variant: ramMb === value ? 'ghostBrand' : 'secondary', size: 'sm' })} onclick={() => ramMb = value}>{value / 1024} GB</button>{/each}<span class="ml-auto text-[10px] text-fg-muted">{uiText("ui.3410fe309bb2f912")}</span></div>
                </div>
            {:else}
                <div class="flex flex-wrap items-center gap-3 text-xs text-fg-muted"><span class="font-semibold text-fg">{instanceName}</span><span class="ml-auto flex items-center gap-2"><Cpu class="h-4 w-4" />{ramMb / 1024} {uiText("ui.48e64b8ad7aed3b9")}</span></div>
                <ol class="grid grid-cols-4 gap-2" aria-label={uiText("ui.19108679a75dbdf5")}>{#each stages as step, index}<li class="flex flex-col items-center gap-2 text-center text-[10px] {index <= stage ? 'text-brand-400' : 'text-fg-subtle'}"><span class="flex h-9 w-9 items-center justify-center rounded-xl border {index <= stage ? 'border-brand-500/30 bg-brand-500/10' : 'border-border bg-bg-subtle'}">{#if index < stage}<Check class="h-4 w-4" />{:else if index === stage}<Loader2 class="h-4 w-4 animate-spin" />{:else}<step.icon class="h-4 w-4" />{/if}</span>{step.label}</li>{/each}</ol>
                <div class="space-y-3 rounded-xl border border-brand-500/20 bg-brand-500/5 p-5"><div class="flex items-center justify-between gap-3"><h3 class="text-sm font-semibold text-fg">{cancelling ? uiText("ui.6ae6319c82afbb57") : stages[stage].label}</h3>{#if !indeterminate}<span class="text-lg font-bold tabular-nums text-brand-400">{Math.round(percent)}%</span>{/if}</div><div role="progressbar" aria-label={uiText("ui.84fb8a36577543fb")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={indeterminate ? undefined : percent} class="h-2 overflow-hidden rounded-full bg-fg/10"><div class="installer-progress-fill h-full rounded-full bg-brand-500 {indeterminate ? 'animate-pulse' : ''}" style:width={indeterminate ? '100%' : `${percent}%`}></div></div><p class="break-words text-xs leading-relaxed text-fg-muted" role="status">{progressText || uiText("ui.27fb477239f6b217")}</p></div>
            {/if}
        </div>
        <footer class="shrink-0 flex items-center justify-between gap-3 border-t border-border bg-bg-subtle px-6 py-4"><button type="button" class={button({ variant: 'secondary' })} disabled={isInstalling && cancelling} onclick={isInstalling ? onCancel : onClose}>{cancelling ? uiText("ui.6ae6319c82afbb57") : isInstalling ? uiText("ui.bf526874f31d132e") : uiText("common.cancel")}</button>{#if !isInstalling}<button type="button" class={button({ variant: 'primary', size: 'lg' })} disabled={!instanceName.trim()} onclick={onConfirm}><Download class="h-4 w-4" />{uiText("ui.ae9010b55c74629b")}</button>{:else}<span class="flex items-center gap-2 text-xs font-semibold text-fg-muted"><Loader2 class="h-4 w-4 animate-spin" />{cancelling ? uiText("ui.6ae6319c82afbb57") : uiText("ui.e767cd8c5098119b")}</span>{/if}</footer>
    </div>
</div>
