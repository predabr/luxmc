<script lang="ts">
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { DownloadCloud, Sparkles, ArrowRight, X, Loader2, ExternalLink, Copy } from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { updaterStore } from "$lib/stores/updater.svelte";

	onMount(() => {
		const timer = setTimeout(() => {
			updaterStore.check(false);
		}, 3000);
		const interval = setInterval(() => {
			updaterStore.check(false);
		}, 1800000);
		return () => {
			clearTimeout(timer);
			clearInterval(interval);
		};
	});
</script>

{#if updaterStore.showModal}
	<div
		class="fixed inset-0 z-[100] flex items-center justify-center p-6 bg-bg-overlay/80 backdrop-blur-md"
		in:fade={{ duration: 300 }}
		out:fade={{ duration: 200 }}
	>
		<div
			class="surface-glass relative w-full max-w-lg border-brand-500/30 bg-bg-elevated/85 p-8 shadow-2xl backdrop-blur-2xl overflow-hidden"
			in:scale={{ start: 0.95, duration: 300, opacity: 0 }}
		>
			<div class="absolute -top-20 -right-20 w-64 h-64 bg-brand-500/20 rounded-full blur-3xl pointer-events-none"></div>
			<div class="absolute -bottom-20 -left-20 w-64 h-64 bg-brand-500/10 rounded-full blur-3xl pointer-events-none"></div>

			{#if !updaterStore.isUpdating}
				<button
					class="absolute top-4 right-4 p-2 rounded-full text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors"
					onclick={() => (updaterStore.showModal = false)}
				>
					<X class="w-5 h-5" />
				</button>
			{/if}

			<div class="relative flex flex-col items-center text-center space-y-6">
				<div
					class="w-20 h-20 bg-brand-500/15 border-2 border-brand-500/30 rounded-full flex items-center justify-center shadow-glow"
				>
					{#if updaterStore.isUpdating}
						<Loader2 class="w-10 h-10 text-brand-500 animate-spin" />
					{:else}
						<DownloadCloud class="w-10 h-10 text-brand-500" />
					{/if}
				</div>

				<div>
					<h2 class="text-2xl font-black text-fg flex items-center justify-center gap-2">
						{#if updaterStore.isUpdating}
							Atualizando o Luxmc...
						{:else}
							<Sparkles class="w-5 h-5 text-brand-500" /> Nova Versão Disponível!
						{/if}
					</h2>
					<p class="text-sm text-fg/60 mt-2">
						{#if updaterStore.isUpdating}
							{updaterStore.statusText || "Baixando e instalando a atualização..."}
						{:else}
							O Luxmc ficou ainda melhor. Atualize agora sem perder suas instâncias e configurações!
						{/if}
					</p>
				</div>

				<div class="flex items-center justify-center gap-4 w-full bg-bg-overlay/40 rounded-2xl p-4 border border-fg/5">
					<div class="flex flex-col items-center">
						<span class="text-[10px] text-fg/40 font-bold uppercase tracking-widest">Sua Versão</span>
						<span class="text-lg font-mono text-fg/80 font-bold">v{updaterStore.currentVersion}</span>
					</div>
					<ArrowRight class="w-5 h-5 text-fg/30" />
					<div class="flex flex-col items-center">
						<span class="text-[10px] text-brand-500 font-bold uppercase tracking-widest">Nova Versão</span>
						<span class="text-lg font-mono text-brand-500 font-black">v{updaterStore.latestVersion}</span>
					</div>
				</div>

				{#if updaterStore.isUpdating}
					<div class="w-full space-y-3 bg-bg-overlay/30 p-4 rounded-2xl border border-fg/5">
						<div class="flex justify-between text-xs font-bold text-fg/70">
							<span>{updaterStore.statusText}</span>
							<span class="text-brand-500 font-mono">{updaterStore.progressPercent}%</span>
						</div>
						<p class="text-xs font-mono text-fg/50">{(updaterStore.transferredBytes / 1048576).toFixed(1)} MB{updaterStore.totalBytes > 0 ? ` / ${(updaterStore.totalBytes / 1048576).toFixed(1)} MB` : ""}</p>
						<div class="w-full bg-fg/10 rounded-full h-3 overflow-hidden">
							<div
								class="bg-brand-500 h-full rounded-full transition-all duration-300"
								style="width: {updaterStore.progressPercent}%;"
							></div>
						</div>
					</div>
				{:else}
					<div class="w-full text-left bg-fg/5 rounded-2xl p-4 max-h-44 overflow-y-auto custom-scrollbar border border-fg/5">
						<span class="text-[10px] text-fg/40 font-bold uppercase tracking-widest block mb-2">Novidades:</span>
						<p class="text-xs text-fg/80 whitespace-pre-line leading-relaxed">
							{updaterStore.releaseNotes}
						</p>
					</div>
					{#if updaterStore.updateError}
						<div class="w-full rounded-2xl border border-danger/30 bg-danger/10 p-4 text-left"><p class="text-xs font-bold text-danger">Não foi possível concluir a atualização</p><p class="mt-1 text-xs text-fg/65 break-words">{updaterStore.updateError}</p></div>
					{/if}
					{#if updaterStore.terminalCommand}
						<div class="w-full rounded-2xl border border-fg/10 bg-bg-overlay/50 p-3 text-left"><p class="text-[10px] font-bold uppercase tracking-wider text-fg/45">Atualizar via Terminal</p><code class="mt-2 block break-all text-xs text-fg/80">{updaterStore.terminalCommand}</code><button type="button" class="mt-3 inline-flex items-center gap-2 text-xs font-bold text-brand-400 hover:text-brand-300" onclick={() => navigator.clipboard.writeText(updaterStore.terminalCommand)}><Copy class="w-3.5 h-3.5" />Copiar comando</button></div>
					{/if}

					{#if updaterStore.downloadUrl}
					<button
						class="w-full py-3.5 bg-brand-500 hover:brightness-110 text-brand-foreground font-black rounded-2xl transition-all hover:scale-[1.02] active:scale-[0.98] flex items-center justify-center gap-2 shadow-glow cursor-pointer"
						onclick={() => updaterStore.startUpdate()}
					>
						Atualizar Agora Automaticamente <DownloadCloud class="w-4 h-4" />
					</button>
					{/if}
					<button class="w-full py-3 border border-fg/10 bg-fg/[0.04] hover:bg-fg/[0.08] text-fg font-bold rounded-2xl transition-all active:scale-[0.97] flex items-center justify-center gap-2 cursor-pointer" onclick={() => openUrl(updaterStore.releaseUrl || "https://github.com/predabr/luxmc/releases/latest")}>Abrir no GitHub <ExternalLink class="w-4 h-4" /></button>

					<button
						class="text-xs font-bold text-fg/40 hover:text-fg transition-colors cursor-pointer"
						onclick={() => (updaterStore.showModal = false)}
					>
						Lembrar mais tarde
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}
