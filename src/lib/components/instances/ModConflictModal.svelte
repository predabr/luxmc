<script lang="ts">
	import { fade, scale } from "svelte/transition";
	import {
		AlertTriangle,
		ShieldAlert,
		X,
		Check,
		Wrench,
		Sparkles
	} from "lucide-svelte";
	import type { PreLaunchCheckResult, ModConflict } from "$lib/api";
	import { instanceModToggle } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let {
		isOpen = $bindable(false),
		profileId,
		conflictsResult,
		onResolved,
		onProceedAnyway,
		onClose
	}: {
		isOpen: boolean;
		profileId: string;
		conflictsResult: PreLaunchCheckResult | null;
		onResolved: () => void;
		onProceedAnyway: () => void;
		onClose: () => void;
	} = $props();

	let resolving = $state(false);

	async function handleFixConflict(c: ModConflict) {
		resolving = true;
		try {
			await instanceModToggle(profileId, c.fileToDisable, true);
			toast(`Mod conflitante "${c.fileToDisable}" desativado com sucesso!`, "success");
			playSound("click");
			onResolved();
		} catch (e) {
			toast("Falha ao desativar mod: " + String(e), "error");
		} finally {
			resolving = false;
		}
	}

	async function handleFixAll() {
		if (!conflictsResult) return;
		resolving = true;
		try {
			for (const c of conflictsResult.conflicts) {
				await instanceModToggle(profileId, c.fileToDisable, true);
			}
			toast("Todos os conflitos resolvidos automaticamente!", "success");
			playSound("click");
			onResolved();
		} catch (e) {
			toast("Falha ao resolver conflitos: " + String(e), "error");
		} finally {
			resolving = false;
		}
	}
</script>

{#if isOpen && conflictsResult && conflictsResult.hasConflicts}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/85 backdrop-blur-md select-none" in:fade={{ duration: 150 }}>
		<div class="w-full max-w-lg rounded-3xl bg-[#141518] border border-rose-500/30 p-6 shadow-2xl space-y-5" in:scale={{ start: 0.95, duration: 200 }}>
			<!-- Header -->
			<div class="flex items-center justify-between border-b border-white/10 pb-4">
				<div class="flex items-center gap-3">
					<div class="w-11 h-11 rounded-2xl bg-rose-500/20 border border-rose-500/40 flex items-center justify-center text-rose-400">
						<ShieldAlert class="w-6 h-6" />
					</div>
					<div>
						<h3 class="text-base font-black text-white">Incompatibilidade Detectada!</h3>
						<p class="text-xs text-rose-300/80">Evite que o jogo feche antes de abrir</p>
					</div>
				</div>

				<button
					type="button"
					class="p-2 rounded-xl text-white/40 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
					onclick={onClose}
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			<!-- Description -->
			<p class="text-xs text-white/60 leading-relaxed">
				O Luxmc analisou seus mods e encontrou conflitos conhecidos que causam travamentos imediatos na inicialização do Minecraft. Recomendamos desativar o mod conflitante:
			</p>

			<!-- Conflicts List -->
			<div class="space-y-3 max-h-64 overflow-y-auto pr-1 custom-scrollbar">
				{#each conflictsResult.conflicts as c}
					<div class="p-4 rounded-2xl bg-[#18191c] border border-rose-500/20 space-y-2.5">
						<div class="flex items-center justify-between">
							<span class="text-xs font-black text-rose-300">{c.title}</span>
							<span class="text-[10px] font-mono text-white/40 bg-white/5 px-2 py-0.5 rounded">Crash Crítico</span>
						</div>
						<p class="text-[11px] text-white/50 leading-relaxed">{c.description}</p>
						<div class="flex items-center justify-between pt-1">
							<span class="text-[10px] font-bold text-emerald-400">{c.recommendedAction}</span>
							<button
								type="button"
								class="px-4 py-1.5 rounded-xl bg-emerald-500/15 hover:bg-emerald-500/25 border border-emerald-500/30 text-emerald-300 font-bold text-xs flex items-center gap-1.5 transition-all cursor-pointer active:scale-95 shadow-sm"
								onclick={() => handleFixConflict(c)}
								disabled={resolving}
							>
								<Wrench class="w-3.5 h-3.5" />
								<span>Corrigir com 1 Clique</span>
							</button>
						</div>
					</div>
				{/each}

				{#each conflictsResult.duplicates as dup}
					<div class="p-3 rounded-xl bg-amber-500/10 border border-amber-500/20 text-xs text-amber-300">
						{dup}
					</div>
				{/each}
			</div>

			<!-- Actions -->
			<div class="flex items-center justify-between pt-2 border-t border-white/10">
				<button
					type="button"
					class="text-xs text-white/40 hover:text-white transition-colors cursor-pointer hover:underline"
					onclick={onProceedAnyway}
				>
					Ignorar e Iniciar Mesmo Assim
				</button>

				<button
					type="button"
					class="px-6 py-2.5 rounded-2xl bg-gradient-to-r from-emerald-500 to-teal-500 hover:from-emerald-400 hover:to-teal-400 text-black font-black text-xs uppercase tracking-wider transition-all shadow-lg shadow-emerald-500/20 cursor-pointer active:scale-95"
					onclick={handleFixAll}
					disabled={resolving}
				>
					Corrigir Todos
				</button>
			</div>
		</div>
	</div>
{/if}
