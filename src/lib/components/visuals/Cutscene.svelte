<script lang="ts">
    import { onMount } from "svelte";
    import { appState } from "$lib/stores/app.svelte";
    let { onComplete = () => {} }: { onComplete?: () => void } = $props();
    let leaving = $state(false);
    let finished = false;
    let closeTimer: ReturnType<typeof setTimeout>;
    const destination = $derived(appState.activeGameDetails?.name);
    function finish() {
        if (finished) return;
        finished = true;
        leaving = true;
        closeTimer = setTimeout(onComplete, 300);
    }
    onMount(() => {
        const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
        const timer = setTimeout(finish, reduced ? 150 : 2400);
        const key = (event: KeyboardEvent) => {
            if (["Escape", "Enter", " "].includes(event.key)) { event.preventDefault(); finish(); }
        };
        window.addEventListener("keydown", key);
        return () => { clearTimeout(timer); clearTimeout(closeTimer); window.removeEventListener("keydown", key); };
    });
</script>
<div class="intro fixed inset-0 z-[99999] overflow-hidden bg-bg text-fg transition-opacity duration-300" class:opacity-0={leaving} role="status" aria-label="Abrindo Luxmc">
    <img src="/bg_night.jpg" alt="" class="landscape absolute inset-0 h-full w-full object-cover opacity-40" />
    <div class="absolute inset-0 bg-gradient-to-t from-bg via-bg/50 to-bg/30"></div>
    <div class="absolute inset-x-0 top-0 h-16 bg-bg/90"></div>
    <div class="absolute inset-x-0 bottom-0 h-16 bg-bg/90"></div>
    <div class="absolute left-8 right-8 top-8 flex items-center justify-between text-[10px] uppercase tracking-[.25em] text-fg-muted">
        <span>Luxmc Launcher</span><span>Java Edition</span>
    </div>
    <div class="absolute inset-0 flex flex-col items-center justify-center px-8">
        <div class="portal mb-8 relative h-24 w-24 sm:h-32 sm:w-32">
            <div class="portal-frame absolute inset-0 rounded-[1.75rem] border border-brand-300/40 bg-brand-500/10"></div>
            <div class="absolute inset-3 rounded-2xl border border-brand-200/20 bg-bg/40"></div>
            <img src="/logo.png" alt="" class="emblem absolute inset-5 h-[calc(100%-2.5rem)] w-[calc(100%-2.5rem)] object-contain" />
        </div>
        <div class="wordmark text-center">
            <p class="text-[10px] font-semibold uppercase tracking-[.4em] text-brand-200">Explore. Construa. Compartilhe.</p>
            <h1 class="mt-5 text-6xl font-black tracking-[-.06em] sm:text-8xl">Seu próximo mundo<span class="text-brand-400">.</span></h1>
            <p class="mt-6 max-w-xl truncate px-6 text-sm text-fg/70">{destination || "Uma biblioteca de possibilidades. Um lugar para começar."}</p>
        </div>
        <div class="mt-10 h-px w-48 overflow-hidden bg-fg/10"><div class="light h-full w-full origin-left bg-brand-300"></div></div>
    </div>
    <div class="absolute bottom-6 inset-x-8 flex items-center justify-between text-[10px] uppercase tracking-[.2em] text-fg-muted">
        <span>Feito para jogar do seu jeito</span>
        <button class="rounded-lg border border-fg/15 px-4 py-2 hover:bg-fg/10 hover:text-fg focus-visible:outline-brand-400" onclick={finish}>Pular <span class="ml-2 text-fg/40">Esc</span></button>
    </div>
</div>
<style>
    .landscape { animation: landscape 2.7s cubic-bezier(.2,.7,.2,1) both; }
    .portal { animation: arrival 1s cubic-bezier(.2,.7,.2,1) both; }
    .portal-frame { animation: portal 1.6s cubic-bezier(.2,.7,.2,1) both; }
    .wordmark { animation: arrival 1s .2s cubic-bezier(.2,.7,.2,1) both; }
    .light { animation: reveal 2.3s cubic-bezier(.2,.7,.2,1) both; }
    @keyframes landscape { from { transform: scale(1.08); opacity: 0; } to { transform: scale(1); opacity: .4; } }
    @keyframes portal { from { transform: rotate(-30deg) scale(.7); opacity: 0; } to { transform: rotate(12deg) scale(1); opacity: 1; } }
    @keyframes arrival { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: translateY(0); } }
    @keyframes reveal { from { opacity: 0; transform: scaleX(0); } to { opacity: 1; transform: scaleX(1); } }
    @media (prefers-reduced-motion: reduce) { .landscape, .portal, .portal-frame, .wordmark, .light { animation: none; } }
</style>
