<script lang="ts">
    import {onMount} from 'svelte';
    import {translateUi as uiText} from '$lib/i18n/useTranslation.svelte';
    import {button} from '$lib/components/ui/button';
    import {createLogoFracture, type LogoPhase} from '$lib/utils/logoFracture';

    let {onComplete = () => {}}: {onComplete?: () => void} = $props();
    let canvas = $state<HTMLCanvasElement | null>(null);
    let phase = $state<LogoPhase>('reveal');
    let rendered = $state(false);
    let leaving = $state(false);
    let finished = false;
    let disposed = false;
    let frame = 0;
    let closeTimer: ReturnType<typeof setTimeout> | undefined;
    let fallbackTimer: ReturnType<typeof setTimeout> | undefined;
    let simulation: Awaited<ReturnType<typeof createLogoFracture>> | undefined;
    function finish(immediate = false) {
        if (finished || disposed) return;
        finished = true; cancelAnimationFrame(frame); simulation?.destroy(); simulation = undefined;
        clearTimeout(fallbackTimer);
        if (immediate) onComplete();
        else { leaving = true; closeTimer = setTimeout(onComplete, 300); }
    }
    onMount(() => {
        const key = (event: KeyboardEvent) => { if (['Escape', 'Enter', ' '].includes(event.key)) { event.preventDefault(); finish(true); } };
        window.addEventListener('keydown', key);
        const image = new Image(); image.src = '/logo.png';
        fallbackTimer = setTimeout(() => finish(), 10000);
        void image.decode().then(async () => {
            if (disposed || finished || !canvas) return;
            const created = await createLogoFracture(canvas, image, value => phase = value);
            if (disposed || finished) { created.destroy(); return; }
            simulation = created; clearTimeout(fallbackTimer);
            let previous = performance.now(), elapsed = 0;
            const animate = (now: number) => {
                if (disposed || finished) return;
                const delta = Math.min(50, Math.max(0, now - previous)); previous = now;
                if (!document.hidden) { elapsed += delta; simulation?.render(elapsed, delta); rendered = true; }
                if (elapsed >= 5350) finish(); else frame = requestAnimationFrame(animate);
            };
            frame = requestAnimationFrame(animate);
        }).catch(() => { if (!disposed && !finished) { clearTimeout(fallbackTimer); fallbackTimer = setTimeout(() => finish(), 700); } });
        return () => { disposed = true; cancelAnimationFrame(frame); clearTimeout(closeTimer); clearTimeout(fallbackTimer); simulation?.destroy(); window.removeEventListener('keydown', key); };
    });
</script>

<div data-cutscene data-phase={phase} class="fixed inset-0 z-[99999] select-none overflow-hidden bg-bg text-fg transition-opacity duration-300" class:opacity-0={leaving} class:pointer-events-none={leaving} role="status" aria-label={uiText('ui.6b516b49d3d1f7a3')}>
    <canvas bind:this={canvas} class="absolute inset-0 h-full w-full" aria-hidden="true"></canvas>
    {#if !rendered}<img src="/logo.png" alt="Luxmc" fetchpriority="high" class="absolute left-1/2 top-1/2 w-72 max-w-[60vw] -translate-x-1/2 -translate-y-1/2 object-contain" />{/if}
    <button type="button" class={button({variant:'ghost',size:'sm',class:'absolute bottom-6 right-6'})} onclick={() => finish(true)}>{uiText('app.skip')} <span class="text-fg-muted">Esc</span></button>
</div>
