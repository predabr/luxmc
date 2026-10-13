<script lang="ts">
    import { onMount } from "svelte";
    import { Check, Loader2, Clock3 } from "lucide-svelte";
    import { appState } from "$lib/stores/app.svelte";
    import { settings } from "$lib/stores/settings.svelte";
    let { status }: { status: string } = $props();
    let now = $state(Date.now());
    const seconds = $derived(Math.max(0, Math.floor((now - appState.launchAttemptStartedAt) / 1000)));
    const elapsed = $derived(`${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`);
    const phase = $derived(["resolvingClasspath", "extractingNatives", "resolvingArgs", "validatingArgs"].includes(appState.launchStage) ? 1 : ["spawning", "waitingForWindow", "running"].includes(appState.launchStage) ? 2 : 0);
    const animated = $derived(settings.value.animations !== false);
    onMount(() => { const timer = setInterval(() => { now = Date.now(); }, 1000); return () => clearInterval(timer); });
</script>

<div class="launch-progress mt-5 rounded-2xl border border-border bg-bg-overlay/35 px-4 py-3" class:animated>
    <div class="flex items-center justify-between gap-4">
        <div class="flex min-w-0 items-center gap-3" role="status" aria-live="polite">
            <span class="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-brand-500/10 text-brand-400"><Loader2 class="launch-spinner h-4 w-4" /></span>
            <span class="text-sm font-medium text-fg">{status || "Preparando Minecraft…"}</span>
        </div>
        <span class="flex shrink-0 items-center gap-1.5 text-xs tabular-nums text-fg-muted"><Clock3 class="h-3.5 w-3.5" />{elapsed}</span>
    </div>
    <div class="mt-3 grid grid-cols-3 gap-2" aria-label="Etapas da inicialização">
        {#each ["Preparação", "Java e bibliotecas", "Abrindo o jogo"] as label, index}
            <div class="space-y-2" class:active={phase === index} class:complete={phase > index}>
                <div class="phase-track h-1 overflow-hidden rounded-full bg-fg/10"><div class="phase-fill h-full rounded-full bg-brand-400"></div></div>
                <span class="flex items-center gap-1.5 text-[11px] text-fg-muted">{#if phase > index}<Check class="h-3 w-3 text-brand-400" />{/if}{label}</span>
            </div>
        {/each}
    </div>
</div>

<style>
    .phase-fill { width: 0; }
    .complete .phase-fill { width: 100%; }
    .active .phase-fill { width: 35%; opacity: .8; }
    .active span { color: rgb(var(--fg)); }
    .animated .active .phase-fill { animation: phase-pulse 1.8s ease-in-out infinite; }
    .animated :global(.launch-spinner) { animation: launch-spin 1.2s linear infinite; }
    @keyframes phase-pulse { 0% { transform: translateX(-100%); } 100% { transform: translateX(390%); } }
    @keyframes launch-spin { to { transform: rotate(360deg); } }
    @media (prefers-reduced-motion: reduce) { .animated .active .phase-fill, .animated :global(.launch-spinner) { animation: none; } }
</style>
