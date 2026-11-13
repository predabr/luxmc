<script lang="ts">
	import { Box, Cpu, Palette, Sparkles, HardDrive, Globe } from "lucide-svelte";
	import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";

	let { selectedType = $bindable("Modpack") }: { selectedType: string } = $props();
	const types = $derived([
		{ id: "Modpack", label: uiText("ui.ff74891e9da6c4d7"), icon: Box },
		{ id: "Mod", label: uiText("ui.2fe83d132e3db243"), icon: Cpu },
		{ id: "Resource Pack", label: uiText("ui.7ca53deac08c5017"), icon: Palette },
		{ id: "Shader", label: uiText("ui.c2385aa66c95026c"), icon: Sparkles },
		{ id: "Data Pack", label: uiText("ui.37db17c75522082f"), icon: HardDrive },
		{ id: "World", label: uiText("ui.78ae647dc5544d22"), icon: Globe }
	]);
</script>

<nav class="catalog-navigation" aria-label={uiText("ui.931d062a37647b8d")}>
	{#each types as type (type.id)}
		<button type="button" class="catalog-type" aria-current={selectedType === type.id ? "page" : undefined} onclick={() => selectedType = type.id}>
			<type.icon class="type-icon h-4 w-4" aria-hidden="true" />
			<span>{type.label}</span>
		</button>
	{/each}
</nav>

<style>
	.catalog-navigation { display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); gap: 10px; }
	.catalog-type {
		display: flex; align-items: center; gap: 10px; min-width: 0; min-height: 38px;
		padding: 0 12px; border: 1px solid rgb(var(--fg) / .08); border-radius: var(--radius-sm);
		background: rgb(var(--fg) / .05); color: rgb(var(--fg)); font-size: 12px; font-weight: 600; text-align: left;
		transition: color 140ms ease, background-color 140ms ease, border-color 140ms ease;
	}
	.catalog-type span { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.catalog-type:hover { background: rgb(var(--fg) / .09); border-color: rgb(var(--fg) / .1); color: rgb(var(--fg)); }
	.catalog-type[aria-current="page"] {
		border-color: transparent; background: rgb(var(--brand-500) / .12);
		color: rgb(var(--brand-action));
		box-shadow: none;
	}
	:global(.type-icon) { flex: none; opacity: .8; }
	.catalog-type[aria-current="page"] :global(.type-icon) { opacity: 1; }
	.catalog-type:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 2px; }
	:global(html.has-custom-wallpaper) .catalog-type { background-color: rgb(var(--bg-elevated) / max(.84, var(--interface-opacity, .84))); backdrop-filter: blur(12px); }
	@media (max-width: 760px) {
		.catalog-navigation { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 9px; }
		.catalog-type { justify-content: flex-start; min-height: 42px; padding-inline: 12px; }
	}
	@media (min-width: 761px) and (max-width: 1000px) { .catalog-navigation { grid-template-columns: repeat(3, minmax(0, 1fr)); } }
	@media (prefers-reduced-motion: reduce) { .catalog-type { transition: none; } }
</style>
