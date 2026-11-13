<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { quintOut } from "svelte/easing";
	import { onMount, onDestroy } from "svelte";
	import { slide } from "svelte/transition";
	import { Download, CheckCircle2, Clock, Zap, X, ChevronDown, ChevronUp } from "lucide-svelte";
	import { listenDownloadProgress, type DownloadProgress } from "$lib/api";
	import { appState } from "$lib/stores/app.svelte";
	import { modpackInstallation } from "$lib/stores/modpackInstallation.svelte";
    import { transferComplete } from '$lib/utils/installProgress';

	let progress = $state<DownloadProgress | null>(null);
	let isVisible = $state(false);
	let expanded = $state(false);
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
		if (isComplete) return uiText("ui.775284dda8418eeb");
		if (!progress) return uiText("ui.4d13b0441680f3d4");
		switch (progress.phase) {
			case "client":
			case "downloading":
				return uiText("ui.44abd9d3d37d32fd");
			case "libraries":
				return uiText("ui.d37dd9864798ec50");
			case "assets":
				return uiText("ui.da9c8dbc9b5e7f84");
			case "asset_index":
				return uiText("ui.473b37b2b73ee832");
			case "java":
				return "Instalando Java Runtime";
			default:
				return progress.phase || uiText("ui.4d13b0441680f3d4");
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

    const isComplete = $derived(progress !== null && transferComplete(progress));

	function scheduleHide(delay: number) {
		if (hideTimeout) clearTimeout(hideTimeout);
		hideTimeout = setTimeout(() => {
			isVisible = false;
			progress = null;
			hideTimeout = null;
		}, delay);
	}

	onMount(() => {
		let disposed = false;
		void listenDownloadProgress((data) => {
			if ((data.phase === "downloading" || data.phase === "done") && modpackInstallation.state.busy) return;
			progress = data;
			isVisible = true;

            if (hideTimeout) { clearTimeout(hideTimeout); hideTimeout = null; }
            if (transferComplete(data)) scheduleHide(2500);
		}).then((fn) => {
			if (disposed) fn();
			else unlisten = fn;
		}).catch((e) => console.error("download progress listener failed", e));
		return () => {
			disposed = true;
		};
	});

	$effect(() => {
		if (appState.isLaunching || appState.isGameRunning) return;
		if (!isVisible) return;
		if (progress?.phase === "java" && !isComplete) return;
		scheduleHide(3000);
	});

	onDestroy(() => {
		if (unlisten) unlisten();
		if (hideTimeout) clearTimeout(hideTimeout);
	});
</script>

{#if isVisible && progress}
	<div 
		class="w-full select-none"
		transition:slide={{ easing: quintOut, duration: 250, axis: "y" }}
	>
		<div class="dl-card relative space-y-2.5 overflow-hidden rounded-xl border border-fg/12 bg-bg-elevated/95 p-3 shadow-elevated backdrop-blur-xl">
			<div class="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-brand-400/70 to-transparent"></div>
			<div class="flex items-center justify-between text-xs gap-3">
				<button type="button" class="flex min-w-0 flex-1 items-center gap-3 text-left" onclick={() => expanded = !expanded} aria-expanded={expanded}>
					{#if isComplete}
						<div class="h-8 w-8 rounded-lg bg-emerald-500/15 border border-emerald-500/30 text-emerald-400 flex items-center justify-center shrink-0 shadow-sm">
							<CheckCircle2 class="w-4 h-4" />
						</div>
					{:else}
						<div class="h-8 w-8 rounded-lg bg-brand-400/15 border border-brand-400/30 text-brand-400 flex items-center justify-center shrink-0 shadow-sm">
							<Download class="w-4 h-4 animate-bounce motion-reduce:animate-none" />
						</div>
					{/if}
					<span class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<p class="font-extrabold text-fg text-xs truncate tracking-tight">
								{phaseTitle}
							</p>
							{#if !isComplete && progress.total > 0}
								<span class="text-[10px] font-mono px-2 py-0.2 rounded-full bg-fg/5 border border-fg/10 text-fg/50">
									{progress.completed}/{progress.total}
								</span>
							{/if}
						</div>
					</span>
				</button>
				<div class="flex items-center gap-2 shrink-0">
					<span class="dl-meta">
						{#if !isComplete && speedText}<span class="dl-chip"><Zap class="h-3 w-3" />{speedText}</span>{/if}
						{#if !isComplete && etaText}<span class="dl-chip"><Clock class="h-3 w-3" />{etaText}</span>{/if}
					</span>
					<span class="font-black text-brand-400 text-xs font-mono min-w-[38px] text-right tabular-nums">{percent}%</span>
					<button type="button" class={launcherButton({ variant: "secondary", size: "icon", class: "flex items-center justify-center" })} onclick={() => expanded = !expanded} aria-label={uiText(expanded ? "downloadsDesign.collapseProgress" : "downloadsDesign.expandProgress")} aria-expanded={expanded}>
						{#if expanded}<ChevronUp class="w-3.5 h-3.5" />{:else}<ChevronDown class="w-3.5 h-3.5" />{/if}
					</button>
					<button 
						type="button"
						class={launcherButton({ variant: "secondary", size: "icon", class: "flex items-center justify-center" })}
						onclick={() => isVisible = false}
						title={uiText("ui.f5d746dc17252472")}
						aria-label={uiText("ui.f5d746dc17252472")}
					>
						<X class="w-3.5 h-3.5" />
					</button>
				</div>
			</div>
			{#if expanded}
				<div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-[10px] text-fg-muted">
					{#if progress.currentFile}<span class="max-w-full truncate font-mono">{progress.currentFile}</span>{/if}
					{#if speedText && !isComplete}<span class="inline-flex items-center gap-1 font-mono"><Zap class="h-3 w-3 text-brand-400" />{speedText}</span>{/if}
					{#if etaText && !isComplete}<span class="inline-flex items-center gap-1 font-mono"><Clock class="h-3 w-3" />{etaText}</span>{/if}
				</div>
			{/if}
			<div class="dl-track h-2 w-full overflow-hidden rounded-full bg-fg/8">
				<div 
					class="dl-fill relative h-full rounded-full bg-gradient-to-r from-brand-600 via-brand-400 to-brand-300 transition-[width] duration-300 ease-out"
					style="width: {percent}%"
				>
					<div class="dl-shine absolute inset-0"></div>
				</div>
			</div>
		</div>
	</div>
{/if}
