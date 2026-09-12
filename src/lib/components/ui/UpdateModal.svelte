<script lang="ts">
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { DownloadCloud, Sparkles, ArrowRight, X, Loader2, CheckCircle2 } from "lucide-svelte";
	import { updaterStore } from "$lib/stores/updater.svelte";

	onMount(() => {
		const timer = setTimeout(() => {
			updaterStore.check(false);
		}, 2000);
		return () => clearTimeout(timer);
	});
</script>

{#if updaterStore.showModal}
	<div
		class="fixed inset-0 z-[100] flex items-center justify-center p-6 bg-black/80 backdrop-blur-md"
		in:fade={{ duration: 300 }}
		out:fade={{ duration: 200 }}
	>
		<div
			class="relative w-full max-w-lg bg-[#18191c] border border-brand-500/30 rounded-3xl p-8 shadow-2xl overflow-hidden"
			in:scale={{ start: 0.95, duration: 300, opacity: 0 }}
		>
			<div class="absolute -top-20 -right-20 w-64 h-64 bg-brand-500/20 rounded-full blur-3xl pointer-events-none"></div>
			<div class="absolute -bottom-20 -left-20 w-64 h-64 bg-brand-500/10 rounded-full blur-3xl pointer-events-none"></div>

			{#if !updaterStore.isUpdating}
				<button
					class="absolute top-4 right-4 p-2 rounded-full text-white/50 hover:text-white hover:bg-white/10 transition-colors"
					onclick={() => (updaterStore.showModal = false)}
				>
					<X class="w-5 h-5" />
				</button>
			{/if}

			<div class="relative flex flex-col items-center text-center space-y-6">
				<div
					class="w-20 h-20 bg-brand-500/15 border-2 border-brand-500/30 rounded-full flex items-center justify-center shadow-[0_0_30px_rgba(var(--brand-500-rgb),0.3)]"
				>
					{#if updaterStore.isUpdating}
						<Loader2 class="w-10 h-10 text-brand-500 animate-spin" />
					{:else}
						<DownloadCloud class="w-10 h-10 text-brand-500" />
					{/if}
				</div>

				<div>
					<h2 class="text-2xl font-black text-white flex items-center justify-center gap-2">
						{#if updaterStore.isUpdating}
							Atualizando o Luxmc...
						{:else}
							<Sparkles class="w-5 h-5 text-brand-500" /> Nova Versão Disponível!
						{/if}
					</h2>
					<p class="text-sm text-white/60 mt-2">
						{#if updaterStore.isUpdating}
							{updaterStore.statusText || "Baixando e instalando a atualização..."}
						{:else}
							O Luxmc ficou ainda melhor. Atualize agora sem perder suas instâncias e configurações!
						{/if}
					</p>
				</div>

				<div class="flex items-center justify-center gap-4 w-full bg-black/40 rounded-2xl p-4 border border-white/5">
					<div class="flex flex-col items-center">
						<span class="text-[10px] text-white/40 font-bold uppercase tracking-widest">Sua Versão</span>
						<span class="text-lg font-mono text-white/80 font-bold">v{updaterStore.currentVersion}</span>
					</div>
					<ArrowRight class="w-5 h-5 text-white/30" />
					<div class="flex flex-col items-center">
						<span class="text-[10px] text-brand-500 font-bold uppercase tracking-widest">Nova Versão</span>
						<span class="text-lg font-mono text-brand-500 font-black">v{updaterStore.latestVersion}</span>
					</div>
				</div>

				{#if updaterStore.isUpdating}
					<div class="w-full space-y-3 bg-black/30 p-4 rounded-2xl border border-white/5">
						<div class="flex justify-between text-xs font-bold text-white/70">
							<span>{updaterStore.statusText}</span>
							<span class="text-brand-500 font-mono">{updaterStore.progressPercent}%</span>
						</div>
						<div class="w-full bg-white/10 rounded-full h-3 overflow-hidden">
							<div
								class="bg-brand-500 h-full rounded-full transition-all duration-300"
								style="width: {updaterStore.progressPercent}%;"
							></div>
						</div>
						<p class="text-[11px] text-white/40 text-center">
							O aplicativo será reiniciado automaticamente quando a instalação concluir.
						</p>
					</div>
				{:else}
					<div class="w-full text-left bg-white/5 rounded-2xl p-4 max-h-32 overflow-y-auto custom-scrollbar border border-white/5">
						<span class="text-[10px] text-white/40 font-bold uppercase tracking-widest block mb-2">Novidades:</span>
						<p class="text-xs text-white/80 whitespace-pre-line leading-relaxed">
							{updaterStore.releaseNotes}
						</p>
					</div>

					<button
						class="w-full py-3.5 bg-brand-500 hover:brightness-110 text-black font-black rounded-2xl transition-all hover:scale-[1.02] active:scale-[0.98] flex items-center justify-center gap-2 shadow-[0_0_20px_rgba(var(--brand-500-rgb),0.4)] cursor-pointer"
						onclick={() => updaterStore.startUpdate()}
					>
						Atualizar Agora Automaticamente <DownloadCloud class="w-4 h-4" />
					</button>

					<button
						class="text-xs font-bold text-white/40 hover:text-white transition-colors cursor-pointer"
						onclick={() => (updaterStore.showModal = false)}
					>
						Lembrar mais tarde
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}
