<script lang="ts">
    import { Box, Cpu, Palette, Sparkles, HardDrive, Globe } from 'lucide-svelte';
    import { translateUi as uiText } from '$lib/i18n/useTranslation.svelte';
    let { selectedType = $bindable('Modpack') }: { selectedType: string } = $props();
    const types = $derived([
        { id: 'Modpack', label: uiText('ui.ff74891e9da6c4d7'), icon: Box },
        { id: 'Mod', label: uiText('ui.2fe83d132e3db243'), icon: Cpu },
        { id: 'Resource Pack', label: uiText('ui.7ca53deac08c5017'), icon: Palette },
        { id: 'Shader', label: uiText('ui.c2385aa66c95026c'), icon: Sparkles },
        { id: 'Data Pack', label: uiText('ui.37db17c75522082f'), icon: HardDrive },
        { id: 'World', label: uiText('ui.78ae647dc5544d22'), icon: Globe }
    ]);
</script>

<nav class="catalog-navigation" aria-label={uiText('ui.931d062a37647b8d')}>
    {#each types as type}
        <button type="button" class="catalog-type" aria-pressed={selectedType === type.id} onclick={() => selectedType = type.id}>
            <type.icon class="h-4 w-4 shrink-0" /><span>{type.label}</span>
        </button>
    {/each}
</nav>

<style>
    .catalog-navigation { display: flex; flex-wrap: wrap; gap: 6px; padding: 0 0 10px; margin-bottom: 20px; border-bottom: 1px solid rgb(var(--border)); background: transparent; }
    .catalog-type { display: flex; align-items: center; justify-content: center; flex: 0 1 auto; gap: 8px; padding: 10px 16px; border-radius: 10px; border: 1px solid transparent; font-size: 12px; font-weight: 600; color: rgb(var(--fg-muted)); transition: color 180ms ease, background-color 180ms ease, border-color 180ms ease; }
    .catalog-type:hover { background: rgb(var(--fg) / .04); color: rgb(var(--fg)); }
    .catalog-type[aria-pressed="true"] { color: rgb(var(--brand-action)); background: rgb(var(--brand-500) / .12); border-color: rgb(var(--brand-400) / .25); box-shadow: inset 0 -2px 0 rgb(var(--brand-400) / .7); }
    .catalog-type:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 2px; }
    @media (max-width: 700px) { .catalog-type { flex-basis: 28%; padding-inline: 8px; font-size: 11px; } }
</style>
