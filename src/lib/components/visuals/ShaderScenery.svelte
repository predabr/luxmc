<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { scenery } from "$lib/visuals/scenery";
    let { scene = "cherry", compact = false, background = false, enabled = true }: { scene?: "cherry" | "forest" | "city"; compact?: boolean; background?: boolean; enabled?: boolean } = $props();
    const titles = $derived({ cherry: uiText("ui.a4709a3a0d646f76"), forest: uiText("ui.eab2f0faea9aff89"), city: uiText("ui.126c77a6db2b25d0") });
</script>

<div class="shader-scene" class:compact class:background use:scenery={enabled}>
    <div class="scene-photo"><img src={`/cinema/${scene}.webp`} alt="" loading={compact ? "lazy" : "eager"} decoding="async" /></div>
    <div class="scene-shade"></div>
    <div class="scene-caption"><span class="scene-eyebrow">{uiText("ui.04075ef9a124e240")}</span><strong>{titles[scene]}</strong></div>
    <a class="scene-credit" href="https://www.complementary.dev/shaders/" target="_blank" rel="noopener noreferrer">{uiText("ui.42328ef4c097e3ae")}</a>
</div>

<style>
    .shader-scene { position: relative; isolation: isolate; overflow: hidden; border-radius: 18px; height: clamp(150px, 18vw, 235px); margin: 20px 24px; border: 1px solid rgb(var(--fg) / .12); perspective: 1100px; background: rgb(var(--bg)); }
    .scene-photo { position: absolute; inset: -18px; transform: translate3d(calc(var(--scene-x, 0) * -18px), calc(var(--scene-y, 0) * -12px), 0) scale(1.04); transition: transform 650ms cubic-bezier(.2,.7,.2,1); }
    img { width: 100%; height: 100%; object-fit: cover; object-position: center 58%; animation: scenic-drift 28s ease-in-out infinite alternate; animation-play-state: paused; }
    .shader-scene:global([data-scene-active="true"]) img { animation-play-state: running; }
    .scene-shade { position: absolute; inset: 0; background: linear-gradient(90deg, rgb(var(--bg) / .85), rgb(var(--bg) / .16) 75%), linear-gradient(0deg, rgb(var(--bg) / .55), transparent 60%); }
    .scene-caption { position: absolute; left: 28px; top: 50%; transform: translate3d(calc(var(--scene-x, 0) * 8px), -50%, 20px); display: grid; gap: 12px; color: rgb(var(--fg)); max-width: 65%; }
    .scene-eyebrow { font-size: 10px; letter-spacing: .2em; opacity: .8; }
    strong { font-size: clamp(20px, 2.5vw, 34px); line-height: 1.12; letter-spacing: -.035em; }
    .scene-credit { position: absolute; bottom: 14px; right: 18px; font-size: 9px; color: rgb(var(--fg)); padding: 5px 8px; border-radius: 5px; background: rgb(var(--bg) / .8); }
    .scene-credit:focus-visible { outline: 2px solid rgb(var(--brand-500)); outline-offset: 3px; }
    .compact { height: 115px; margin-top: 30px; }
    .compact strong { font-size: 19px; }
    .background { position: absolute; inset: 0; height: 100%; margin: 0; border: 0; border-radius: 0; z-index: -1; }
    .background .scene-caption { display: none; }
    .background .scene-shade { background: rgb(var(--bg) / .65); }
    @keyframes scenic-drift { from { transform: scale(1); } to { transform: scale(1.07) translateX(-1%); } }
    @media (max-width: 700px) { .shader-scene { margin: 14px; } .scene-caption { left: 18px; } .scene-credit { font-size: 8px; right: 10px; bottom: 10px; } }
    @media (prefers-reduced-motion: reduce) { img { animation: none; } .scene-photo { transition: none; transform: none; } }
</style>
