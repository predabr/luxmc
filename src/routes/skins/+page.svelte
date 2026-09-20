<script lang="ts">
	import { onMount, untrack } from "svelte";
	import { deepLinks } from "$lib/stores/deepLinks.svelte";
	import { authChangeSkin, authSetAccountCape } from "$lib/api/auth";
	import { 
		RefreshCw, 
		Upload, 
		Search, 
		RotateCcw, 
		Play, 
		Pause, 
		Layers, 
		CheckCircle2, 
		Share2,
		Pencil,
		Plus,
		Check,
		Info,
		ChevronDown,
		ChevronUp,
		Move,
		Trash2,
		X,
		ArrowLeft,
		ChevronRight,
		Shirt
	} from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import SkinViewer3D from "$lib/components/ui/SkinViewer3D.svelte";
	import { activeSkinStore, type CapeType } from "$lib/stores/skin.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { getCapePreviewDataUrl, getFullCapeDataUrl } from "$lib/utils/capeTextures";
	import { goto } from "$app/navigation";
	import { fade, slide, fly } from "svelte/transition";

	let isUpdating = $state(false);
	let skinType = $state<"steve" | "alex">("steve");
	let isRotating = $state(true);
	let activeAnimation = $state<"idle" | "walk" | "run" | "fly" | "none">("idle");
	let viewerRef = $state<SkinViewer3D | null>(null);

	let searchNick = $state("");
	let isSearchingNick = $state(false);
	let selectedCape = $state<CapeType>("luxmc");
	let customCapeDataUrl = $state("");
	let showEditModal = $state(false);

	let savedSkinsExpanded = $state(true);
	let defaultSkinsExpanded = $state(true);

	type SavedSkinItem = { id: string; name: string; url: string; model: "steve" | "alex" };
	let savedSkins = $state<SavedSkinItem[]>([]);

	let selectedSkinNick = $state("Steve");

	const isMicrosoft = $derived(
		Boolean(
			account.value?.minecraftToken &&
			!account.value?.id.startsWith("offline_") &&
			!account.value?.id.startsWith("offline-")
		)
	);

	const username = $derived(account.value?.username || "Steve");
	
	let previewSkinUrl = $state("");

	const currentSkinUrl = $derived(
		previewSkinUrl ||
		activeSkinStore.current.skinUrl ||
		account.value?.skinUrl ||
		`https://minotar.net/skin/${username}`
	);

	const defaultSkins = [
		{ name: "Steve", nick: "Steve", model: "steve" as const },
		{ name: "Alex", nick: "Alex", model: "alex" as const },
		{ name: "Ari", nick: "Ari", model: "alex" as const },
		{ name: "Efe", nick: "Efe", model: "alex" as const },
		{ name: "Kai", nick: "Kai", model: "alex" as const },
		{ name: "Makena", nick: "Makena", model: "alex" as const },
		{ name: "Noor", nick: "Noor", model: "alex" as const },
		{ name: "Sunny", nick: "Sunny", model: "alex" as const },
		{ name: "Zuri", nick: "Zuri", model: "alex" as const },
		{ name: "Technoblade", nick: "Technoblade", model: "steve" as const },
		{ name: "Dream", nick: "Dream", model: "steve" as const },
		{ name: "Mumbo Jumbo", nick: "Mumbo", model: "steve" as const },
		{ name: "DanTDM", nick: "DanTDM", model: "steve" as const },
		{ name: "Grian", nick: "Grian", model: "steve" as const }
	];

	const capeList: { id: CapeType; name: string; desc: string }[] = [
		{ id: "none", name: "Nenhuma", desc: "Sem capa" },
		{ id: "luxmc", name: "Luxmc Oficial", desc: "Ouro & Obsidiana" },
		{ id: "optifine", name: "OptiFine OF", desc: "Clássica vermelha" },
		{ id: "migrator", name: "Migrator", desc: "Ouro Mojang" },
		{ id: "cherry", name: "Cherry Blossom", desc: "Flor de Cerejeira" },
		{ id: "vanilla", name: "Vanilla Cape", desc: "Edição Especial" },
		{ id: "minecon2011", name: "Minecon 2011", desc: "Creeper Vermelho" },
		{ id: "minecon2012", name: "Minecon 2012", desc: "Picareta Noturna" },
		{ id: "minecon2013", name: "Minecon 2013", desc: "Pistão Redstone" },
		{ id: "minecon2015", name: "Minecon 2015", desc: "Golem de Ferro" },
		{ id: "minecon2016", name: "Minecon 2016", desc: "Enderman Roxo" },
		{ id: "tiktok", name: "TikTok", desc: "Comemorativa" },
		{ id: "twitch", name: "Twitch", desc: "Roxa Glitch" },
		{ id: "custom", name: "Customizada", desc: "Arquivo .PNG local" },
	];

	onMount(() => {
		if (activeSkinStore.current.type) {
			skinType = activeSkinStore.current.type;
		}
		if (activeSkinStore.current.capeType) {
			selectedCape = activeSkinStore.current.capeType;
		}
		if (activeSkinStore.current.customCapeUrl) {
			customCapeDataUrl = activeSkinStore.current.customCapeUrl;
		}
		if (activeSkinStore.current.skinUrl) {
			previewSkinUrl = activeSkinStore.current.skinUrl;
		}
		try {
			const saved = localStorage.getItem("luxmc_saved_skins");
			if (saved) savedSkins = JSON.parse(saved);
		} catch {}
	});

    $effect(() => {
        const skin = deepLinks.skin;
        if (!skin) return;
        untrack(() => {
            previewSkinUrl = skin.url;
            skinType = skin.model === "slim" ? "alex" : "steve";
            deepLinks.skin = null;
            toast("Skin recebida do portal web!", "info");
        });
    });

	function selectDefaultSkin(skin: typeof defaultSkins[0]) {
		selectedSkinNick = skin.nick;
		skinType = skin.model;
		previewSkinUrl = `https://mineskin.eu/skin/${encodeURIComponent(skin.nick)}`;
	}

	function selectSavedSkin(skin: SavedSkinItem) {
		selectedSkinNick = skin.name;
		skinType = skin.model;
		previewSkinUrl = skin.url;
	}

	function removeSavedSkin(id: string) {
		savedSkins = savedSkins.filter(s => s.id !== id);
		try {
			localStorage.setItem("luxmc_saved_skins", JSON.stringify(savedSkins));
		} catch {}
	}

	async function handleAddSkinFile() {
		try {
			const selected = await open({
				multiple: false,
				filters: [{ name: "Minecraft Skin PNG", extensions: ["png"] }]
			});
			if (!selected || typeof selected !== "string") return;

			const dataUrl = convertFileSrc(selected);
			previewSkinUrl = dataUrl;
			selectedSkinNick = "Skin Customizada";

			const newSkin: SavedSkinItem = {
				id: String(Date.now()),
				name: `Skin ${savedSkins.length + 1}`,
				url: dataUrl,
				model: skinType
			};
			savedSkins = [newSkin, ...savedSkins];
			try {
				localStorage.setItem("luxmc_saved_skins", JSON.stringify(savedSkins));
			} catch {}
			toast("Skin adicionada às suas skins salvas!", "success");
		} catch (e) {
			toast("Erro ao carregar arquivo de skin: " + String(e), "error");
		}
	}

	function handleSearchNick() {
		const nick = searchNick.trim();
		if (!nick) {
			toast("Digite um nickname para buscar.", "info");
			return;
		}
		isSearchingNick = true;
		const targetUrl = `https://mineskin.eu/skin/${encodeURIComponent(nick)}`;
		
		const img = new Image();
		img.crossOrigin = "anonymous";
		img.onload = () => {
			previewSkinUrl = targetUrl;
			selectedSkinNick = nick;
			isSearchingNick = false;
			toast(`Skin de "${nick}" carregada com sucesso!`, "success");
		};
		img.onerror = () => {
			previewSkinUrl = `https://minotar.net/skin/${encodeURIComponent(nick)}`;
			selectedSkinNick = nick;
			isSearchingNick = false;
			toast(`Skin de "${nick}" carregada via Minotar!`, "success");
		};
		img.src = targetUrl;
	}

	async function handleCustomCapeUpload() {
		try {
			const selected = await open({
				multiple: false,
				filters: [{ name: "Minecraft Cape PNG", extensions: ["png"] }]
			});
			if (!selected || typeof selected !== "string") return;

			const dataUrl = convertFileSrc(selected);
			customCapeDataUrl = dataUrl;
			selectedCape = "custom";
			toast("Capa personalizada carregada!", "success");
		} catch (e) {
			toast("Erro ao carregar capa personalizada: " + String(e), "error");
		}
	}

	async function handleApplyToAccount(): Promise<void> {
        if (isUpdating) return;
        if (!account.value) { toast("Selecione uma conta primeiro.", "error"); return; }
		isUpdating = true;
		try {
			let finalSkin = currentSkinUrl;
            if (!finalSkin.startsWith("https://") && !finalSkin.startsWith("http://") && !finalSkin.startsWith("data:")) {
                const response = await fetch(finalSkin);
                if (!response.ok) throw new Error("Não foi possível ler a textura local.");
                const blob = await response.blob();
                finalSkin = await new Promise<string>((resolve, reject) => {
                    const reader = new FileReader();
                    reader.onload = () => typeof reader.result === "string" ? resolve(reader.result) : reject(new Error("PNG inválido"));
                    reader.onerror = () => reject(new Error("Falha ao ler o PNG"));
                    reader.readAsDataURL(blob);
                });
            }
            await authChangeSkin(account.value.id, skinType === "alex" ? "slim" : "classic", finalSkin);
			const hasCape = selectedCape !== "none";
			const capeUrl = selectedCape === "custom" 
				? customCapeDataUrl 
				: (selectedCape !== "none" ? getFullCapeDataUrl(selectedCape) : "");

			activeSkinStore.setSkin({
				id: account.value?.uuid || "offline",
				name: username,
				url: finalSkin,
				skinUrl: finalSkin,
				avatarUrl: `https://mc-heads.net/avatar/${encodeURIComponent(selectedSkinNick || username)}/100`,
				type: skinType,
				hasCape,
				capeType: selectedCape,
				customCapeUrl: customCapeDataUrl
			});

			if (account.value) {
				account.value = {
					...account.value,
					skinUrl: finalSkin,
					skinVariant: skinType === "alex" ? "slim" : "classic",
					capeUrl: hasCape ? capeUrl : null
				};
				if (typeof window !== "undefined") {
					try {
						localStorage.setItem("luxmc_current_account", JSON.stringify(account.value));
					} catch {}
				}
				await authSetAccountCape(account.value.id, hasCape ? capeUrl : null).catch(() => {});
			}

			toast("Skin e capa aplicadas com sucesso no Minecraft!", "success");
		} catch (e) {
			toast("Erro ao salvar skin: " + String(e), "error");
		} finally {
			setTimeout(() => { isUpdating = false; }, 300);
		}
	}

	async function handleSyncWithWeb(): Promise<void> {
		const nick = selectedSkinNick || username;
		const webUrl = `https://luxmc-r92.pages.dev/skins.html?nick=${encodeURIComponent(nick)}&model=${skinType}&cape=${selectedCape}`;
		try { 
			await openUrl(webUrl); 
			toast("Abrindo Estúdio de Skins no navegador...", "info"); 
		} catch (error) { 
			toast(String(error), "error"); 
		}
	}
</script>

<div class="h-full flex flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-24">
	
	<header class="flex items-center justify-between gap-4 py-1">
		<div class="flex items-center gap-3">
			<div class="flex items-center gap-1 bg-[#14171d] border border-white/[0.08] rounded-xl p-1 shadow-sm">
				<button 
					type="button" 
					class="p-1.5 rounded-lg text-white/40 hover:text-white hover:bg-white/[0.06] transition-colors cursor-pointer"
					title="Voltar"
					onclick={() => history.back()}
				>
					<ArrowLeft class="w-3.5 h-3.5" />
				</button>
				<button 
					type="button" 
					class="p-1.5 rounded-lg text-white/40 hover:text-white hover:bg-white/[0.06] transition-colors cursor-pointer"
					title="Avançar"
					onclick={() => history.forward()}
				>
					<ChevronRight class="w-3.5 h-3.5" />
				</button>
			</div>

			<div class="flex items-center gap-2 text-xs font-bold text-white/80">
				<Shirt class="w-3.5 h-3.5 text-white/60" />
				<span class="text-white font-extrabold">Skin selector</span>
			</div>
		</div>

		<div class="flex items-center gap-2.5">
			<button
				type="button"
				onclick={handleSyncWithWeb}
				class="flex items-center gap-2 px-3.5 py-1.5 rounded-xl bg-[#14171d] hover:bg-[#1a1e26] text-white/80 hover:text-white text-xs font-bold border border-white/[0.08] transition-all cursor-pointer shadow-sm"
				title="Abrir no NameMC / Web Studio"
			>
				<Share2 class="w-3.5 h-3.5 text-[#1bd96a]" />
				<span>Web Studio</span>
			</button>
		</div>
	</header>

	<div class="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
		
		<div class="lg:col-span-4 flex flex-col items-center gap-4 lg:sticky lg:top-2">
			
			<div class="w-full h-[520px] rounded-3xl bg-[#14171d] border border-white/[0.06] relative overflow-hidden shadow-2xl flex flex-col items-center justify-center p-4">
				
				<div class="absolute top-3 left-3 right-3 flex items-center justify-between z-10 pointer-events-none">
					<span class="text-[11px] font-black uppercase tracking-wider text-white/50 bg-white/[0.04] border border-white/10 px-2.5 py-1 rounded-full">
						{skinType === "alex" ? "Slim (3px)" : "Classic (4px)"}
					</span>

					<button
						type="button"
						onclick={() => isRotating = !isRotating}
						class="p-2 rounded-xl bg-black/40 hover:bg-black/60 border border-white/10 text-white/70 hover:text-white pointer-events-auto transition-all cursor-pointer shadow-sm"
						title={isRotating ? "Pausar rotação" : "Ativar rotação"}
					>
						{#if isRotating}
							<Pause class="w-3.5 h-3.5 text-[#1bd96a]" />
						{:else}
							<Play class="w-3.5 h-3.5" />
						{/if}
					</button>
				</div>

				<SkinViewer3D
					bind:this={viewerRef}
					skinUrl={currentSkinUrl}
					slim={skinType === "alex"}
					cape={selectedCape}
					customCapeUrl={selectedCape === "custom" ? customCapeDataUrl : ""}
					autoRotate={isRotating}
					animation={activeAnimation}
					className="w-full h-full"
				/>

			</div>

			<div class="flex flex-col items-center gap-2.5 w-full">
				<div class="flex items-center gap-1.5 text-xs text-white/40 font-medium">
					<Move class="w-3.5 h-3.5" />
					<span>Drag to rotate</span>
				</div>

				<button
					type="button"
					onclick={() => showEditModal = true}
					class="w-full max-w-[200px] py-2 px-4 rounded-xl bg-[#1a1d24] hover:bg-[#222731] text-white border border-white/[0.08] hover:border-white/[0.15] text-xs font-bold transition-all flex items-center justify-center gap-2 cursor-pointer shadow-sm active:scale-[0.98]"
				>
					<Pencil class="w-3.5 h-3.5 text-white/70" />
					<span>Edit skin</span>
				</button>
			</div>

		</div>

		<div class="lg:col-span-8 flex flex-col gap-6">
			
			<section class="space-y-3">
				<button 
					type="button" 
					onclick={() => savedSkinsExpanded = !savedSkinsExpanded}
					class="flex items-center gap-2 text-sm font-black text-white hover:text-[#1bd96a] transition-colors cursor-pointer"
				>
					{#if savedSkinsExpanded}
						<ChevronUp class="w-4 h-4 text-white/50" />
					{:else}
						<ChevronDown class="w-4 h-4 text-white/50" />
					{/if}
					<span>Saved skins</span>
				</button>

				{#if savedSkinsExpanded}
					<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 gap-3.5" transition:slide={{ duration: 150 }}>
						
						<button
							type="button"
							onclick={handleAddSkinFile}
							class="rounded-2xl border-2 border-dashed border-white/[0.12] hover:border-[#1bd96a] bg-white/[0.01] hover:bg-white/[0.03] p-5 flex flex-col items-center justify-center text-center gap-2 transition-all cursor-pointer min-h-[170px] group shadow-sm"
						>
							<div class="w-10 h-10 rounded-full bg-white/[0.04] border border-white/10 group-hover:border-[#1bd96a] flex items-center justify-center text-white/60 group-hover:text-[#1bd96a] transition-colors">
								<Plus class="w-5 h-5 stroke-[2.5]" />
							</div>
							<div>
								<span class="text-xs font-bold text-white block">Add skin</span>
								<span class="text-[11px] text-white/40 block mt-0.5">Drag and drop</span>
							</div>
						</button>

						{#each savedSkins as s (s.id)}
							{@const isSelected = previewSkinUrl === s.url}
							<div
								role="button"
								tabindex="0"
								onclick={() => selectSavedSkin(s)}
								onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") selectSavedSkin(s); }}
								class="rounded-2xl bg-[#14171d] hover:bg-[#181c24] border transition-all p-3 flex flex-col items-center justify-between relative cursor-pointer group min-h-[170px] shadow-sm {isSelected ? 'border-[#1bd96a] ring-1 ring-[#1bd96a]' : 'border-white/[0.06] hover:border-white/[0.15]'}"
							>
								{#if isSelected}
									<div class="absolute top-2.5 right-2.5 w-5 h-5 rounded-full bg-black border border-white flex items-center justify-center text-white shadow-md z-10">
										<Check class="w-3 h-3 stroke-[3]" />
									</div>
								{/if}

								<div class="w-full flex-1 flex items-center justify-center my-1 overflow-hidden">
									<img 
										src={s.url} 
										alt={s.name}
										class="h-28 object-contain [image-rendering:pixelated] drop-shadow-md transition-transform group-hover:scale-105" 
									/>
								</div>

								<div class="w-full flex items-center justify-between pt-2 border-t border-white/[0.04]">
									<span class="text-xs font-bold text-white truncate max-w-[90px]">{s.name}</span>
									<button
										type="button"
										onclick={(e) => {
											e.stopPropagation();
											removeSavedSkin(s.id);
										}}
										class="text-white/30 hover:text-red-400 transition-colors p-1 cursor-pointer"
										title="Remover skin salva"
									>
										<Trash2 class="w-3 h-3" />
									</button>
								</div>
							</div>
						{/each}

					</div>
				{/if}
			</section>

			<section class="space-y-3">
				<button 
					type="button" 
					onclick={() => defaultSkinsExpanded = !defaultSkinsExpanded}
					class="flex items-center gap-2 text-sm font-black text-white hover:text-[#1bd96a] transition-colors cursor-pointer"
				>
					{#if defaultSkinsExpanded}
						<ChevronUp class="w-4 h-4 text-white/50" />
					{:else}
						<ChevronDown class="w-4 h-4 text-white/50" />
					{/if}
					<span>Default skins</span>
				</button>

				{#if defaultSkinsExpanded}
					<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3.5" transition:slide={{ duration: 150 }}>
						{#each defaultSkins as skin (skin.nick)}
							{@const isSelected = selectedSkinNick === skin.nick}
							<div
								role="button"
								tabindex="0"
								onclick={() => selectDefaultSkin(skin)}
								onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") selectDefaultSkin(skin); }}
								class="rounded-2xl bg-[#14171d] hover:bg-[#181c24] border transition-all p-3 flex flex-col items-center justify-between relative cursor-pointer group min-h-[175px] shadow-sm {isSelected ? 'border-[#1bd96a] ring-1 ring-[#1bd96a]' : 'border-white/[0.06] hover:border-white/[0.15]'}"
							>
								{#if isSelected}
									<div class="absolute top-2.5 right-2.5 w-5 h-5 rounded-full bg-black border border-white flex items-center justify-center text-white shadow-md z-10">
										<Check class="w-3 h-3 stroke-[3]" />
									</div>
								{/if}

								<div class="w-full flex-1 flex items-center justify-center my-1 overflow-hidden">
									<img 
										src={`https://mc-heads.net/body/${skin.nick}/140`} 
										alt={skin.name}
										class="h-28 object-contain [image-rendering:pixelated] drop-shadow-md transition-transform group-hover:scale-105" 
										onerror={(e) => {
											(e.currentTarget as HTMLImageElement).src = `https://minotar.net/armor/body/${skin.nick}/120.png`;
										}} 
									/>
								</div>

								<div class="w-full text-center pt-2 border-t border-white/[0.04]">
									<span class="text-xs font-bold text-white block truncate group-hover:text-[#1bd96a] transition-colors">{skin.name}</span>
									<span class="text-[10px] text-white/40 block mt-0.5">{skin.model === "alex" ? "Slim (3px)" : "Classic (4px)"}</span>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</section>

		</div>

	</div>

	<div class="fixed bottom-6 left-1/2 -translate-x-1/2 w-[90%] max-w-2xl bg-[#14171d]/95 backdrop-blur-xl border border-white/[0.08] rounded-2xl p-3.5 px-5 flex items-center justify-between gap-4 shadow-2xl z-30">
		<div class="flex items-center gap-3 min-w-0">
			<div class="w-8 h-8 rounded-xl bg-white/[0.06] flex items-center justify-center text-white/60 shrink-0">
				<Info class="w-4 h-4" />
			</div>
			<div class="min-w-0">
				{#if isMicrosoft}
					<div class="text-xs font-extrabold text-white truncate">Conta Microsoft Conectada</div>
					<div class="text-[11px] text-white/50 truncate">As skins selecionadas serão aplicadas diretamente ao seu jogo.</div>
				{:else}
					<div class="text-xs font-extrabold text-white truncate">Editando com conta offline / demo</div>
					<div class="text-[11px] text-white/50 truncate">Entre na sua conta Microsoft para sincronizar skins com o servidor!</div>
				{/if}
			</div>
		</div>

		<div class="flex items-center gap-2.5 shrink-0">
			{#if !isMicrosoft}
				<button
					type="button"
					onclick={() => goto("/")}
					class="px-4 py-2 rounded-xl bg-[#1bd96a] hover:bg-[#18c45f] text-[#090a0f] text-xs font-black flex items-center gap-1.5 shadow-sm active:scale-[0.98] cursor-pointer"
				>
					<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
						<path d="M1 1h10v10H1V1zm12 0h10v10H13V1zM1 13h10v10H1V13zm12 0h10v10H13V13z"/>
					</svg>
					<span>Entrar com Microsoft</span>
				</button>
			{/if}

			<button
				type="button"
				onclick={handleApplyToAccount}
				disabled={isUpdating}
				class="px-4 py-2 rounded-xl {isMicrosoft ? 'bg-[#1bd96a] hover:bg-[#18c45f] text-[#090a0f]' : 'bg-white/10 hover:bg-white/20 text-white'} text-xs font-black flex items-center gap-1.5 shadow-sm active:scale-[0.98] cursor-pointer disabled:opacity-50"
			>
				{#if isUpdating}
					<RefreshCw class="w-3.5 h-3.5 animate-spin" />
					<span>Salvando...</span>
				{:else}
					<Check class="w-3.5 h-3.5 stroke-[3]" />
					<span>Aplicar Skin</span>
				{/if}
			</button>
		</div>
	</div>

	{#if showEditModal}
		<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md" in:fade={{ duration: 150 }}>
			<div class="w-full max-w-xl rounded-3xl bg-[#14171d] border border-white/10 p-6 shadow-2xl space-y-5" in:fly={{ y: 20, duration: 200 }}>
				<div class="flex items-center justify-between border-b border-white/[0.06] pb-3">
					<div class="flex items-center gap-2.5">
						<Pencil class="w-4 h-4 text-[#1bd96a]" />
						<h3 class="text-sm font-extrabold text-white">Editar Configurações da Skin</h3>
					</div>
					<button 
						type="button" 
						class="text-white/40 hover:text-white text-xs cursor-pointer p-1"
						onclick={() => showEditModal = false}
					>
						<X class="w-4 h-4" />
					</button>
				</div>

				<div class="space-y-2">
					<span class="text-xs font-bold text-white/70 block">Modelo dos Braços</span>
					<div class="grid grid-cols-2 gap-2">
						<button
							type="button"
							onclick={() => skinType = "steve"}
							class="p-3 rounded-2xl border text-left transition-all cursor-pointer {skinType === 'steve' ? 'bg-[#1bd96a]/15 border-[#1bd96a] text-[#1bd96a]' : 'bg-white/[0.02] border-white/10 text-white/60 hover:text-white'}"
						>
							<div class="text-xs font-extrabold">Classic (Steve)</div>
							<div class="text-[10px] opacity-70">Braços normais com 4 pixels</div>
						</button>
						<button
							type="button"
							onclick={() => skinType = "alex"}
							class="p-3 rounded-2xl border text-left transition-all cursor-pointer {skinType === 'alex' ? 'bg-[#1bd96a]/15 border-[#1bd96a] text-[#1bd96a]' : 'bg-white/[0.02] border-white/10 text-white/60 hover:text-white'}"
						>
							<div class="text-xs font-extrabold">Slim (Alex)</div>
							<div class="text-[10px] opacity-70">Braços finos com 3 pixels</div>
						</button>
					</div>
				</div>

				<div class="space-y-2">
					<label for="search-nickname-input" class="text-xs font-bold text-white/70 block">Buscar por Nickname (Mojang / NameMC)</label>
					<div class="flex items-center gap-2">
						<input 
							id="search-nickname-input"
							type="text" 
							placeholder="Ex: Technoblade, Dream, MumboJumbo..."
							bind:value={searchNick}
							onkeydown={(e) => e.key === 'Enter' && handleSearchNick()}
							class="flex-1 bg-black/40 border border-white/10 rounded-xl px-3.5 py-2 text-xs text-white placeholder-white/30 outline-none focus:border-[#1bd96a]"
						/>
						<button
							type="button"
							onclick={handleSearchNick}
							disabled={isSearchingNick}
							class="px-4 py-2 rounded-xl bg-[#1bd96a] hover:bg-[#18c45f] text-[#090a0f] font-black text-xs cursor-pointer disabled:opacity-50"
						>
							{#if isSearchingNick}
								<RefreshCw class="w-3.5 h-3.5 animate-spin" />
							{:else}
								<span>Buscar</span>
							{/if}
						</button>
					</div>
				</div>

				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<span class="text-xs font-bold text-white/70 block">Escolha de Capa HD</span>
						{#if selectedCape === "custom"}
							<button
								type="button"
								onclick={handleCustomCapeUpload}
								class="text-[11px] font-bold text-[#1bd96a] hover:underline flex items-center gap-1 cursor-pointer"
							>
								<Upload class="w-3 h-3" /> Trocar .PNG da Capa
							</button>
						{/if}
					</div>

					<div class="grid grid-cols-3 sm:grid-cols-4 gap-2 max-h-[160px] overflow-y-auto custom-scrollbar pr-1">
						{#each capeList as c}
							{@const isCapeSelected = selectedCape === c.id}
							{@const previewUrl = c.id !== "none" && c.id !== "custom" ? getCapePreviewDataUrl(c.id) : null}
							<button
								type="button"
								onclick={() => {
									if (c.id === "custom" && !customCapeDataUrl) {
										handleCustomCapeUpload();
									} else {
										selectedCape = c.id;
									}
								}}
								class="p-2 rounded-xl border transition-all cursor-pointer flex flex-col items-center text-center gap-1 relative overflow-hidden {isCapeSelected ? 'bg-[#1bd96a]/15 border-[#1bd96a]' : 'bg-white/[0.02] border-white/10 hover:border-white/20'}"
							>
								<div class="w-8 h-12 rounded-lg bg-black/40 border border-white/10 flex items-center justify-center overflow-hidden shrink-0 shadow-inner">
									{#if previewUrl}
										<img 
											src={previewUrl} 
											alt={c.name} 
											class="w-full h-full object-contain [image-rendering:pixelated]" 
										/>
									{:else if c.id === "custom" && customCapeDataUrl}
										<img 
											src={customCapeDataUrl} 
											alt="Capa Custom" 
											class="w-full h-full object-cover [image-rendering:pixelated]" 
										/>
									{:else if c.id === "custom"}
										<Upload class="w-4 h-4 text-white/50" />
									{:else}
										<span class="text-[9px] text-white/30 font-bold uppercase">OFF</span>
									{/if}
								</div>
								<span class="text-[10px] font-bold truncate block text-white w-full">{c.name}</span>
							</button>
						{/each}
					</div>
				</div>

				<div class="flex items-center justify-end gap-2 pt-2 border-t border-white/[0.06]">
					<button 
						type="button" 
						onclick={() => showEditModal = false}
						class="px-5 py-2 rounded-xl bg-[#1bd96a] hover:bg-[#18c45f] text-[#090a0f] text-xs font-black cursor-pointer"
					>
						Pronto
					</button>
				</div>
			</div>
		</div>
	{/if}

</div>
