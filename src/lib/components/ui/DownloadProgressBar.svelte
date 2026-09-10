<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { fade, slide, scale } from "svelte/transition";
	import { Download, CheckCircle2, Clock, Zap, X, HardDrive } from "lucide-svelte";
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
		if (isComplete) return "Download Concluído!";
		if (!progress) return "Baixando arquivos...";
		switch (progress.phase) {
			case "client":
			case "downloading":
				return "Baixando Minecraft (client.jar)";
			case "libraries":
				return "Baixando bibliotecas nativas";
			case "assets":
				return "Baixando recursos e texturas";
			case "asset_index":
				return "Indexando recursos do jogo";
			case "java":
				return "Instalando Java Runtime";
			default:
				return progress.phase || "Baixando arquivos...";
		}
	});

	const etaText = $derived.by(() => {
		if (!progress?.speed?.bytesPerSecond) return null;
		if (progress.totalBytes > 0 && progress.bytesDownloaded > 0 && progress.totalBytes > progress.bytesDownloaded) {
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
				}, 2500);
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
		class="fixed bottom-4 right-6 z-50 select-none max-w-xl w-[calc(100%-110px)]"
		transition:slide={{ duration: 250, axis: "y" }}
	>
		<div class="bg-[#14151a]/95 backdrop-blur-2xl border border-[#caa97c]/30 rounded-2xl p-4 shadow-[0_12px_40px_rgba(0,0,0,0.8)] space-y-2.5 relative overflow-hidden">
			<!-- Subtle gold accent glow on top edge -->
			<div class="absolute top-0 left-0 right-0 h-[1.5px] bg-gradient-to-r from-transparent via-[#caa97c]/50 to-transparent"></div>

			<!-- Main Top Row: Icon + Title + Metrics + Close -->
			<div class="flex items-center justify-between text-xs gap-3">
				
				<!-- Left: Glowing Icon + Title & File -->
				<div class="flex items-center gap-3 min-w-0">
					{#if isComplete}
						<div class="h-8 w-8 rounded-xl bg-emerald-500/15 border border-emerald-500/30 text-emerald-400 flex items-center justify-center shrink-0 shadow-sm">
							<CheckCircle2 class="w-4 h-4" />
						</div>
					{:else}
						<div class="h-8 w-8 rounded-xl bg-[#caa97c]/15 border border-[#caa97c]/30 text-[#caa97c] flex items-center justify-center shrink-0 shadow-sm">
							<Download class="w-4 h-4 animate-bounce" />
						</div>
					{/if}

					<div class="min-w-0">
						<div class="flex items-center gap-2">
							<p class="font-extrabold text-white text-xs truncate tracking-tight">
								{phaseTitle}
							</p>
							{#if !isComplete && progress.total > 0}
								<span class="text-[10px] font-mono px-2 py-0.2 rounded-full bg-white/5 border border-white/10 text-white/50">
									{progress.completed}/{progress.total}
								</span>
							{/if}
						</div>
						{#if progress.currentFile}
							<p class="text-[10px] text-white/40 truncate font-mono mt-0.5 max-w-sm">
								{progress.currentFile}
							</p>
						{/if}
					</div>
				</div>

				<!-- Right: Speed, ETA & Percentage & Dismiss -->
				<div class="flex items-center gap-2.5 shrink-0">
					{#if speedText && !isComplete}
						<div class="hidden sm:flex items-center gap-1 font-mono text-[10px] px-2 py-0.5 rounded-lg bg-[#caa97c]/10 text-[#caa97c] border border-[#caa97c]/20 font-bold">
							<Zap class="w-3 h-3 text-[#caa97c]" />
							<span>{speedText}</span>
						</div>
					{/if}

					{#if etaText && !isComplete}
						<div class="hidden md:flex items-center gap-1 font-mono text-[10px] px-2 py-0.5 rounded-lg bg-white/5 text-white/50 border border-white/5">
							<Clock class="w-3 h-3 text-white/40" />
							<span>{etaText}</span>
						</div>
					{/if}

					<span class="font-black text-[#caa97c] text-sm font-mono min-w-[36px] text-right">
						{percent}%
					</span>

					<button 
						type="button"
						class="w-6 h-6 rounded-lg text-white/30 hover:text-white hover:bg-white/10 flex items-center justify-center transition-colors cursor-pointer"
						onclick={() => isVisible = false}
						title="Ocultar barra"
					>
						<X class="w-3.5 h-3.5" />
					</button>
				</div>
			</div>

			<!-- Liquid Gold Progress Track -->
			<div class="w-full h-2 bg-black/40 rounded-full overflow-hidden p-0.5 border border-white/5">
				<div 
					class="h-full bg-gradient-to-r from-[#b38e5d] via-[#ecd6a8] to-[#caa97c] rounded-full transition-all duration-200 ease-out relative shadow-[0_0_12px_rgba(202,169,124,0.4)]"
					style="width: {percent}%"
				>
					<div class="absolute inset-0 bg-white/25 animate-pulse"></div>
				</div>
			</div>
		</div>
	</div>
{/if}
