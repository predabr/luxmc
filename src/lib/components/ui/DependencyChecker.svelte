<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { PackageCheck, AlertTriangle, Download, ChevronDown, ChevronUp, Loader2 } from "lucide-svelte";
	import { modsCheckMissingDeps, modsInstallMissingDeps } from "$lib/api/java";
	import type { MissingDep } from "$lib/api/types";
	import { toast } from "$lib/stores/toasts.svelte";

	type Props = {
		profileId: string;
		mcVersion: string;
		loader: string;
		onInstalled?: () => void;
	};

	let { profileId, mcVersion, loader, onInstalled }: Props = $props();

	let checking = $state(false);
	let installing = $state(false);
	let checked = $state(false);
	let missing = $state<MissingDep[]>([]);
	let expanded = $state(false);

	async function check() {
		checking = true;
		try {
			const result = await modsCheckMissingDeps(profileId, mcVersion, loader);
			missing = result.missing;
			checked = true;
			expanded = result.missing.length > 0;
		} catch {
			toast(uiText("ui.78493f8c2d0378e4"), "error");
		} finally {
			checking = false;
		}
	}

	async function installAll() {
		if (missing.length === 0) return;
		installing = true;
		try {
			const ids = missing.map((d) => d.projectId);
			const count = await modsInstallMissingDeps(profileId, mcVersion, loader, ids);
			toast(uiText("ui.0ff2c7badb0c731d", {arg0: (count)}), "success");
			missing = [];
			onInstalled?.();
		} catch (e) {
			toast(uiText("ui.8d1ee235e8a8c05f", {arg0: (e)}), "error");
		} finally {
			installing = false;
		}
	}
</script>

<div class="rounded-2xl border overflow-hidden transition-[color,background-color,border-color,box-shadow,transform,opacity] {checked && missing.length > 0 ? 'border-warning/30 bg-warning/5' : checked ? 'border-emerald-500/20 bg-emerald-500/5' : 'border-fg/5 bg-bg-subtle'}">
	<button
		type="button"
		onclick={() => { if (!checked) { check(); } else { expanded = !expanded; } }}
		class={launcherButton({ variant: "ghost", size: "lg", class: "w-full flex items-center justify-between gap-3" })}
	>
		<div class="flex items-center gap-3">
			{#if checking}
				<Loader2 class="w-4 h-4 text-brand-400 animate-spin" />
				<span class="text-xs font-bold text-fg/70">{uiText("ui.7fbef0cca076a951")}</span>
			{:else if checked && missing.length === 0}
				<PackageCheck class="w-4 h-4 text-emerald-400" />
				<span class="text-xs font-bold text-emerald-400">{uiText("ui.4dfe0abf2163132a")}</span>
			{:else if checked && missing.length > 0}
				<AlertTriangle class="w-4 h-4 text-warning" />
				<span class="text-xs font-bold text-warning">{missing.length} {uiText("ui.d17c6d73091795d2")}</span>
			{:else}
				<PackageCheck class="w-4 h-4 text-fg/40" />
				<span class="text-xs font-medium text-fg/50">{uiText("ui.27f65aaf2b21e7dc")}</span>
			{/if}
		</div>
		{#if checked && missing.length > 0}
			{#if expanded}
				<ChevronUp class="w-3.5 h-3.5 text-fg/30" />
			{:else}
				<ChevronDown class="w-3.5 h-3.5 text-fg/30" />
			{/if}
		{/if}
	</button>

	{#if expanded && missing.length > 0}
		<div class="border-t border-fg/5 px-4 py-3 flex flex-col gap-3">
			<div class="flex flex-col gap-1.5">
				{#each missing as dep}
					<div class="flex items-center justify-between text-[11px] py-1">
						<span class="font-bold text-fg/80">{dep.name}</span>
						<span class="text-fg/30 font-mono">{dep.slug}</span>
					</div>
				{/each}
			</div>
			<button
				type="button"
				disabled={installing}
				onclick={installAll}
				class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center justify-center gap-2 w-full disabled:opacity-60" })}
			>
				{#if installing}
					<Loader2 class="w-3.5 h-3.5 animate-spin" />
					{uiText("ui.b1e8e68efcbda240")}
				{:else}
					<Download class="w-3.5 h-3.5" />
					{uiText("mods.install")} {missing.length} {uiText("ui.7d8663a85b884e8c")}
				{/if}
			</button>
		</div>
	{/if}
</div>
