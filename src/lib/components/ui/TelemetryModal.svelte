<script lang="ts">
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
		class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md"
		transition:fade={{ duration: 200 }}
	>
		<div 
			class="relative w-full max-w-md bg-[#16171a] border border-white/10 rounded-3xl p-6 shadow-2xl overflow-hidden flex flex-col gap-5"
			transition:scale={{ start: 0.95, duration: 220 }}
		>
			<!-- Ambient background glow -->
			<div class="absolute -top-20 -right-20 w-48 h-48 bg-[#caa97c]/15 rounded-full blur-3xl pointer-events-none"></div>

			<!-- Header -->
			<div class="flex items-start justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-[#caa97c]/20 border border-[#caa97c]/30 flex items-center justify-center text-[#caa97c]">
						<Sparkles class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-bold text-white">Relatório Pós-Partida</h3>
						<p class="text-xs text-white/50">{summary.profileName} · {summary.versionId}</p>
					</div>
				</div>
				<button 
					type="button" 
					class="p-2 rounded-xl text-white/40 hover:text-white hover:bg-white/5 transition-colors cursor-pointer"
					onclick={onClose}
					aria-label="Fechar"
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			<!-- Stat Cards Grid -->
			<div class="grid grid-cols-2 gap-3">
				<!-- Session Time -->
				<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3.5 flex flex-col gap-1">
					<div class="flex items-center gap-1.5 text-white/50 text-[11px] font-semibold">
						<Timer class="w-3.5 h-3.5 text-amber-400" />
						Tempo de Jogo
					</div>
					<div class="text-lg font-black text-white">
						{formatDuration(summary.durationSeconds)}
					</div>
				</div>

				<!-- Peak RAM -->
				<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3.5 flex flex-col gap-1">
					<div class="flex items-center gap-1.5 text-white/50 text-[11px] font-semibold">
						<Cpu class="w-3.5 h-3.5 text-blue-400" />
						Pico de Memória RAM
					</div>
					<div class="text-lg font-black text-white">
						{(summary.peakRamMb / 1024).toFixed(1)} GB
					</div>
				</div>
			</div>

			<!-- Status Banner -->
			<div class="p-3 rounded-2xl border flex items-center gap-3 {summary.cleanExit ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-300' : 'bg-amber-500/10 border-amber-500/20 text-amber-300'}">
				{#if summary.cleanExit}
					<CheckCircle2 class="w-5 h-5 shrink-0 text-emerald-400" />
					<div class="text-xs">
						<div class="font-bold">Partida Encerrada com Sucesso</div>
						<div class="text-[10px] opacity-75">Nenhuma anomalia de crash ou perda de chunks detectada.</div>
					</div>
				{:else}
					<AlertTriangle class="w-5 h-5 shrink-0 text-amber-400" />
					<div class="text-xs">
						<div class="font-bold">Processo Finalizado (Código {summary.exitCode})</div>
						<div class="text-[10px] opacity-75">O jogo encerrou inesperadamente. Verifique a aba de Logs.</div>
					</div>
				{/if}
			</div>

			<!-- Actions -->
			<div class="flex items-center justify-end gap-2.5 pt-2 border-t border-white/5">
				<button 
					type="button" 
					class="px-4 py-2 rounded-xl text-xs font-bold text-white/70 hover:text-white bg-white/5 hover:bg-white/10 transition-all flex items-center gap-1.5 cursor-pointer"
					onclick={() => { onClose(); goto(`/logs`); }}
				>
					<FileText class="w-3.5 h-3.5" /> Ver Logs
				</button>
				{#if onPlayAgain}
					<Button 
						variant="primary" 
						size="sm"
						class="flex items-center gap-1.5"
						onclick={() => { onClose(); onPlayAgain?.(); }}
					>
						<Play class="w-3.5 h-3.5 fill-current" /> Jogar Novamente
					</Button>
				{:else}
					<Button variant="secondary" size="sm" onclick={onClose}>
						Fechar
					</Button>
				{/if}
			</div>
		</div>
	</div>
{/if}
