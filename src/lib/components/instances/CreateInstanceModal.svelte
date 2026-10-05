<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { quintOut } from "svelte/easing";
	import { fade, slide } from "svelte/transition";
	import { focusTrap } from "$lib/utils/focusTrap";
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
	import { getIconSrc } from "$lib/utils/icons";
	import { loadersVersions } from "$lib/api/launch";

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
		loaderVersion?: string;
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

	let newName = $state("Vanilla");
	let newVersion = $state("");
	let newLoader = $state("vanilla");
	let loaderReleaseType = $state<"stable" | "latest" | "other">("stable");
	let availableLoaderVersions = $state<Array<{ id: string; stable: boolean }>>([]);
	let selectedLoaderVersion = $state("");
	let loaderVersionsLoading = $state(false);
	let loaderVersionError = $state("");
	let newIcon = $state("grass_block");
	let showIconPicker = $state(false);

	let selectedRamGb = $state(4);
	let newAutoOptimize = $state(true);
	let newUseVulkan = $state(false);
	let newInstallPerfPack = $state(false);
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

	const iconPresets = $derived([
		{ id: "grass_block", label: uiText("ui.5ceea7c54da592b2"), src: "/grass_block.png" },
		{ id: "modpack_fo", label: uiText("ui.91ad8c25ff27e75e"), src: "/modpack_fo_icon.png" },
		{ id: "modpack_better_mc", label: uiText("ui.0ac0494022a49a66"), src: "/modpack_bmc_icon.webp" },
		{ id: "modpack_cobblemon", label: uiText("ui.7f1c6e0fb0701436"), src: "/modpack_cobblemon_icon.png" },
		{ id: "logo", label: uiText("ui.3e3da60bcc8c4452"), src: "/logo.png" },
		{ id: "grass_head", label: "Steve", src: "/grass_head.png" }
	]);

	async function handleCustomIconUpload() {
		try {
			const file = await open({
				title: uiText("ui.d59c8a2192bc82cd"),
				multiple: false,
				filters: [{ name: "Imagens", extensions: ["png", "jpg", "jpeg", "webp"] }]
			});
			if (file && typeof file === "string") {
				newIcon = convertFileSrc(file);
				showIconPicker = false;
				toast(uiText("ui.8e20eafc85920ffa"), "success");
			}
		} catch (e) {
			toast(uiText("ui.30db10065a2fd909") + String(e), "error");
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

	$effect(() => {
		const loader = newLoader;
		const version = newVersion;
		if (!isOpen || loader === "vanilla" || !version) {
			availableLoaderVersions = [];
			loaderVersionsLoading = false;
			loaderVersionError = "";
			return;
		}
		let active = true;
		availableLoaderVersions = [];
		loaderVersionsLoading = true;
		loaderVersionError = "";
		void loadersVersions(loader, version).then(result => {
			if (!active) return;
			availableLoaderVersions = result.versions;
			if (!result.versions.some(candidate => candidate.id === selectedLoaderVersion)) {
				selectedLoaderVersion = result.versions[0]?.id ?? "";
			}
			if (!result.versions.length) loaderVersionError = uiText("ui.4a1b977af3b86f73", {arg0: (loader), arg1: (version)});
		}).catch(error => {
			if (active) loaderVersionError = uiText("ui.dbaeff1adfd85859", {arg0: (loader), arg1: (String(error))});
		}).finally(() => { if (active) loaderVersionsLoading = false; });
		return () => { active = false; };
	});

	function selectLoader(ldrId: string) {
        versionTouched = true;
		newLoader = ldrId;
		const ldr = loaderOptions.find(l => l.id === ldrId);
		const ldrName = ldr?.name || "Vanilla";
		newName = ldrId === 'vanilla' ? `Vanilla ${newVersion}` : `${ldrName} ${newVersion}`;
	}

	function handleVersionChange(ver: string) {
        versionTouched = true;
		const ldr = loaderOptions.find(l => l.id === newLoader);
		const ldrName = ldr?.name || "Vanilla";
		newName = newLoader === 'vanilla' ? `Vanilla ${ver}` : `${ldrName} ${ver}`;
	}

    let wasOpen = false;
    let versionTouched = false;
    $effect(() => {
        if (isOpen && !wasOpen) { resetForm(); versionTouched = false; }
        if (isOpen && !versionTouched && newLoader === "vanilla") {
            const latest = versions.filter(version => version.versionType === "release").toSorted((a,b) => b.releaseTime.localeCompare(a.releaseTime))[0]?.id;
            if (latest) { newVersion = latest; newName = `Vanilla ${latest}`; }
        }
        wasOpen = isOpen;
    });
	function resetForm() {
		newVersion = versions.filter(version => version.versionType === "release").toSorted((a,b) => b.releaseTime.localeCompare(a.releaseTime))[0]?.id || "";
        newLoader = "vanilla";
        newName = newVersion ? `Vanilla ${newVersion}` : "Vanilla";
		newIcon = "grass_block";
		loaderReleaseType = "stable";
		selectedLoaderVersion = "";
		newAutoOptimize = true;
		newUseVulkan = false;
		newInstallPerfPack = false;
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
		if (!newName.trim() || !newVersion || versionsLoading) return;
		const chosenLoaderVersion = loaderReleaseType === "other"
			? selectedLoaderVersion
			: (loaderReleaseType === "stable"
				? availableLoaderVersions.find(candidate => candidate.stable)?.id ?? availableLoaderVersions[0]?.id
				: availableLoaderVersions[0]?.id);
		if (newLoader !== "vanilla" && !chosenLoaderVersion) {
			lastError = loaderVersionError || uiText("ui.53a3efc325fa11fd");
			return;
		}
		creating = true;
		lastError = null;
		createSuccess = false;
		try {
			await onCreate?.({
				name: newName.trim(),
				version: newVersion,
				loader: newLoader,
				loaderVersion: newLoader === "vanilla" ? undefined : chosenLoaderVersion,
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
			lastError = uiText("ui.ba3d218390288dc6") + String(e);
			toast(lastError, "error");
		} finally {
			creating = false;
		}
	}
</script>

{#if isOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-bg-overlay/80 backdrop-blur-md p-4 overflow-y-auto"
		transition:fade={{ easing: quintOut, duration: 220 }}
		use:focusTrap
		onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
		onkeydown={(e) => { if (e.key === "Escape") { e.stopPropagation(); handleClose(); } }}
		role="dialog"
		tabindex="-1"
		aria-modal="true"
		aria-label={uiText("mods.createInstance")}
	>
		<div
			class="rounded-3xl bg-bg-elevated border border-fg/10 p-6 shadow-2xl space-y-5 select-none max-w-lg w-full max-h-[92vh] overflow-y-auto custom-scrollbar my-auto"
			in:fade={{ easing: quintOut, duration: 240 }}
		>
			<!-- Header -->
			<div class="flex items-center justify-between">
				<h3 class="text-base font-bold text-fg tracking-tight">{uiText("mods.createInstance")}</h3>
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "flex items-center justify-center" })}
					onclick={handleClose}
					title={uiText("statusBanner.dismiss")}
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			{#if createSuccess}
				<div class="flex items-center gap-3 rounded-2xl p-4 text-sm bg-success/10 border border-success/30 text-success font-bold">
					<Check class="h-5 w-5" />
					{uiText("ui.c14dce9ce6cc09c2")}{newName}{uiText("ui.f9fbbf8636c544e7")}
				</div>
			{:else}
				<!-- Top: Icon & 3 Stacked Buttons -->
				<div class="flex items-center gap-4">
					<div class="w-20 h-20 rounded-2xl bg-bg-subtle border border-fg/10 flex items-center justify-center p-2 shrink-0 shadow-inner overflow-hidden">
						<img loading="lazy" decoding="async"
							src={getIconSrc(newIcon)}
							alt={uiText("ui.f15f8f19139853cf")}
							class="w-16 h-16 object-contain [image-rendering:pixelated]"
							onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
						/>
					</div>

					<div class="flex flex-col gap-1.5 flex-1">
						<button
							type="button"
							onclick={handleCustomIconUpload}
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
						>
							<Upload class="w-3.5 h-3.5" /> {uiText("ui.e1d12d1a29a77b00")}
						</button>
						<button
							type="button"
							onclick={handleRandomizeIcon}
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
						>
							<RefreshCw class="w-3.5 h-3.5" /> {uiText("ui.b095078f355b4721")}
						</button>
						<button
							type="button"
							onclick={() => showIconPicker = !showIconPicker}
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
						>
							<Palette class="w-3.5 h-3.5" /> {uiText("ui.af283098bccceacd")}
						</button>
					</div>
				</div>

				<!-- Icon Picker Popover / Drawer -->
				{#if showIconPicker}
					<div class="p-3 bg-bg-subtle rounded-2xl border border-fg/10 space-y-2" transition:slide={{ easing: quintOut, duration: 220 }}>
						<span class="text-[11px] font-bold text-fg/70 block">{uiText("ui.3b640f8f1bd94d4b")}</span>
						<div class="flex items-center gap-2 flex-wrap">
							{#each iconPresets as ip}
								<button
									type="button"
									onclick={() => { newIcon = ip.id; showIconPicker = false; }}
									class="w-9 h-9 rounded-xl p-1.5 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex items-center justify-center border {newIcon === ip.id ? 'bg-success/20 border-success' : 'bg-bg-elevated border-fg/10 hover:border-fg/30'}"
									title={ip.label}
								>
									<img loading="lazy" decoding="async" src={ip.src} alt={ip.label} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
								</button>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Nome -->
				<div class="space-y-1.5">
					<label for="instance-name" class="block text-xs font-bold text-fg/80">{uiText("instances.sortName")}</label>
					<input
						id="instance-name"
						type="text"
						bind:value={newName}
						class="w-full h-10 rounded-xl px-3.5 text-xs font-medium text-fg bg-bg-subtle border border-fg/10 focus:border-success outline-none transition-[color,background-color,border-color,box-shadow,transform,opacity]"
						placeholder={uiText("ui.9169b7ddb239cac2")}
					/>
				</div>

				<!-- Loader -->
				<div class="space-y-1.5">
					<span class="block text-xs font-bold text-fg/80">{uiText("instances.loader")}</span>
					<div class="flex flex-wrap gap-2">
						{#each loaderOptions as ldr}
							{@const isSelected = newLoader === ldr.id}
							<button
								type="button"
								onclick={() => selectLoader(ldr.id)}
								class="px-3.5 py-1.5 rounded-xl text-xs font-medium border transition-[color,background-color,border-color,box-shadow,transform,opacity] flex items-center gap-1.5 cursor-pointer {isSelected ? 'bg-success/20 text-success border-success/50 shadow-sm' : 'bg-bg-subtle border-fg/10 text-fg/70 hover:text-fg hover:border-fg/20'}"
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
					<span class="block text-xs font-bold text-fg/80">{uiText("ui.be772845450077f0")}</span>
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
						<span class="block text-xs font-bold text-fg/80">{uiText("instances.loaderVersion")}</span>
						<div class="flex gap-2">
							{#each [
								{ id: 'stable' as const, label: uiText("ui.a26ad010d2b24254") },
								{ id: 'latest' as const, label: uiText("ui.57dc95cb14ce4165") },
								{ id: 'other' as const, label: uiText("ui.f26d93169aee9bf9") }
							] as lv}
								{@const isSelected = loaderReleaseType === lv.id}
								<button
									type="button"
									onclick={() => loaderReleaseType = lv.id}
									class="px-3.5 py-1.5 rounded-xl text-xs font-medium border transition-[color,background-color,border-color,box-shadow,transform,opacity] flex items-center gap-1.5 cursor-pointer {isSelected ? 'bg-success/20 text-success border-success/50 shadow-sm' : 'bg-bg-subtle border-fg/10 text-fg/70 hover:text-fg hover:border-fg/20'}"
								>
									{#if isSelected}
										<Check class="w-3.5 h-3.5 stroke-[2.5]" />
									{/if}
									{lv.label}
								</button>
							{/each}
						</div>
						{#if loaderVersionsLoading}<p class="text-xs text-fg/50">{uiText("ui.39928af4a13cacba")}</p>{/if}
						{#if loaderVersionError}<p class="text-xs text-danger" role="alert">{loaderVersionError}</p>{/if}
						{#if loaderReleaseType === 'other' && availableLoaderVersions.length > 0}
							<select bind:value={selectedLoaderVersion} class="w-full rounded-xl border border-fg/10 bg-bg-subtle px-3 py-2 text-xs text-fg">
								{#each availableLoaderVersions as loaderVersion}<option value={loaderVersion.id}>{loaderVersion.id}{loaderVersion.stable ? uiText("ui.9b83b2b7eb93f7ff") : ''}</option>{/each}
							</select>
						{/if}
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
							<span>{uiText("ui.50889396f5bb48f1")}</span>
						</div>
						{#if showAdvanced}
							<ChevronUp class="w-3.5 h-3.5" />
						{:else}
							<ChevronDown class="w-3.5 h-3.5" />
						{/if}
					</button>

					{#if showAdvanced}
						<div class="space-y-3 pt-2" transition:slide={{ easing: quintOut, duration: 220 }}>
							<!-- Memória RAM -->
							<div class="space-y-1.5">
								<div class="flex justify-between text-xs">
									<span class="font-bold text-fg/80">{uiText("ui.0b6c9817a5ae09a9")}</span>
									<span class="font-mono text-success font-bold">{selectedRamGb} GB</span>
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
									<span>{uiText("ui.4a00b64876baefdd")}</span>
								</label>
								{#if newLoader !== "vanilla"}
									<label class="flex items-center gap-2 text-xs text-fg/80 cursor-pointer">
										<input type="checkbox" bind:checked={newInstallPerfPack} class="accent-emerald-500 rounded" />
										<span>{uiText("ui.76ce558dd6842f19")}</span>
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
						class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
					>
						<ArrowLeft class="w-3.5 h-3.5" /> {uiText("common.back")}
					</button>
					<button
						type="button"
						disabled={creating || versionsLoading || !newVersion || !newName.trim() || (newLoader !== 'vanilla' && (loaderVersionsLoading || availableLoaderVersions.length === 0))}
						onclick={handleCreate}
						class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-2 disabled:opacity-50" })}
					>
						{#if creating}
							<div class="w-4 h-4 rounded-full border-2 border-black border-t-transparent animate-spin"></div>
							{uiText("ui.7000cd83756f9135")}
						{:else}
							<Plus class="w-4 h-4 stroke-[3]" /> {uiText("mods.createInstance")}
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
						class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
						onclick={() => (lastError = null)}
					>
						<X class="h-3.5 h-3.5" />
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}
