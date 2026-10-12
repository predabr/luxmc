<script lang="ts">
    import { encodeProfileImage } from '$lib/utils/profileImages';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import { Crop, RotateCcw, Check } from 'lucide-svelte';
    let { image, kind, onapply, oncancel }: { image: ImageBitmap; kind: 'portrait' | 'banner'; onapply: (value: string) => void; oncancel: () => void } = $props();
    let canvas = $state<HTMLCanvasElement>();
    let horizontal = $state(50), vertical = $state(50), zoom = $state(1);
    let error = $state('');
    $effect(() => {
        if (!canvas) return;
        const width = kind === 'portrait' ? 320 : 1280, height = kind === 'portrait' ? 320 : 384;
        canvas.width = width; canvas.height = height;
        const scale = Math.max(width / image.width, height / image.height) * zoom;
        const cropWidth = width / scale, cropHeight = height / scale;
        const context = canvas.getContext('2d');
        context?.clearRect(0, 0, width, height);
        context?.drawImage(image, (image.width - cropWidth) * horizontal / 100, (image.height - cropHeight) * vertical / 100, cropWidth, cropHeight, 0, 0, width, height);
    });
    function apply() {
        if (!canvas) return;
        try { onapply(encodeProfileImage(canvas)); }
        catch { error = t('publicProfile.imageLimit'); }
    }
</script>

<section class="space-y-4 rounded-2xl border border-brand-400/25 bg-brand-500/5 p-4" aria-label={t('profileStudio.cropTitle')}>
    <div class="flex items-center gap-2 text-sm font-semibold"><Crop class="h-4 w-4 text-brand-400" />{t('profileStudio.cropTitle')}</div>
    <div class="overflow-hidden rounded-xl border border-border bg-bg-overlay {kind === 'portrait' ? 'mx-auto max-w-56' : ''}"><canvas bind:this={canvas} class="block h-auto w-full" aria-label={t('profileStudio.cropPreview')}></canvas></div>
    <div class="grid gap-3 sm:grid-cols-3">
        <label class="space-y-2 text-xs text-fg-muted"><span class="flex justify-between">{t('profileStudio.zoom')}<span>{zoom.toFixed(1)}×</span></span><input class="w-full accent-brand-500" type="range" min="1" max="3" step="0.05" bind:value={zoom} /></label>
        <label class="space-y-2 text-xs text-fg-muted"><span>{t('profileStudio.horizontal')}</span><input class="w-full accent-brand-500" type="range" min="0" max="100" step="1" bind:value={horizontal} /></label>
        <label class="space-y-2 text-xs text-fg-muted"><span>{t('profileStudio.vertical')}</span><input class="w-full accent-brand-500" type="range" min="0" max="100" step="1" bind:value={vertical} /></label>
    </div>
    {#if error}<p role="alert" class="text-xs text-danger">{error}</p>{/if}
    <div class="flex flex-wrap gap-2"><button type="button" class={button({variant:'ghost',size:'sm'})} onclick={() => {horizontal=50;vertical=50;zoom=1;}}><RotateCcw class="h-3.5 w-3.5" />{t('profileStudio.center')}</button><button type="button" class={button({variant:'secondary',size:'sm',class:'ml-auto'})} onclick={oncancel}>{t('common.cancel')}</button><button type="button" class={button({variant:'primary',size:'sm'})} onclick={apply}><Check class="h-3.5 w-3.5" />{t('profileStudio.applyCrop')}</button></div>
</section>
