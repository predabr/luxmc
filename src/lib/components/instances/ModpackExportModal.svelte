<script lang="ts">
	import { backOut, quintOut } from "svelte/easing";
	import { fade, scale } from "svelte/transition";
	import {
		Package,
		Download,
		X,
		Check,
		FolderOpen,
		Sparkles,
		FileArchive
	} from "lucide-svelte";
	import { instanceExportModpack, instanceExportShareCode, type ExportResult } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let {
		isOpen = $bindable(false),
		profileId,
		instanceName,
		onClose
	}: {
		isOpen: boolean;
		profileId: string;
		instanceName: string;
		onClose: () => void;
	} = $props();

	let exportFormat = $state<"mrpack" | "zip">("mrpack");
	let customName = $state("MeuModpack");
	let isExporting = $state(false);
	let exportResult = $state<ExportResult | null>(null);
	let shareCode = $state<string | null>(null);
	let isGeneratingCode = $state(false);

	$effect(() => {
		if (isOpen) {
			customName = instanceName || "MeuModpack";
			exportResult = null;
			shareCode = null;
		}
	});

	async function handleExport() {
		isExporting = true;
		try {
			const res = await instanceExportModpack(profileId, exportFormat, customName.trim());
			exportResult = res;
			playSound("click");
			toast(`Modpack exportado com sucesso! (${(res.fileSize / (1024 * 1024)).toFixed(1)} MB)`, "success");
		} catch (e) {
			toast("Falha na exportação: " + String(e), "error");
		} finally {
			isExporting = false;
		}
	}

	async function handleShareCode() {
		isGeneratingCode = true;
		try {
			shareCode = await instanceExportShareCode(profileId);
			await navigator.clipboard.writeText(shareCode);
			playSound("chime");
			toast("Código de modpack copiado.", "success");
		} catch (error) {
			toast("Falha ao gerar código: " + String(error), "error");
		} finally {
			isGeneratingCode = false;
		}
	}
</script>

{#if isOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-md select-none" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="w-full max-w-md rounded-3xl bg-bg-elevated border border-fg/15 p-6 shadow-2xl space-y-5" in:scale={{ easing: backOut, start: 0.95, duration: 260 }}>
			<!-- Header -->
			<div class="flex items-center justify-between border-b border-fg/10 pb-4">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center text-brand-400">
						<Package class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-sm font-black text-fg">Exportador de Modpack</h3>
						<p class="text-[11px] text-fg/50">Crie pacotes limpos para compartilhar com amigos</p>
					</div>
				</div>

				<button
					type="button"
					class="p-2 rounded-xl text-fg/40 hover:text-fg hover:bg-fg/10 transition-colors cursor-pointer"
					onclick={onClose}
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			{#if exportResult}
				<!-- Result Card -->
				<div class="p-5 rounded-2xl bg-emerald-500/10 border border-emerald-500/30 space-y-3" in:fade={{ easing: quintOut, duration: 220 }}>
					<div class="flex items-center gap-2 text-emerald-400 font-bold text-xs">
						<Check class="w-4 h-4 stroke-[3]" />
						<span>Modpack Pronto para Distribuição!</span>
					</div>
					<div class="space-y-1 text-xs text-fg/70 font-mono">
						<div>Arquivo: <span class="text-fg font-bold">{exportResult.filePath.split('/').pop()}</span></div>
						<div>Tamanho: <span class="text-fg font-bold">{(exportResult.fileSize / (1024 * 1024)).toFixed(2)} MB</span></div>
						<div>Total de Mods: <span class="text-fg font-bold">{exportResult.totalMods} mods</span></div>
					</div>
					<p class="text-[10px] text-fg/40 pt-1">
						Salvo na sua pasta de Downloads: <br />
						<span class="text-fg/60 select-all font-mono break-all">{exportResult.filePath}</span>
					</p>
				</div>
			{:else}
				<!-- Configuration -->
				<div class="space-y-4">
					<div class="space-y-1.5">
						<label for="export-name-input" class="text-xs font-bold text-fg/80 block">Nome do Arquivo</label>
						<input
							id="export-name-input"
							type="text"
							bind:value={customName}
							placeholder="Nome do Modpack"
							class="w-full px-4 py-2.5 rounded-xl bg-bg-overlay/40 border border-fg/10 text-xs text-fg focus:outline-none focus:border-brand-500 font-mono"
						/>
					</div>

					<div class="space-y-1.5">
						<span class="text-xs font-bold text-fg/80 block">Formato de Distribuição</span>
						<div class="grid grid-cols-2 gap-2.5">
							<button
								type="button"
								class="p-3 rounded-2xl border text-left transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer flex flex-col justify-between {exportFormat === 'mrpack' ? 'bg-emerald-500/15 border-emerald-500/40 text-fg' : 'bg-bg-elevated hover:bg-bg-subtle border-fg/5 text-fg/60'}"
								onclick={() => exportFormat = 'mrpack'}
							>
								<div class="flex items-center justify-between">
									<span class="text-xs font-black">.MRPACK</span>
									<span class="text-[9px] font-bold text-emerald-400 bg-emerald-500/20 px-1.5 py-0.5 rounded">Modrinth</span>
								</div>
								<span class="text-[10px] text-fg/40 mt-1.5">Padrão moderno, leve e compatível com todos os launchers modernos.</span>
							</button>

							<button
								type="button"
								class="p-3 rounded-2xl border text-left transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer flex flex-col justify-between {exportFormat === 'zip' ? 'bg-amber-500/15 border-amber-500/40 text-fg' : 'bg-bg-elevated hover:bg-bg-subtle border-fg/5 text-fg/60'}"
								onclick={() => exportFormat = 'zip'}
							>
								<div class="flex items-center justify-between">
									<span class="text-xs font-black">.ZIP</span>
									<span class="text-[9px] font-bold text-amber-400 bg-amber-500/20 px-1.5 py-0.5 rounded">CurseForge</span>
								</div>
								<span class="text-[10px] text-fg/40 mt-1.5">Estrutura clássica de manifest.json com pasta de overrides completa.</span>
							</button>
						</div>
					</div>

					<div class="p-3 rounded-2xl bg-fg/5 border border-fg/5 text-[11px] text-fg/50 leading-relaxed">
						🛡️ O Luxmc limpa automaticamente arquivos de log, relatórios de crash e dados pessoais para garantir um pacote 100% limpo e seguro.
					</div>
				</div>
			{/if}

			<div class="rounded-2xl border border-brand-500/20 bg-brand-500/5 p-4 space-y-3">
				<div><p class="text-xs font-black text-fg">Compartilhar por código</p><p class="mt-1 text-[10px] text-fg/50">Inclui versão, loader e os IDs oficiais dos mods para instalar direto no Luxmc.</p></div>
				{#if shareCode}<textarea readonly class="h-20 w-full rounded-xl border border-fg/10 bg-bg-overlay/40 p-2 font-mono text-[10px] text-fg" value={shareCode}></textarea>{/if}
				<button type="button" class="px-4 py-2 rounded-xl border border-brand-500/30 bg-brand-500/10 text-xs font-bold text-brand-400 disabled:opacity-50" disabled={isGeneratingCode} onclick={handleShareCode}>{isGeneratingCode ? 'Gerando...' : 'Gerar e copiar Share Code'}</button>
			</div>

			<!-- Actions -->
			<div class="flex items-center justify-end gap-2.5 pt-2 border-t border-fg/10">
				<button
					type="button"
					class="px-5 py-2.5 rounded-2xl bg-fg/5 hover:bg-fg/10 text-fg/60 hover:text-fg font-bold text-xs transition-colors cursor-pointer"
					onclick={onClose}
				>
					{exportResult ? 'Fechar' : 'Cancelar'}
				</button>

				{#if !exportResult}
					<button
						type="button"
						class="px-7 py-2.5 rounded-2xl bg-brand-500 hover:bg-brand-400 text-brand-foreground font-black text-xs uppercase tracking-wider transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] shadow-md cursor-pointer disabled:opacity-50 flex items-center gap-2"
						onclick={handleExport}
						disabled={isExporting}
					>
						{#if isExporting}
							<div class="w-4 h-4 rounded-full border-2 border-bg-overlay border-t-transparent animate-spin"></div>
							<span>Exportando...</span>
						{:else}
							<Download class="w-4 h-4 stroke-[2.5]" />
							<span>Exportar Agora</span>
						{/if}
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}
