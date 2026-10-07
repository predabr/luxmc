<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { ChevronDown, Filter, Box, Cpu, Palette, Sparkles, HardDrive, Globe, Flame } from "lucide-svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { onMount } from "svelte";
	import { versionsList } from "$lib/api/launch";

	let {
		selectedSource = $bindable("all"),
		selectedType = $bindable("Modpack"),
		selectedVersion = $bindable("Qualquer Versão"),
		selectedLoader = $bindable(null),
		selectedCategory = $bindable(null)
	}: {
		selectedSource: "all" | "modrinth" | "curseforge";
		selectedType: string;
		selectedVersion: string;
		selectedLoader: string | null;
		selectedCategory: string | null;
	} = $props();

	const modLoaders = [
		{ name: "Fabric", color: "text-cyan-400 border-cyan-500/30 bg-cyan-500/10" },
		{ name: "Quilt", color: "text-purple-400 border-purple-500/30 bg-purple-500/10" },
		{ name: "Forge", color: "text-amber-300 border-amber-500/30 bg-amber-500/15" },
		{ name: "NeoForge", color: "text-orange-400 border-orange-500/30 bg-orange-500/10" },
	];

	const categories = [
		"Adventure", "Challenging", "Combat", "Kitchen-Sink",
		"Lightweight", "Magic", "Multiplayer", "Optimization",
		"Quests", "Technology"
	];

	let releaseVersions = $state<string[]>([]);
	const mcVersions = $derived(["Qualquer Versão", ...new Set([
		...(selectedVersion !== "Qualquer Versão" ? [selectedVersion] : []),
		...profiles.list.map(profile => profile.mcVersion),
		...releaseVersions
	])]);
	let versionsUnavailable = $state(false);
	onMount(() => {
		let disposed = false;
		void versionsList().then(manifest => {
			if (!disposed) releaseVersions = manifest.versions.filter(version => version.versionType === "release").map(version => version.id);
		}).catch(() => { if (!disposed) versionsUnavailable = true; });
		return () => { disposed = true; };
	});

	const contentTypeItems = $derived([
		{ id: "Modpack", label: uiText("ui.ff74891e9da6c4d7"), icon: Box },
		{ id: "Mod", label: uiText("ui.2fe83d132e3db243"), icon: Cpu },
		{ id: "Resource Pack", label: uiText("ui.7ca53deac08c5017"), icon: Palette },
		{ id: "Shader", label: uiText("ui.c2385aa66c95026c"), icon: Sparkles },
		{ id: "Data Pack", label: uiText("ui.37db17c75522082f"), icon: HardDrive },
		{ id: "World", label: uiText("ui.78ae647dc5544d22"), icon: Globe }
	]);
</script>

<aside class="w-full xl:w-72 shrink-0 self-start bg-bg-elevated border border-border rounded-2xl p-5 font-sans flex flex-col justify-between shadow-soft">
	<div class="space-y-5">
		<div class="flex items-center gap-2 text-xs font-bold text-fg uppercase tracking-wider">
			<Filter class="w-3.5 h-3.5 text-success" /> {uiText("ui.57ac13ce3d5bedff")}
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("ui.4b69288df9bb8f94")}</div>
			<div class="grid grid-cols-2 bg-bg/40 p-1 rounded-xl border border-fg/5 gap-1">
				<button
					type="button"
					class="col-span-2 py-2 px-2 rounded-xl text-xs font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex items-center justify-center {selectedSource === 'all' ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/50 hover:text-fg'}"
					onclick={() => selectedSource = 'all'}
				>
					{uiText("mods.categoryAll")}
				</button>
				<button
					type="button"
					class="py-2 px-2 rounded-xl text-xs font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex items-center justify-center gap-1.5 {selectedSource === 'modrinth' ? 'bg-success/15 text-success shadow-sm border border-success/30' : 'text-fg/50 hover:text-fg'}"
					onclick={() => selectedSource = 'modrinth'}
				>
					<span class="text-success font-black text-xs">m</span> Modrinth
				</button>
				<button
					type="button"
					class="py-2 px-2 rounded-xl text-xs font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex items-center justify-center gap-1.5 {selectedSource === 'curseforge' ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/50 hover:text-fg'}"
					onclick={() => selectedSource = 'curseforge'}
				>
					<Flame class="w-3.5 h-3.5 text-orange-400" /> CurseForge
				</button>
			</div>
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("ui.931d062a37647b8d")}</div>
			<div class="grid grid-cols-2 gap-2">
				{#each contentTypeItems as item}
					<button
						type="button"
						class="px-2.5 py-2 rounded-xl text-xs font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex items-center gap-2 {selectedType === item.id ? 'bg-success text-bg font-black shadow-md' : 'bg-bg-elevated text-fg/60 border border-fg/5 hover:bg-bg-subtle hover:text-fg'}"
						aria-pressed={selectedType === item.id}
						onclick={() => selectedType = item.id}
					>
						<item.icon class="w-3.5 h-3.5 shrink-0" />
						<span class="text-left leading-snug">{item.label}</span>
					</button>
				{/each}
			</div>
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("ui.5df9df4db410ba86")}</div>
			<div class="relative">
				<select
					bind:value={selectedVersion}
					aria-label={uiText("instances.version")}
					class="w-full bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs font-semibold text-fg focus:outline-none focus:border-brand-500 appearance-none cursor-pointer pr-8"
				>
					{#each mcVersions as ver}
						<option value={ver}>{ver === "Qualquer Versão" ? uiText("instances.allVersions") : ver}</option>
					{/each}
				</select>
				<ChevronDown class="absolute right-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-fg/40 pointer-events-none" />
			</div>
			{#if versionsUnavailable}<p class="mt-2 text-xs text-fg-muted">{uiText("ui.939284277cae2162")}</p>{/if}
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("ui.4ce6e1a0766463a1")}</div>
			<div class="grid grid-cols-2 gap-2">
				{#each modLoaders as loader}
					<button
						type="button"
						aria-pressed={selectedLoader?.toLowerCase() === loader.name.toLowerCase()}
						class="px-2.5 py-2 rounded-xl text-xs font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer border flex items-center gap-1.5 {selectedLoader?.toLowerCase() === loader.name.toLowerCase() ? `${loader.color} font-bold shadow-sm` : 'bg-bg-elevated text-fg/60 border-fg/5 hover:border-fg/20 hover:text-fg'}"
						onclick={() => selectedLoader = selectedLoader?.toLowerCase() === loader.name.toLowerCase() ? null : loader.name.toLowerCase()}
					>
						<span class="w-1.5 h-1.5 rounded-full bg-current"></span>
						<span class="truncate">{loader.name}</span>
					</button>
				{/each}
			</div>
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("ui.3adb5537c59ac03f")}</div>
			<div class="flex flex-wrap gap-1.5 max-h-48 overflow-y-auto custom-scrollbar pr-1">
				{#each categories as cat}
					<button
						type="button"
						class="px-2.5 py-1 rounded-full text-[11px] font-medium transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {selectedCategory === cat ? 'bg-success/20 text-success border border-success/40 font-bold' : 'bg-bg-elevated text-fg/50 border border-fg/5 hover:border-fg/20 hover:text-fg'}"
						onclick={() => selectedCategory = selectedCategory === cat ? null : cat}
					>
						{cat}
					</button>
				{/each}
			</div>
		</div>
	</div>

	<p class="mt-5 border-t border-border pt-4 text-xs leading-relaxed text-fg-muted">{uiText("ui.c7984fbe61202b7f")}</p>
</aside>
