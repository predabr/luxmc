<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { fade, slide } from "svelte/transition";
	import { Download, CheckCircle2, Clock, Zap } from "lucide-svelte";
	import { listenDownloadProgress, type DownloadProgress } from "$lib/api";

	let progress = $state<DownloadProgress | null>(null);
	let isVisible = $state(false);
	let hideTimeout: ReturnType<typeof setTimeout> | null = null;
	let unlisten: (() => void) | null = null;

	const percent = $derived.by(() => {
		if (!progress) return 0;
		if (progress.totalBytes > 0 && progress.bytesDownloaded > 0) {
			return Math.min(100, Math.max(0, Math.round((progress.bytesDownloaded / progress.totalBytes) * 100)));
		}
		if (progress.total > 0) {
			return Math.min(100, Math.max(0, Math.round((progress.completed / progress.total) * 100)));
		}
		return 0;
	});

	const speedText = $derived.by(() => {
		if (!progress?.speed?.bytesPerSecond) return null;
		const bps = progress.speed.bytesPerSecond;
		if (bps > 1024 * 1024) {
			return `${(bps / (1024 * 1024)).toFixed(1)} MB/s`;
		}
		return `${Math.round(bps / 1024)} KB/s`;
	});

	const phaseTitle = $derived.by(() => {
		if (isComplete) return "Download concluído!";
		if (!progress) return "Baixando arquivos...";
		switch (progress.phase) {
			case "client":
			case "downloading":
				return "Baixando Minecraft client.jar...";
			case "libraries":
				return "Baixando bibliotecas nativas...";
			case "assets":
				return "Baixando texturas e recursos (assets)...";
			case "asset_index":
				return "Indexando recursos do jogo...";
			case "java":
				return "Instalando Java Runtime...";
			default:
				return progress.phase || "Baixando arquivos...";
		}
	});

	const etaText = $derived.by(() => {
		if (!progress?.speed?.bytesPerSecond) return null;
		if (progress.totalBytes > 0 && progress.totalBytes > progress.bytesDownloaded) {
			const remainingBytes = progress.totalBytes - progress.bytesDownloaded;
			const remainingSecs = Math.round(remainingBytes / progress.speed.bytesPerSecond);
			if (remainingSecs > 0 && remainingSecs <= 3600) {
				const mins = Math.floor(remainingSecs / 60);
				const secs = remainingSecs % 60;
				return mins > 0 ? `${mins}m ${secs}s` : `${secs}s`;
			}
		} else if (progress.total > 0 && progress.total > progress.completed && progress.completed > 0 && progress.speed.elapsedMs > 0) {
			const remainingItems = progress.total - progress.completed;
			const msPerItem = progress.speed.elapsedMs / progress.completed;
			const remainingSecs = Math.round((remainingItems * msPerItem) / 1000);
			if (remainingSecs > 0 && remainingSecs <= 3600) {
				const mins = Math.floor(remainingSecs / 60);
				const secs = remainingSecs % 60;
				return mins > 0 ? `${mins}m ${secs}s` : `${secs}s`;
			}
		}
		return null;
	});

	const isComplete = $derived.by(() => {
		if (!progress) return false;
		if (progress.total > 0 && progress.completed >= progress.total) return true;
		if (progress.totalBytes > 0 && progress.bytesDownloaded >= progress.totalBytes) return true;
		return false;
	});

	onMount(() => {
		void listenDownloadProgress((data) => {
			progress = data;
			isVisible = true;

			if (hideTimeout) {
				clearTimeout(hideTimeout);
				hideTimeout = null;
			}

			const done = (data.total > 0 && data.completed >= data.total) ||
				(data.totalBytes > 0 && data.bytesDownloaded >= data.totalBytes);

			if (done) {
				hideTimeout = setTimeout(() => {
					isVisible = false;
					progress = null;
				}, 2200);
			}
		}).then((fn) => {
			unlisten = fn;
		}).catch(() => {});
	});

	onDestroy(() => {
		if (unlisten) unlisten();
		if (hideTimeout) clearTimeout(hideTimeout);
	});
</script>

{#if isVisible && progress}
	<div 
		class="fixed bottom-3 right-6 left-[86px] z-50 select-none"
		transition:slide={{ duration: 250, axis: "y" }}
	>
		<div class="mx-auto max-w-4xl bg-[#15161b]/95 backdrop-blur-xl border border-white/10 rounded-2xl p-3.5 shadow-2xl space-y-2">
			<div class="flex items-center justify-between text-xs gap-3">
				<div class="flex items-center gap-2.5 min-w-0">
					{#if isComplete}
						<div class="h-6 w-6 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center shrink-0">
							<CheckCircle2 class="w-3.5 h-3.5" />
						</div>
					{:else}
						<div class="h-6 w-6 rounded-full bg-brand-500/20 text-brand-500 flex items-center justify-center shrink-0">
							<Download class="w-3.5 h-3.5 animate-bounce" />
						</div>
					{/if}

					<div class="min-w-0">
						<p class="font-bold text-white text-xs truncate">
							{phaseTitle}
						</p>
						{#if progress.currentFile}
							<p class="text-[10px] text-white/40 truncate font-mono">
								{progress.currentFile}
							</p>
						{/if}
					</div>
				</div>

				<div class="flex items-center gap-3 shrink-0 text-[11px] text-white/60">
					{#if speedText && !isComplete}
						<span class="flex items-center gap-1 font-mono text-brand-500 font-bold">
							<Zap class="w-3 h-3 text-brand-500" />
							{speedText}
						</span>
					{/if}

					{#if etaText && !isComplete}
						<span class="flex items-center gap-1 font-mono text-white/40">
							<Clock class="w-3 h-3 text-white/30" />
							{etaText}
						</span>
					{/if}

					{#if progress.total > 0}
						<span class="text-white/40 text-[10px] font-mono">
							{progress.completed}/{progress.total}
						</span>
					{/if}

					<span class="font-bold text-white text-xs min-w-[36px] text-right font-mono">
						{percent}%
					</span>
				</div>
			</div>

			<!-- Smooth Progress Track -->
			<div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden relative">
				<div 
					class="h-full bg-gradient-to-r from-brand-500 via-[#ebd095] to-brand-500 rounded-full transition-all duration-300 ease-out relative"
					style="width: {percent}%"
				>
					<div class="absolute inset-0 bg-white/20 animate-pulse"></div>
				</div>
			</div>
		</div>
	</div>
{/if}
