<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { fade, scale } from "svelte/transition";
	import { 
		Timer, 
		Cpu, 
		CheckCircle2, 
		AlertTriangle, 
		X, 
		Play, 
		FileText,
		Sparkles
	} from "lucide-svelte";
	import type { GameTelemetrySummary } from "$lib/api/types";
	import Button from "./Button.svelte";
	import { goto } from "$app/navigation";

	type Props = {
		summary: GameTelemetrySummary | null;
		onClose: () => void;
		onPlayAgain?: () => void;
	};

	let { summary, onClose, onPlayAgain }: Props = $props();

	function formatDuration(seconds: number): string {
		if (seconds < 60) return `${seconds}s`;
		const mins = Math.floor(seconds / 60);
		const secs = seconds % 60;
		if (mins < 60) return `${mins}m ${secs}s`;
		const hours = Math.floor(mins / 60);
		const remMins = mins % 60;
		return `${hours}h ${remMins}m`;
	}
</script>

{#if summary}
	<div 
		class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/75 backdrop-blur-md"
		transition:fade={{ easing: quintOut, duration: 260 }}
	>
		<div 
			class="relative w-full max-w-md bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-2xl overflow-hidden flex flex-col gap-5"
			transition:scale={{ easing: backOut, start: 0.95, duration: 220 }}
		>
			<!-- Ambient background glow -->
			<div class="absolute -top-20 -right-20 w-48 h-48 bg-brand-400/15 rounded-full blur-3xl pointer-events-none"></div>

			<!-- Header -->
			<div class="flex items-start justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-400/20 border border-brand-400/30 flex items-center justify-center text-brand-400">
						<Sparkles class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-bold text-fg">{uiText("ui.6b4c2576adca4c08")}</h3>
						<p class="text-xs text-fg/50">{summary.profileName} · {summary.versionId}</p>
					</div>
				</div>
				<button 
					type="button" 
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					onclick={onClose}
					aria-label={uiText("statusBanner.dismiss")}
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			<!-- Stat Cards Grid -->
			<div class="grid grid-cols-2 gap-3">
				<!-- Session Time -->
				<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex flex-col gap-1">
					<div class="flex items-center gap-1.5 text-fg/50 text-[11px] font-semibold">
						<Timer class="w-3.5 h-3.5 text-amber-400" />
						{uiText("ui.4ea691a877c5cfeb")}
					</div>
					<div class="text-lg font-black text-fg">
						{formatDuration(summary.durationSeconds)}
					</div>
				</div>

				<!-- Peak RAM -->
				<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex flex-col gap-1">
					<div class="flex items-center gap-1.5 text-fg/50 text-[11px] font-semibold">
						<Cpu class="w-3.5 h-3.5 text-blue-400" />
						{uiText("ui.57f3dae6f993c39f")}
					</div>
					<div class="text-lg font-black text-fg">
						{(summary.peakRamMb / 1024).toFixed(1)} GB
					</div>
				</div>
			</div>

			<!-- Status Banner -->
			<div class="p-3 rounded-2xl border flex items-center gap-3 {summary.cleanExit ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-300' : 'bg-amber-500/10 border-amber-500/20 text-amber-300'}">
				{#if summary.cleanExit}
					<CheckCircle2 class="w-5 h-5 shrink-0 text-emerald-400" />
					<div class="text-xs">
						<div class="font-bold">{uiText("ui.3ec71c1979ba6b3f")}</div>
						<div class="text-[10px] opacity-75">{uiText("ui.dccde811775eb863")}</div>
					</div>
				{:else}
					<AlertTriangle class="w-5 h-5 shrink-0 text-amber-400" />
					<div class="text-xs">
						<div class="font-bold">{uiText("ui.5444322ee7c2b179")} {summary.exitCode})</div>
						<div class="text-[10px] opacity-75">{uiText("ui.571063fb89953cff")}</div>
					</div>
				{/if}
			</div>

			<!-- Actions -->
			<div class="flex items-center justify-end gap-2.5 pt-2 border-t border-fg/5">
				<button 
					type="button" 
					class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
					onclick={() => { onClose(); goto(`/logs`); }}
				>
					<FileText class="w-3.5 h-3.5" /> {uiText("ui.5a52736799b5335a")}
				</button>
				{#if onPlayAgain}
					<Button 
						variant="primary" 
						size="sm"
						class="flex items-center gap-1.5"
						onclick={() => { onClose(); onPlayAgain?.(); }}
					>
						<Play class="w-3.5 h-3.5 fill-current" /> {uiText("ui.79c5cb116ea84d99")}
					</Button>
				{:else}
					<Button variant="secondary" size="sm" onclick={onClose}>
						{uiText("statusBanner.dismiss")}
					</Button>
				{/if}
			</div>
		</div>
	</div>
{/if}
