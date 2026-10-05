<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { Download, PackagePlus, Layers } from "lucide-svelte";
	import type { ModVersion } from "$lib/api";

	let {
		versions = [],
		showAll = $bindable(false),
		contentType = "Mod",
		onInstall
	}: {
		versions: ModVersion[];
		showAll?: boolean;
		contentType?: string;
		onInstall: (versionId: string) => void;
	} = $props();

	function formatBytes(bytes: number): string {
		if (!bytes || bytes === 0) return "0 B";
		const k = 1024;
		const sizes = ["B", "KB", "MB", "GB"];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
	}

	const displayedVersions = $derived(showAll ? versions : versions.slice(0, 30));
	const isModpack = $derived(contentType === "Modpack");
</script>

{#if versions.length === 0}
	<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-12 text-center text-fg/40 text-xs">
		<Layers class="w-10 h-10 text-fg/20 mx-auto mb-3" />
		<p>{uiText("ui.9135dcfa3a4a2e7f")}</p>
	</div>
{:else}
	<div class="bg-bg-elevated border border-fg/5 rounded-3xl overflow-hidden shadow-md">
		<div class="divide-y divide-white/5">
			{#each displayedVersions as ver (ver.id)}
				{@const file = ver.files[0]}
				<div class="p-4 flex items-center justify-between hover:bg-fg/[0.02] transition-colors gap-4">
					<div class="min-w-0">
						<div class="flex items-center gap-2">
							<h4 class="text-xs font-extrabold text-fg truncate">{ver.name || ver.versionNumber}</h4>
							{#if ver.versionNumber && ver.name !== ver.versionNumber}
								<span class="text-[10px] font-mono px-2 py-0.5 rounded bg-fg/5 text-fg/60">
									{ver.versionNumber}
								</span>
							{/if}
						</div>
						<div class="flex items-center gap-2 text-[10px] text-fg/40 mt-1 font-mono">
							{#if file}
								<span>{file.filename}</span>
								<span>•</span>
								<span>{formatBytes(file.size)}</span>
							{/if}
						</div>
					</div>

					<button
						type="button"
						class={launcherButton({ variant: "primary", size: "sm", class: "shrink-0 flex items-center gap-1.5" })}
						onclick={() => onInstall(ver.id)}
					>
						{#if isModpack}
							<PackagePlus class="w-3 h-3" />
							<span>{uiText("common.create")}</span>
						{:else}
							<Download class="w-3 h-3" />
							<span>{uiText("mods.install")}</span>
						{/if}
					</button>
				</div>
			{/each}
		</div>
		{#if !showAll && versions.length > 30}
			<div class="p-3 text-center border-t border-fg/5 bg-fg/[0.01]">
				<button
					type="button"
					class={launcherButton({ variant: "ghost", size: "sm", class: "hover:underline" })}
					onclick={() => showAll = true}
				>
					{uiText("ui.c52bf495166dfc20")} {versions.length} {uiText("ui.e270e835d3dd8b85")}
				</button>
			</div>
		{/if}
	</div>
{/if}
