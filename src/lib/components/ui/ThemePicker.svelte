<script lang="ts">
	import { settings } from "$lib/stores/settings.svelte";
	import { Check } from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	type ThemePreset = {
		id: "cyan" | "emerald" | "violet" | "amber" | "rose";
		nameKey: "defaultCyan" | "emeraldForest" | "violetPulse" | "amberHearth" | "roseQuartz";
		preview: string[];
		accent: "cyan" | "emerald" | "violet" | "gold" | "rose";
	};

	const presets: ThemePreset[] = [
		{
			id: "cyan",
			nameKey: "defaultCyan",
			preview: ["rgb(20, 184, 166)", "rgb(13, 148, 136)", "rgb(94, 214, 198)"],
			accent: "cyan",
		},
		{
			id: "emerald",
			nameKey: "emeraldForest",
			preview: ["rgb(16, 185, 129)", "rgb(5, 150, 105)", "rgb(110, 231, 183)"],
			accent: "emerald",
		},
		{
			id: "violet",
			nameKey: "violetPulse",
			preview: ["rgb(139, 92, 246)", "rgb(124, 58, 237)", "rgb(196, 181, 253)"],
			accent: "violet",
		},
		{
			id: "amber",
			nameKey: "amberHearth",
			preview: ["rgb(245, 158, 11)", "rgb(217, 119, 6)", "rgb(252, 211, 77)"],
			accent: "gold",
		},
		{
			id: "rose",
			nameKey: "roseQuartz",
			preview: ["rgb(244, 63, 94)", "rgb(225, 29, 72)", "rgb(253, 164, 175)"],
			accent: "rose",
		},
	];

	function selectTheme(preset: ThemePreset) {
		const html = document.documentElement;
		html.classList.remove("accent-gold", "accent-cyan", "accent-emerald", "accent-rose", "accent-violet", "accent-orange", "accent-blue");
		html.classList.add(`accent-${preset.accent}`);
		settings.patch({ accentTheme: preset.accent, theme: "default-dark" });
	}
</script>

<div class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3">
	{#each presets as preset (preset.id)}
		<button
			type="button"
			class="group relative flex flex-col gap-2 rounded-lg border p-3 text-left transition-colors hover:border-brand-400"
			style="border-color: rgb(var(--border)); background: rgb(var(--bg-subtle));"
			onclick={() => selectTheme(preset)}
		>
			<div class="flex items-center gap-2">
				{#each preset.preview as color (color)}
					<span class="h-5 w-5 rounded-full" style="background: {color};"></span>
				{/each}
				<span class="ml-auto text-xs font-medium" style="color: rgb(var(--fg-muted));">
					{settings.value.accentTheme === preset.accent ? t("themePicker.active") : ""}
				</span>
			</div>
			<div class="flex items-center justify-between">
				<p class="text-sm font-medium" style="color: rgb(var(--fg));">{t(`themePicker.${preset.nameKey}`)}</p>
				{#if settings.value.accentTheme === preset.accent}
					<Check class="h-4 w-4" style="color: rgb(var(--brand-400));" />
				{/if}
			</div>
		</button>
	{/each}
</div>
