<script lang="ts">
	import { fade, slide } from "svelte/transition";
	import FilterableVersionSelect from "$lib/components/ui/FilterableVersionSelect.svelte";
	import {
		Plus,
		Check,
		X,
		AlertTriangle,
		Sparkles,
		Zap,
		Cpu,
		FolderOpen,
		ChevronDown,
		ChevronUp,
		Sliders,
		Upload,
		RefreshCw,
		Palette,
		ArrowLeft
	} from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";

	const { t } = useTranslation();

	interface Props {
		isOpen: boolean;
		onClose?: () => void;
		versions: Array<{ id: string; versionType: string; releaseTime: string }>;
		versionsLoading?: boolean;
		systemRamMb?: number;
		onCreate?: (input: CreateInput) => Promise<void>;
	}

	interface CreateInput {
		name: string;
		version: string;
		loader: string;
		icon: string;
		ramGb: number;
		autoOptimize: boolean;
		useVulkan: boolean;
		installPerfPack: boolean;
		resolutionW?: number;
		resolutionH?: number;
		fullscreen?: boolean;
		javaPath?: string;
		jvmArgs?: string;
		gameDir?: string;
	}

	let {
		isOpen = $bindable(false),
		onClose,
		versions = [],
		versionsLoading = false,
		systemRamMb = 8192,
		onCreate
	}: Props = $props();

	let newName = $state("Fabric 1.21.4");
	let newVersion = $state("1.21.4");
	let newLoader = $state("fabric");
	let loaderReleaseType = $state<"stable" | "latest" | "other">("stable");
	let newIcon = $state("grass_block");
	let showIconPicker = $state(false);

	let selectedRamGb = $state(4);
	let newAutoOptimize = $state(true);
	let newUseVulkan = $state(false);
	let newInstallPerfPack = $state(true);
	let showAdvanced = $state(false);
	let customResW = $state(settings.value.defaultResWidth ?? 1920);
	let customResH = $state(settings.value.defaultResHeight ?? 1080);
	let customFullscreen = $state(settings.value.startFullscreen ?? true);
	let customJavaPath = $state("");
	let customJvmArgs = $state("");
	let customGameDir = $state("");
	let creating = $state(false);
	let lastError = $state<string | null>(null);
	let createSuccess = $state(false);

	const iconPresets = [
		{ id: "grass_block", label: "Bloco de Grama", src: "/grass_block.png" },
		{ id: "modpack_fo", label: "Fabulously Optimized", src: "/modpack_fo_icon.png" },
		{ id: "modpack_better_mc", label: "Better MC", src: "/modpack_bmc_icon.webp" },
		{ id: "modpack_cobblemon", label: "Cobblemon", src: "/modpack_cobblemon_icon.png" },
		{ id: "logo", label: "Luxmc Logo", src: "/logo.png" },
		{ id: "grass_head", label: "Steve", src: "/grass_head.png" }
	];

	function getIconSrc(iconStr?: string): string {
		if (!iconStr || iconStr === "grass_block" || iconStr === "/grass_block" || iconStr === "grass" || iconStr === "/grass") return "/grass_block.png";
		if (iconStr === "modpack_fo") return "/modpack_fo_icon.png";
		if (iconStr === "modpack_better_mc") return "/modpack_bmc_icon.webp";
		if (iconStr === "modpack_cobblemon") return "/modpack_cobblemon_icon.png";
		if (iconStr === "logo") return "/logo.png";
		if (iconStr === "grass_head") return "/grass_head.png";
		if (iconStr.startsWith("/") || iconStr.startsWith("http") || iconStr.startsWith("data:") || iconStr.startsWith("asset:")) return iconStr;
		return "/grass_block.png";
	}

	async function handleCustomIconUpload() {
		try {
			const file = await open({
				title: "Selecione uma imagem para o ícone",
				multiple: false,
				filters: [{ name: "Imagens", extensions: ["png", "jpg", "jpeg", "webp"] }]
			});
			if (file && typeof file === "string") {
				newIcon = convertFileSrc(file);
				showIconPicker = false;
				toast("Ícone personalizado carregado!", "success");
			}
		} catch (e) {
			toast("Erro ao carregar imagem: " + String(e), "error");
		}
	}

	function handleRandomizeIcon() {
		const list = ["grass_block", "modpack_fo", "modpack_better_mc", "modpack_cobblemon", "logo", "grass_head"];
		const currentIdx = list.indexOf(newIcon);
		const nextIdx = (currentIdx + 1 + Math.floor(Math.random() * (list.length - 1))) % list.length;
		newIcon = list[nextIdx];
	}

	const loaderOptions = [
		{ id: 'vanilla', name: 'Vanilla' },
		{ id: 'fabric', name: 'Fabric' },
		{ id: 'neoforge', name: 'NeoForge' },
		{ id: 'forge', name: 'Forge' },
		{ id: 'quilt', name: 'Quilt' }
	];

	function selectLoader(ldrId: string) {
		newLoader = ldrId;
		const ldr = loaderOptions.find(l => l.id === ldrId);
		const ldrName = ldr?.name || "Vanilla";
		newName = ldrId === 'vanilla' ? `Vanilla ${newVersion}` : `${ldrName} ${newVersion}`;
	}

	function handleVersionChange(ver: string) {
		const ldr = loaderOptions.find(l => l.id === newLoader);
		const ldrName = ldr?.name || "Vanilla";
		newName = newLoader === 'vanilla' ? `Vanilla ${ver}` : `${ldrName} ${ver}`;
	}

	function resetForm() {
		newName = "Fabric 1.21.4";
		newLoader = "fabric";
		newVersion = "1.21.4";
		newIcon = "grass_block";
		loaderReleaseType = "stable";
		newAutoOptimize = true;
		newUseVulkan = false;
		newInstallPerfPack = true;
		lastError = null;
		createSuccess = false;
		showIconPicker = false;
		showAdvanced = false;
	}

	function handleClose() {
		resetForm();
		onClose?.();
	}

	async function handleCreate() {
		if (!newName.trim()) return;
		creating = true;
		lastError = null;
		createSuccess = false;
		try {
			await onCreate?.({
				name: newName.trim(),
				version: newVersion,
				loader: newLoader,
				icon: newIcon,
				ramGb: selectedRamGb,
				autoOptimize: newAutoOptimize,
				useVulkan: newUseVulkan,
				installPerfPack: newInstallPerfPack,
				resolutionW: customResW,
				resolutionH: customResH,
				fullscreen: customFullscreen,
				javaPath: customJavaPath.trim() || undefined,
				jvmArgs: customJvmArgs.trim() || undefined,
				gameDir: customGameDir.trim() || undefined,
			});
			createSuccess = true;
			toast(t("instances.createdSuccess"), "success");
			setTimeout(() => handleClose(), 1200);
		} catch (e) {
			lastError = "Falha ao criar instância: " + String(e);
			toast(lastError, "error");
		} finally {
			creating = false;
		}
	}
</script>

{#if isOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-bg-overlay/80 backdrop-blur-md p-4 overflow-y-auto"
		transition:fade={{ duration: 150 }}
		onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
		onkeydown={(e) => { if (e.key === "Escape") handleClose(); }}
		role="dialog"
		tabindex="-1"
		aria-modal="true"
	>
		<div
			class="rounded-3xl bg-bg-elevated border border-fg/10 p-6 shadow-2xl space-y-5 select-none max-w-lg w-full max-h-[92vh] overflow-y-auto custom-scrollbar my-auto"
			in:fade={{ duration: 180 }}
		>
			<!-- Header -->
			<div class="flex items-center justify-between">
				<h3 class="text-base font-bold text-fg tracking-tight">Criar instância</h3>
				<button
					type="button"
					class="h-8 w-8 rounded-full bg-fg/5 hover:bg-fg/10 text-fg/60 hover:text-fg flex items-center justify-center transition-all cursor-pointer"
					onclick={handleClose}
					title="Fechar"
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			{#if createSuccess}
				<div class="flex items-center gap-3 rounded-2xl p-4 text-sm bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 font-bold">
					<Check class="h-5 w-5" />
					Instância "{newName}" criada com sucesso! Redirecionando...
				</div>
			{:else}
				<!-- Top: Icon & 3 Stacked Buttons -->
				<div class="flex items-center gap-4">
					<div class="w-20 h-20 rounded-2xl bg-bg-subtle border border-fg/10 flex items-center justify-center p-2 shrink-0 shadow-inner overflow-hidden">
						<img
							src={getIconSrc(newIcon)}
							alt="Ícone da Instância"
							class="w-14 h-14 object-contain [image-rendering:pixelated]"
						/>
					</div>

					<div class="flex flex-col gap-1.5 flex-1">
						<button
							type="button"
							onclick={handleCustomIconUpload}
							class="h-8 px-3 rounded-xl bg-bg-subtle hover:bg-fg/10 border border-fg/10 text-xs font-semibold text-fg/80 hover:text-fg flex items-center gap-2 transition-colors cursor-pointer"
						>
							<Upload class="w-3.5 h-3.5" /> Enviar
						</button>
						<button
							type="button"
							onclick={handleRandomizeIcon}
							class="h-8 px-3 rounded-xl bg-bg-subtle hover:bg-fg/10 border border-fg/10 text-xs font-semibold text-fg/80 hover:text-fg flex items-center gap-2 transition-colors cursor-pointer"
						>
							<RefreshCw class="w-3.5 h-3.5" /> Aleatorizar
						</button>
						<button
							type="button"
							onclick={() => showIconPicker = !showIconPicker}
							class="h-8 px-3 rounded-xl bg-bg-subtle hover:bg-fg/10 border border-fg/10 text-xs font-semibold text-fg/80 hover:text-fg flex items-center gap-2 transition-colors cursor-pointer"
						>
							<Palette class="w-3.5 h-3.5" /> Personalizar
						</button>
					</div>
				</div>

				<!-- Icon Picker Popover / Drawer -->
				{#if showIconPicker}
					<div class="p-3 bg-bg-subtle rounded-2xl border border-fg/10 space-y-2" transition:slide={{ duration: 150 }}>
						<span class="text-[11px] font-bold text-fg/70 block">Escolha um ícone pré-definido:</span>
						<div class="flex items-center gap-2 flex-wrap">
							{#each iconPresets as ip}
								<button
									type="button"
									onclick={() => { newIcon = ip.id; showIconPicker = false; }}
									class="w-9 h-9 rounded-xl p-1.5 transition-all cursor-pointer flex items-center justify-center border {newIcon === ip.id ? 'bg-emerald-500/20 border-emerald-500' : 'bg-bg-elevated border-fg/10 hover:border-fg/30'}"
									title={ip.label}
								>
									<img src={ip.src} alt={ip.label} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
								</button>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Nome -->
				<div class="space-y-1.5">
					<label for="instance-name" class="block text-xs font-bold text-fg/80">Nome</label>
					<input
						id="instance-name"
						type="text"
						bind:value={newName}
						class="w-full h-10 rounded-xl px-3.5 text-xs font-medium text-fg bg-bg-subtle border border-fg/10 focus:border-emerald-500 outline-none transition-all"
						placeholder="ex: Fabric 1.21.4"
					/>
				</div>

				<!-- Loader -->
				<div class="space-y-1.5">
					<span class="block text-xs font-bold text-fg/80">Loader</span>
					<div class="flex flex-wrap gap-2">
						{#each loaderOptions as ldr}
							{@const isSelected = newLoader === ldr.id}
							<button
								type="button"
								onclick={() => selectLoader(ldr.id)}
								class="px-3.5 py-1.5 rounded-xl text-xs font-medium border transition-all flex items-center gap-1.5 cursor-pointer {isSelected ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/50 shadow-sm' : 'bg-bg-subtle border-fg/10 text-fg/70 hover:text-fg hover:border-fg/20'}"
							>
								{#if isSelected}
									<Check class="w-3.5 h-3.5 stroke-[2.5]" />
								{/if}
								{ldr.name}
							</button>
						{/each}
					</div>
				</div>

				<!-- Versão do jogo -->
				<div class="space-y-1.5">
					<span class="block text-xs font-bold text-fg/80">Versão do jogo</span>
					<FilterableVersionSelect
						{versions}
						bind:value={newVersion}
						loading={versionsLoading}
						onChange={handleVersionChange}
					/>
				</div>

				<!-- Versão do loader (quando loader != vanilla) -->
				{#if newLoader !== 'vanilla'}
					<div class="space-y-1.5">
						<span class="block text-xs font-bold text-fg/80">Versão do loader</span>
						<div class="flex gap-2">
							{#each [
								{ id: 'stable' as const, label: 'Estável' },
								{ id: 'latest' as const, label: 'Última' },
								{ id: 'other' as const, label: 'Outra' }
							] as lv}
								{@const isSelected = loaderReleaseType === lv.id}
								<button
									type="button"
									onclick={() => loaderReleaseType = lv.id}
									class="px-3.5 py-1.5 rounded-xl text-xs font-medium border transition-all flex items-center gap-1.5 cursor-pointer {isSelected ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/50 shadow-sm' : 'bg-bg-subtle border-fg/10 text-fg/70 hover:text-fg hover:border-fg/20'}"
								>
									{#if isSelected}
										<Check class="w-3.5 h-3.5 stroke-[2.5]" />
									{/if}
									{lv.label}
								</button>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Opções Avançadas (Sanfonada) -->
				<div class="pt-2 border-t border-fg/5 space-y-2">
					<button
						type="button"
						onclick={() => showAdvanced = !showAdvanced}
						class="w-full flex items-center justify-between text-left text-xs font-semibold text-fg/50 hover:text-fg transition-colors cursor-pointer py-1"
					>
						<div class="flex items-center gap-1.5">
							<Sliders class="w-3.5 h-3.5" />
							<span>Opções avançadas (RAM, Otimizações, Java)</span>
						</div>
						{#if showAdvanced}
							<ChevronUp class="w-3.5 h-3.5" />
						{:else}
							<ChevronDown class="w-3.5 h-3.5" />
						{/if}
					</button>

					{#if showAdvanced}
						<div class="space-y-3 pt-2" transition:slide={{ duration: 150 }}>
							<!-- Memória RAM -->
							<div class="space-y-1.5">
								<div class="flex justify-between text-xs">
									<span class="font-bold text-fg/80">Alocação de RAM</span>
									<span class="font-mono text-emerald-400 font-bold">{selectedRamGb} GB</span>
								</div>
								<input
									type="range"
									min="2"
									max={Math.max(8, Math.floor(systemRamMb / 1024))}
									step="1"
									bind:value={selectedRamGb}
									class="w-full accent-emerald-500 cursor-pointer"
								/>
							</div>

							<div class="space-y-2 pt-1">
								<label class="flex items-center gap-2 text-xs text-fg/80 cursor-pointer">
									<input type="checkbox" bind:checked={newAutoOptimize} class="accent-emerald-500 rounded" />
									<span>Aikar G1GC Flags (Eliminar microtravamentos)</span>
								</label>
								{#if newLoader !== "vanilla"}
									<label class="flex items-center gap-2 text-xs text-fg/80 cursor-pointer">
										<input type="checkbox" bind:checked={newInstallPerfPack} class="accent-emerald-500 rounded" />
										<span>Instalar Sodium / Iris / Lithium automaticamente</span>
									</label>
								{/if}
							</div>
						</div>
					{/if}
				</div>

				<div class="flex items-center justify-end gap-3 pt-3">
					<button
						type="button"
						onclick={handleClose}
						class="px-4 py-2 rounded-xl text-xs font-bold text-fg/70 hover:text-fg hover:bg-fg/5 transition-colors cursor-pointer flex items-center gap-1.5"
					>
						<ArrowLeft class="w-3.5 h-3.5" /> Voltar
					</button>
					<button
						type="button"
						disabled={creating || !newName.trim()}
						onclick={handleCreate}
						class="px-5 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-black font-extrabold text-xs flex items-center gap-2 shadow-lg shadow-emerald-500/20 active:scale-[0.98] transition-all cursor-pointer disabled:opacity-50"
					>
						{#if creating}
							<div class="w-4 h-4 rounded-full border-2 border-black border-t-transparent animate-spin"></div>
							Criando...
						{:else}
							<Plus class="w-4 h-4 stroke-[3]" /> Criar instância
						{/if}
					</button>
				</div>
			{/if}

			{#if lastError}
				<div class="flex items-center justify-between rounded-xl px-4 py-2.5 text-xs bg-red-500/10 border border-red-500/30 text-red-400 font-bold">
					<div class="flex items-center gap-2">
						<AlertTriangle class="h-4 w-4 shrink-0" />
						{lastError}
					</div>
					<button
						type="button"
						class="p-1 rounded hover:bg-fg/10 cursor-pointer"
						onclick={() => (lastError = null)}
					>
						<X class="h-3.5 h-3.5" />
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}
