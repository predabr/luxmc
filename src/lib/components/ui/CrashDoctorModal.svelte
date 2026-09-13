<script lang="ts">
	import { fade, scale } from "svelte/transition";
	import { goto } from "$app/navigation";
	import { 
		AlertTriangle, 
		CheckCircle2, 
		Wrench, 
		FileText, 
		X, 
		ChevronDown, 
		ChevronUp,
		Zap,
		ShieldAlert
	} from "lucide-svelte";
	import { crashDoctor } from "$lib/stores/crashDoctor.svelte";

	let showSnippet = $state(false);

	const diagnosis = $derived(crashDoctor.diagnosis);

	async function handleFix() {
		await crashDoctor.applyFix();
	}

	function goToLogs() {
		crashDoctor.close();
		goto("/logs");
	}
</script>

{#if crashDoctor.isOpen && diagnosis}
	<div 
		class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md"
		transition:fade={{ duration: 150 }}
	>
		<div 
			class="relative w-full max-w-2xl bg-[#141518] border border-amber-500/30 rounded-3xl p-6 md:p-8 shadow-2xl overflow-hidden"
			transition:scale={{ duration: 200, start: 0.95 }}
		>
			<div class="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-amber-500 via-red-500 to-amber-500"></div>

			<div class="flex items-start justify-between gap-4 mb-6">
				<div class="flex items-center gap-3">
					<div class="w-12 h-12 rounded-2xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
						<ShieldAlert class="w-6 h-6" />
					</div>
					<div>
						<span class="text-[10px] font-black uppercase tracking-wider text-amber-400 bg-amber-500/10 px-2.5 py-0.5 rounded-full border border-amber-500/20">
							Crash Doctor · Diagnóstico em Português
						</span>
						<h3 class="text-lg font-black text-white mt-1">
							{diagnosis.title}
						</h3>
					</div>
				</div>

				<button 
					type="button"
					class="text-white/40 hover:text-white p-2 rounded-xl hover:bg-white/5 transition-colors cursor-pointer"
					onclick={() => crashDoctor.close()}
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			<div class="space-y-4">
				<div class="bg-red-500/10 border border-red-500/20 rounded-2xl p-4 text-xs text-red-200 leading-relaxed">
					<div class="flex items-center gap-2 font-bold text-red-400 mb-1">
						<AlertTriangle class="w-4 h-4 shrink-0" />
						O que aconteceu:
					</div>
					{diagnosis.message}
				</div>

				<div class="bg-emerald-500/10 border border-emerald-500/20 rounded-2xl p-4 text-xs text-emerald-200 leading-relaxed">
					<div class="flex items-center gap-2 font-bold text-emerald-400 mb-1">
						<CheckCircle2 class="w-4 h-4 shrink-0" />
						Como resolver:
					</div>
					{diagnosis.solution}
				</div>

				{#if diagnosis.logSnippet}
					<div class="border border-white/5 rounded-2xl bg-black/40 overflow-hidden text-xs">
						<button 
							type="button"
							class="w-full flex items-center justify-between p-3.5 text-white/50 hover:text-white transition-colors cursor-pointer select-none"
							onclick={() => showSnippet = !showSnippet}
						>
							<span class="flex items-center gap-2 font-mono text-[11px]">
								<FileText class="w-3.5 h-3.5 text-amber-400" />
								Trecho do erro detectado no log
							</span>
							{#if showSnippet}
								<ChevronUp class="w-4 h-4" />
							{:else}
								<ChevronDown class="w-4 h-4" />
							{/if}
						</button>
						{#if showSnippet}
							<pre class="p-3.5 pt-0 text-[10px] font-mono text-amber-200/80 overflow-x-auto whitespace-pre-wrap leading-relaxed max-h-48 custom-scrollbar border-t border-white/5">
								{diagnosis.logSnippet}
							</pre>
						{/if}
					</div>
				{/if}
			</div>

			<div class="flex flex-wrap items-center justify-end gap-3 mt-6 pt-5 border-t border-white/5">
				<button 
					type="button"
					class="px-4 py-2.5 rounded-xl text-xs font-bold text-white/60 hover:text-white hover:bg-white/5 transition-all cursor-pointer"
					onclick={goToLogs}
				>
					Ver Log Completo
				</button>

				{#if diagnosis.recommendedAction === 'repair_modpack'}
					<button 
						type="button"
						class="px-5 py-2.5 rounded-xl text-xs font-black bg-gradient-to-r from-emerald-500 to-emerald-600 hover:from-emerald-400 hover:to-emerald-500 text-black shadow-lg shadow-emerald-500/20 transition-all flex items-center gap-2 cursor-pointer disabled:opacity-50"
						disabled={crashDoctor.isFixing}
						onclick={handleFix}
					>
						<Wrench class="w-4 h-4 fill-current" />
						{crashDoctor.isFixing ? 'Reparando...' : 'Reparar Modpack e Baixar Faltantes'}
					</button>
				{:else if diagnosis.recommendedAction === 'disable_optifine' || diagnosis.recommendedAction === 'disable_mod'}
					<button 
						type="button"
						class="px-5 py-2.5 rounded-xl text-xs font-black bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-400 hover:to-amber-500 text-black shadow-lg shadow-amber-500/20 transition-all flex items-center gap-2 cursor-pointer disabled:opacity-50"
						disabled={crashDoctor.isFixing}
						onclick={handleFix}
					>
						<Zap class="w-4 h-4 fill-current" />
						{crashDoctor.isFixing ? 'Aplicando...' : 'Desativar Mod Automaticamente'}
					</button>
				{/if}

				<button 
					type="button"
					class="px-5 py-2.5 rounded-xl text-xs font-bold bg-white/10 hover:bg-white/15 text-white transition-all cursor-pointer"
					onclick={() => crashDoctor.close()}
				>
					Entendido
				</button>
			</div>
		</div>
	</div>
{/if}
