<script lang="ts">
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
	<div class="bg-[#18191c] border border-white/5 rounded-3xl p-12 text-center text-white/40 text-xs">
		<Layers class="w-10 h-10 text-white/20 mx-auto mb-3" />
		<p>Nenhuma versão listada para este projeto.</p>
	</div>
{:else}
	<div class="bg-[#18191c] border border-white/5 rounded-3xl overflow-hidden shadow-md">
		<div class="divide-y divide-white/5">
			{#each displayedVersions as ver}
				{@const file = ver.files[0]}
				<div class="p-4 flex items-center justify-between hover:bg-white/[0.02] transition-colors gap-4">
					<div class="min-w-0">
						<div class="flex items-center gap-2">
							<h4 class="text-xs font-extrabold text-white truncate">{ver.name || ver.versionNumber}</h4>
							{#if ver.versionNumber}
								<span class="text-[10px] font-mono px-2 py-0.5 rounded bg-white/5 text-white/60">
									{ver.versionNumber}
								</span>
							{/if}
						</div>
						<div class="flex items-center gap-2 text-[10px] text-white/40 mt-1 font-mono">
							{#if file}
								<span>{file.filename}</span>
								<span>•</span>
								<span>{formatBytes(file.size)}</span>
							{/if}
						</div>
					</div>

					<button
						type="button"
						class="shrink-0 bg-[#2b2c32] hover:bg-[#caa97c] hover:text-black text-white text-xs font-bold px-3.5 py-1.5 rounded-xl flex items-center gap-1.5 transition-all shadow-sm cursor-pointer"
						onclick={() => onInstall(ver.id)}
					>
						{#if isModpack}
							<PackagePlus class="w-3 h-3" />
							<span>Criar</span>
						{:else}
							<Download class="w-3 h-3" />
							<span>Instalar</span>
						{/if}
					</button>
				</div>
			{/each}
		</div>
		{#if !showAll && versions.length > 30}
			<div class="p-3 text-center border-t border-white/5 bg-white/[0.01]">
				<button
					type="button"
					class="text-xs text-[#caa97c] hover:underline font-bold cursor-pointer"
					onclick={() => showAll = true}
				>
					Mostrar todas as {versions.length} versões
				</button>
			</div>
		{/if}
	</div>
{/if}
