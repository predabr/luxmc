<script lang="ts">
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { 
		Sliders, 
		Sun, 
		Eye, 
		Monitor, 
		Volume2, 
		Code2, 
		Save, 
		X, 
		Sparkles, 
		CheckCircle2 
	} from "lucide-svelte";
	import { 
		instanceOptionsGet, 
		instanceOptionsSet, 
		instanceConfigRead, 
		instanceConfigWrite 
	} from "$lib/api/instances";
	import type { InstanceMinecraftOptions } from "$lib/api/types";
	import { toast } from "$lib/stores/toasts.svelte";
	import Button from "./Button.svelte";
	import CodeEditor from "./CodeEditor.svelte";

	type Props = {
		open: boolean;
		profileId: string;
		onClose: () => void;
	};

	let { open, profileId, onClose }: Props = $props();

	let activeTab = $state<"visual" | "raw">("visual");
	let loading = $state(false);
	let saving = $state(false);

	let options = $state<InstanceMinecraftOptions>({
		gamma: 1.0,
		fov: 70.0,
		renderDistance: 12,
		simulationDistance: 12,
		maxFps: 120,
		guiScale: 0,
		fullscreen: false,
		vsync: false,
		autoJump: false,
		bobView: true,
		soundMaster: 1.0,
		soundMusic: 0.5,
	});

	let rawFilePath = $state("options.txt");
	let rawContent = $state("");

	async function loadOptions() {
		if (!profileId) return;
		loading = true;
		try {
			options = await instanceOptionsGet(profileId);
			const cfg = await instanceConfigRead(profileId, rawFilePath);
			rawContent = cfg.content;
		} catch (e) {
			console.warn("Failed to load options:", e);
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (open) {
			loadOptions();
		}
	});

	async function handleSaveVisual() {
		saving = true;
		try {
			await instanceOptionsSet(profileId, options);
			toast("Configurações salvas com sucesso no options.txt!", "success");
			onClose();
		} catch (e) {
			toast("Erro ao salvar opções: " + String(e), "error");
		} finally {
			saving = false;
		}
	}

	async function handleSaveRaw() {
		saving = true;
		try {
			await instanceConfigWrite(profileId, rawFilePath, rawContent);
			toast(`Arquivo "${rawFilePath}" gravado com sucesso!`, "success");
			await loadOptions();
		} catch (e) {
			toast("Erro ao salvar arquivo: " + String(e), "error");
		} finally {
			saving = false;
		}
	}
</script>

{#if open}
	<div 
		class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md"
		transition:fade={{ duration: 180 }}
	>
		<div 
			class="relative w-full max-w-xl bg-bg-elevated border border-white/10 rounded-3xl p-6 shadow-2xl overflow-hidden flex flex-col gap-5 max-h-[90vh]"
			transition:scale={{ start: 0.95, duration: 200 }}
		>
			<!-- Top Aura -->
			<div class="absolute -top-24 -left-20 w-48 h-48 bg-brand-500/15 rounded-full blur-3xl pointer-events-none"></div>

			<!-- Header -->
			<div class="flex items-start justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-500/20 border border-brand-500/30 flex items-center justify-center text-brand-400">
						<Sliders class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-bold text-white">Editor Visual de Configurações</h3>
						<p class="text-xs text-white/50">Ajuste Gamma / Fullbright, Render Distance e arquivos de configuração</p>
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

			<!-- Tab switch -->
			<div class="flex bg-bg-subtle p-1 rounded-2xl border border-white/5">
				<button 
					type="button"
					class="flex-1 py-1.5 text-xs font-bold rounded-xl transition-all cursor-pointer {activeTab === 'visual' ? 'bg-bg-overlay text-white shadow-sm' : 'text-white/40 hover:text-white'}"
					onclick={() => activeTab = 'visual'}
				>
					Ajustes Visuais Rápidos
				</button>
				<button 
					type="button"
					class="flex-1 py-1.5 text-xs font-bold rounded-xl transition-all cursor-pointer {activeTab === 'raw' ? 'bg-bg-overlay text-white shadow-sm' : 'text-white/40 hover:text-white'}"
					onclick={() => activeTab = 'raw'}
				>
					Editor de Código / Arquivo
				</button>
			</div>

			{#if activeTab === "visual"}
				<div class="flex-1 overflow-y-auto space-y-4 custom-scrollbar pr-1">
					<!-- Fullbright / Gamma Slider -->
					<div class="bg-bg-subtle border border-white/5 rounded-2xl p-4 flex flex-col gap-2.5">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2 text-xs font-bold text-white">
								<Sun class="w-4 h-4 text-amber-400" />
								Brilho & Gamma (Fullbright)
							</div>
							<span class="text-xs font-mono font-bold text-amber-400">
								{options.gamma <= 1.0 ? `${Math.round(options.gamma * 100)}%` : `Fullbright (${options.gamma.toFixed(1)}x)`}
							</span>
						</div>
						<input 
							type="range" 
							min="0" 
							max="5" 
							step="0.05" 
							bind:value={options.gamma}
							class="w-full accent-amber-400 cursor-pointer"
						/>
						<div class="flex justify-between text-[10px] text-white/40 font-semibold">
							<span>Sombrio (0%)</span>
							<span>Padrão Claro (100%)</span>
							<span>Visão Noturna / Fullbright (500%)</span>
						</div>
					</div>

					<!-- FOV Slider -->
					<div class="bg-bg-subtle border border-white/5 rounded-2xl p-4 flex flex-col gap-2.5">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2 text-xs font-bold text-white">
								<Eye class="w-4 h-4 text-emerald-400" />
								Campo de Visão (FOV)
							</div>
							<span class="text-xs font-mono font-bold text-emerald-400">
								{Math.round(options.fov)}° {options.fov >= 110 ? '(Quake Pro)' : ''}
							</span>
						</div>
						<input 
							type="range" 
							min="30" 
							max="110" 
							step="1" 
							bind:value={options.fov}
							class="w-full accent-emerald-400 cursor-pointer"
						/>
					</div>

					<!-- Render & Simulation Distance -->
					<div class="grid grid-cols-2 gap-3">
						<div class="bg-bg-subtle border border-white/5 rounded-2xl p-3.5 flex flex-col gap-2">
							<div class="flex items-center justify-between text-xs font-bold text-white">
								<span>Renderização</span>
								<span class="text-blue-400">{options.renderDistance} chunks</span>
							</div>
							<input 
								type="range" 
								min="2" 
								max="32" 
								step="1" 
								bind:value={options.renderDistance}
								class="w-full accent-blue-400 cursor-pointer"
							/>
						</div>

						<div class="bg-bg-subtle border border-white/5 rounded-2xl p-3.5 flex flex-col gap-2">
							<div class="flex items-center justify-between text-xs font-bold text-white">
								<span>Limite de FPS</span>
								<span class="text-purple-400">{options.maxFps >= 260 ? 'Ilimitado' : `${options.maxFps} FPS`}</span>
							</div>
							<input 
								type="range" 
								min="30" 
								max="260" 
								step="10" 
								bind:value={options.maxFps}
								class="w-full accent-purple-400 cursor-pointer"
							/>
						</div>
					</div>

					<!-- Toggles Grid -->
					<div class="grid grid-cols-3 gap-2.5">
						<button 
							type="button"
							class="p-3 rounded-2xl border text-xs font-bold transition-all flex flex-col gap-1 items-center justify-center cursor-pointer {options.vsync ? 'bg-emerald-500/15 border-emerald-500/30 text-emerald-300' : 'bg-bg-subtle border-white/5 text-white/50'}"
							onclick={() => options.vsync = !options.vsync}
						>
							<span>VSync</span>
							<span class="text-[10px] font-semibold">{options.vsync ? 'Ativado' : 'Desativado'}</span>
						</button>

						<button 
							type="button"
							class="p-3 rounded-2xl border text-xs font-bold transition-all flex flex-col gap-1 items-center justify-center cursor-pointer {options.autoJump ? 'bg-amber-500/15 border-amber-500/30 text-amber-300' : 'bg-bg-subtle border-white/5 text-white/50'}"
							onclick={() => options.autoJump = !options.autoJump}
						>
							<span>Pulo Automático</span>
							<span class="text-[10px] font-semibold">{options.autoJump ? 'Ativado' : 'Desativado'}</span>
						</button>

						<button 
							type="button"
							class="p-3 rounded-2xl border text-xs font-bold transition-all flex flex-col gap-1 items-center justify-center cursor-pointer {options.bobView ? 'bg-blue-500/15 border-blue-500/30 text-blue-300' : 'bg-bg-subtle border-white/5 text-white/50'}"
							onclick={() => options.bobView = !options.bobView}
						>
							<span>Balanço da Câmera</span>
							<span class="text-[10px] font-semibold">{options.bobView ? 'Ativado' : 'Desativado'}</span>
						</button>
					</div>
				</div>

				<!-- Actions -->
				<div class="pt-3 border-t border-white/5 flex items-center justify-end gap-2.5">
					<Button variant="secondary" size="sm" onclick={onClose}>
						Cancelar
					</Button>
					<Button variant="primary" size="sm" disabled={saving} onclick={handleSaveVisual}>
						<Save class="w-3.5 h-3.5" />
						{saving ? "Salvando..." : "Salvar Configurações"}
					</Button>
				</div>
			{:else}
				<div class="flex flex-col gap-3 flex-1 overflow-hidden">
					<div class="flex items-center gap-2">
						<label for="config-file-path" class="text-xs font-semibold text-white/60 shrink-0">Arquivo:</label>
						<input 
							id="config-file-path"
							type="text" 
							bind:value={rawFilePath}
							class="flex-1 bg-bg-subtle border border-white/10 rounded-xl px-3 py-1.5 text-xs text-white font-mono outline-none"
						/>
						<Button variant="secondary" size="sm" onclick={async () => {
							const c = await instanceConfigRead(profileId, rawFilePath);
							rawContent = c.content;
							toast("Arquivo carregado!", "info");
						}}>
							Recarregar
						</Button>
					</div>

					<div class="w-full flex-1 min-h-[300px] overflow-hidden">
						<CodeEditor bind:value={rawContent} />
					</div>

					<div class="pt-2 border-t border-white/5 flex items-center justify-end gap-2.5">
						<Button variant="secondary" size="sm" onclick={onClose}>
							Fechar
						</Button>
						<Button variant="primary" size="sm" disabled={saving} onclick={handleSaveRaw}>
							<Save class="w-3.5 h-3.5" />
							{saving ? "Gravando..." : "Gravar Arquivo"}
						</Button>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}
