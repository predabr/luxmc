<script lang="ts">
	import { onMount } from "svelte";
	import { 
		RefreshCw, 
		Upload, 
		Search,
		RotateCcw,
		Play,
		Pause,
		Layers,
		CheckCircle2,
		Share2
	} from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import SkinViewer3D from "$lib/components/ui/SkinViewer3D.svelte";
	import { activeSkinStore, type CapeType } from "$lib/stores/skin.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { getCapePreviewDataUrl, getFullCapeDataUrl } from "$lib/utils/capeTextures";

	let isUpdating = $state(false);
	let skinType = $state<"steve" | "alex">("steve");
	let isRotating = $state(true);
	let activeAnimation = $state<"idle" | "walk" | "run" | "fly" | "none">("walk");
	let viewerRef = $state<SkinViewer3D | null>(null);

	let searchNick = $state("");
	let isSearchingNick = $state(false);
	let selectedCape = $state<CapeType>("luxmc");
	let customCapeDataUrl = $state("");

	const isMicrosoft = $derived(
		Boolean(
			account.value?.minecraftToken &&
			!account.value?.id.startsWith("offline_") &&
			!account.value?.id.startsWith("offline-")
		)
	);

	const username = $derived(account.value?.username || "Jogador");
	
	let previewSkinUrl = $state("");

	const currentSkinUrl = $derived(
		previewSkinUrl ||
		activeSkinStore.current.skinUrl ||
		account.value?.skinUrl ||
		`https://minotar.net/skin/${username}`
	);

	const popularSkins = [
		{ name: "Steve", nick: "Steve", model: "steve" as const },
		{ name: "Alex", nick: "Alex", model: "alex" as const },
		{ name: "Herobrine", nick: "Herobrine", model: "steve" as const },
		{ name: "Technoblade", nick: "Technoblade", model: "steve" as const },
		{ name: "Dream", nick: "Dream", model: "steve" as const },
		{ name: "Knight", nick: "Knight", model: "steve" as const },
		{ name: "Cyberpunk", nick: "Cyberpunk", model: "alex" as const },
	];

	const capeList: { id: CapeType; name: string; desc: string }[] = [
		{ id: "none", name: "Nenhuma", desc: "Sem capa nas costas" },
		{ id: "luxmc", name: "Luxmc Oficial", desc: "Dourada & Azul Obsidiana" },
		{ id: "optifine", name: "OptiFine OF", desc: "Clássica vermelha OptiFine" },
		{ id: "migrator", name: "Migrator", desc: "Ouro & Borgonha Mojang" },
		{ id: "cherry", name: "Cherry Blossom", desc: "Flor de Cerejeira 1.20" },
		{ id: "vanilla", name: "Vanilla Cape", desc: "Edição Especial Bedrock/Java" },
		{ id: "minecon2011", name: "Minecon 2011", desc: "Capa clássica do Creeper" },
		{ id: "minecon2012", name: "Minecon 2012", desc: "Picareta dourada noturna" },
		{ id: "minecon2013", name: "Minecon 2013", desc: "Pistão vermelho de Redstone" },
		{ id: "minecon2015", name: "Minecon 2015", desc: "Golem de Ferro de Londres" },
		{ id: "minecon2016", name: "Minecon 2016", desc: "Enderman roxo escura" },
		{ id: "tiktok", name: "TikTok", desc: "Capa comemorativa TikTok" },
		{ id: "twitch", name: "Twitch", desc: "Capa roxa comemorativa Twitch" },
		{ id: "custom", name: "Capa Customizada", desc: "Arquivo .PNG local" },
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
	});

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
			isSearchingNick = false;
			toast(`Skin de "${nick}" carregada com sucesso!`, "success");
		};
		img.onerror = () => {
			previewSkinUrl = `https://minotar.net/skin/${encodeURIComponent(nick)}`;
			isSearchingNick = false;
			toast(`Skin de "${nick}" carregada via Minotar!`, "success");
		};
		img.src = targetUrl;
	}

	function handleQuickSelectSkin(skin: { name: string; nick: string; model: "steve" | "alex" }) {
		searchNick = skin.nick;
		skinType = skin.model;
		previewSkinUrl = `https://mineskin.eu/skin/${encodeURIComponent(skin.nick)}`;
		toast(`Skin "${skin.name}" selecionada!`, "info");
	}

	async function handleLocalUpload() {
		try {
			const selected = await open({
				multiple: false,
				filters: [{ name: "Minecraft Skin PNG", extensions: ["png"] }]
			});
			if (!selected || typeof selected !== "string") return;

			const dataUrl = convertFileSrc(selected);
			previewSkinUrl = dataUrl;
			toast("Arquivo de skin carregado no visualizador!", "success");
		} catch (e) {
			toast("Erro ao carregar arquivo de skin: " + String(e), "error");
		}
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
			toast("Capa personalizada carregada com sucesso!", "success");
		} catch (e) {
			toast("Erro ao carregar capa personalizada: " + String(e), "error");
		}
	}

	function handleApplyToAccount() {
		isUpdating = true;
		try {
			const finalSkin = currentSkinUrl;
			const hasCape = selectedCape !== "none";
			const capeUrl = selectedCape === "custom" 
				? customCapeDataUrl 
				: (selectedCape !== "none" ? getFullCapeDataUrl(selectedCape) : "");

			activeSkinStore.setSkin({
				id: account.value?.uuid || "offline",
				name: username,
				url: finalSkin,
				skinUrl: finalSkin,
				avatarUrl: `https://mc-heads.net/avatar/${encodeURIComponent(searchNick || username)}/100`,
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
			}

			toast("Skin e capa aplicadas com sucesso! Seu Minecraft já usará este visual.", "success");
		} catch (e) {
			toast("Erro ao salvar skin na conta: " + String(e), "error");
		} finally {
			setTimeout(() => { isUpdating = false; }, 300);
		}
	}

	function handleSyncWithWeb() {
		const nick = searchNick.trim() || username;
		const webUrl = `https://luxmc-r92.pages.dev/skins.html?nick=${encodeURIComponent(nick)}&model=${skinType}&cape=${selectedCape}`;
		openUrl(webUrl);
		toast("Abrindo Estúdio de Skins sincronizado no navegador...", "info");
	}
</script>

<div class="h-full flex flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-12">
	
	<!-- Header -->
	<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
		<div>
			<div class="flex items-center gap-2.5">
				<h1 class="text-2xl font-black text-white tracking-tight">Personalização 3D de Skins & Capas</h1>
				<span class="text-[10px] font-black uppercase tracking-wider text-blue-400 bg-blue-500/10 border border-blue-500/20 px-2.5 py-0.5 rounded-full">
					Paridade Web
				</span>
			</div>
			<p class="text-xs text-white/50 mt-1">
				Troque skins e capas facilmente para contas Microsoft ou Offline. Suporta visualização 3D em tempo real.
			</p>
		</div>

		<div class="flex items-center gap-2.5">
			<button
				type="button"
				onclick={handleSyncWithWeb}
				class="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 active:scale-95 text-white/80 hover:text-white text-xs font-bold border border-white/10 transition-all cursor-pointer shadow-sm"
				title="Sincronizar com o Estúdio 3D do site oficial"
			>
				<Share2 class="w-3.5 h-3.5 text-blue-400" />
				<span>Studio Web</span>
			</button>

			<button
				type="button"
				onclick={handleApplyToAccount}
				disabled={isUpdating}
				class="flex items-center gap-2 px-5 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 active:scale-95 text-white text-xs font-black transition-all cursor-pointer shadow-lg shadow-blue-600/25 disabled:opacity-50"
			>
				{#if isUpdating}
					<RefreshCw class="w-3.5 h-3.5 animate-spin" />
					<span>Salvando...</span>
				{:else}
					<CheckCircle2 class="w-3.5 h-3.5" />
					<span>Salvar na Conta</span>
				{/if}
			</button>
		</div>
	</div>

	<!-- Main 2-Column Grid -->
	<div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
		
		<!-- Left: 3D Viewport & Visual Controls -->
		<div class="lg:col-span-5 flex flex-col gap-3">
			
			<div class="w-full h-[480px] rounded-3xl bg-[#111216] border border-white/10 relative overflow-hidden shadow-2xl flex flex-col items-center justify-center p-2">
				
				<!-- Top Viewport Floating Controls -->
				<div class="absolute top-3 left-3 right-3 flex items-center justify-between z-10 pointer-events-none">
					<div class="bg-black/60 backdrop-blur-md border border-white/10 rounded-xl px-2.5 py-1 text-[10px] font-bold text-white/60 pointer-events-auto flex items-center gap-1.5">
						<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
						Arraste para girar
					</div>

					<div class="flex items-center gap-1.5 pointer-events-auto">
						<button
							type="button"
							onclick={() => isRotating = !isRotating}
							class="p-2 rounded-xl bg-black/60 hover:bg-black/80 backdrop-blur-md border border-white/10 text-white/70 hover:text-white transition-all cursor-pointer"
							title={isRotating ? "Pausar rotação automática" : "Ativar rotação automática"}
						>
							{#if isRotating}
								<Pause class="w-3.5 h-3.5 text-blue-400" />
							{:else}
								<Play class="w-3.5 h-3.5" />
							{/if}
						</button>

						<button
							type="button"
							onclick={() => viewerRef?.resetCamera()}
							class="p-2 rounded-xl bg-black/60 hover:bg-black/80 backdrop-blur-md border border-white/10 text-white/70 hover:text-white transition-all cursor-pointer"
							title="Redefinir câmera"
						>
							<RotateCcw class="w-3.5 h-3.5" />
						</button>
					</div>
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

				<!-- Bottom Viewport Controls (Steve/Alex & Upload) -->
				<div class="absolute bottom-3 left-3 right-3 flex items-center justify-between bg-black/70 backdrop-blur-md border border-white/10 rounded-2xl p-1.5 shadow-xl z-10">
					<div class="flex items-center gap-1">
						<button
							type="button"
							onclick={() => skinType = "steve"}
							class="px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {skinType === 'steve' ? 'bg-blue-600 text-white shadow-sm' : 'text-white/60 hover:text-white hover:bg-white/5'}"
						>
							Steve (4px)
						</button>
						<button
							type="button"
							onclick={() => skinType = "alex"}
							class="px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {skinType === 'alex' ? 'bg-blue-600 text-white shadow-sm' : 'text-white/60 hover:text-white hover:bg-white/5'}"
						>
							Alex (3px)
						</button>
					</div>

					<button
						type="button"
						onclick={handleLocalUpload}
						title="Carregar arquivo PNG local de skin"
						class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/10 hover:bg-white/20 text-white text-xs font-bold transition-all cursor-pointer active:scale-95"
					>
						<Upload class="w-3.5 h-3.5 text-blue-400" />
						<span>Carregar .PNG</span>
					</button>
				</div>
			</div>

			<!-- Animation Selector Toolbar -->
			<div class="bg-[#141518] border border-white/5 rounded-2xl p-2.5 flex items-center justify-between gap-1 shadow-sm">
				<span class="text-[10px] font-black uppercase tracking-wider text-white/40 px-2">Animação:</span>
				<div class="flex items-center gap-1 flex-1 justify-end">
					{#each [
						{ id: "idle", label: "Parado" },
						{ id: "walk", label: "Andar" },
						{ id: "run", label: "Correr" },
						{ id: "fly", label: "Voar" },
						{ id: "none", label: "Estátua" }
					] as anim}
						<button
							type="button"
							onclick={() => activeAnimation = anim.id as any}
							class="px-2.5 py-1 rounded-xl text-xs font-bold transition-all cursor-pointer {activeAnimation === anim.id ? 'bg-white text-black font-black shadow-sm' : 'text-white/60 hover:text-white hover:bg-white/5'}"
						>
							{anim.label}
						</button>
					{/each}
				</div>
			</div>
		</div>

		<!-- Right: Skin Search, Capes, and Account Settings -->
		<div class="lg:col-span-7 flex flex-col gap-4">
			
			<!-- Nickname Search Box -->
			<div class="p-5 rounded-3xl bg-[#141518] border border-white/5 shadow-xl space-y-3.5">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Search class="w-4 h-4 text-blue-400" />
						<h3 class="text-xs font-extrabold text-white uppercase tracking-wider">
							Buscar Skin por Nickname
						</h3>
					</div>
					<span class="text-[10px] text-white/40 font-mono">Mojang / NameMC / Mineskin</span>
				</div>

				<div class="flex items-center gap-2">
					<div class="relative flex-1">
						<input 
							type="text" 
							placeholder="Ex: Technoblade, Dream, MumboJumbo..."
							bind:value={searchNick}
							onkeydown={(e) => e.key === 'Enter' && handleSearchNick()}
							class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white placeholder-white/30 focus:outline-none focus:border-blue-500 transition-colors font-medium"
						/>
					</div>
					<button
						type="button"
						onclick={handleSearchNick}
						disabled={isSearchingNick}
						class="px-5 py-2.5 rounded-2xl bg-blue-600 hover:bg-blue-500 text-white font-black text-xs transition-all shadow-md active:scale-95 disabled:opacity-50 cursor-pointer flex items-center gap-2 shrink-0"
					>
						{#if isSearchingNick}
							<RefreshCw class="w-3.5 h-3.5 animate-spin" />
							<span>Buscando...</span>
						{:else}
							<Search class="w-3.5 h-3.5" />
							<span>Buscar</span>
						{/if}
					</button>
				</div>

				<!-- Quick Popular Skins Badges -->
				<div class="flex items-center gap-1.5 flex-wrap pt-1">
					<span class="text-[10px] text-white/40 font-bold mr-1">Populares:</span>
					{#each popularSkins as s}
						<button
							type="button"
							onclick={() => handleQuickSelectSkin(s)}
							class="px-2.5 py-1 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 hover:border-white/20 text-[11px] font-bold text-white/70 hover:text-white transition-all cursor-pointer active:scale-95"
						>
							{s.name}
						</button>
					{/each}
				</div>
			</div>

			<!-- Capes Studio -->
			<div class="p-5 rounded-3xl bg-[#141518] border border-white/5 shadow-xl space-y-3.5">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Layers class="w-4 h-4 text-amber-400" />
						<h3 class="text-xs font-extrabold text-white uppercase tracking-wider">
							Capas Exclusivas & Oficiais
						</h3>
					</div>
					{#if selectedCape === "custom"}
						<button
							type="button"
							onclick={handleCustomCapeUpload}
							class="text-[11px] font-bold text-blue-400 hover:underline flex items-center gap-1 cursor-pointer"
						>
							<Upload class="w-3 h-3" /> Trocar .PNG da Capa
						</button>
					{/if}
				</div>

				<!-- Capes Grid -->
				<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2.5 max-h-[220px] overflow-y-auto custom-scrollbar pr-1">
					{#each capeList as c}
						{@const isSelected = selectedCape === c.id}
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
							class="p-2.5 rounded-2xl border transition-all cursor-pointer flex flex-col items-center text-center gap-2 relative overflow-hidden group {isSelected ? 'bg-blue-600/15 border-blue-500 shadow-md' : 'bg-[#1a1b20] hover:bg-[#202127] border-white/5 hover:border-white/15'}"
						>
							<div class="w-10 h-16 rounded-lg bg-black/40 border border-white/10 flex items-center justify-center overflow-hidden shrink-0 shadow-inner">
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
									<Upload class="w-5 h-5 text-blue-400/60 group-hover:text-blue-400" />
								{:else}
									<span class="text-[10px] text-white/30 font-bold uppercase">OFF</span>
								{/if}
							</div>

							<div class="min-w-0 w-full">
								<span class="text-[11px] font-black truncate block text-white group-hover:text-blue-300 transition-colors">
									{c.name}
								</span>
								<span class="text-[9px] text-white/40 truncate block mt-0.5">
									{c.desc}
								</span>
							</div>

							{#if isSelected}
								<div class="absolute top-1.5 right-1.5 w-2 h-2 rounded-full bg-blue-400"></div>
							{/if}
						</button>
					{/each}
				</div>
			</div>

			<!-- Active Player Info & Mode Note -->
			<div class="p-4 rounded-2xl bg-[#111216] border border-white/5 flex items-center justify-between text-xs text-white/60">
				<div class="flex items-center gap-3">
					<div class="w-8 h-8 rounded-xl bg-blue-500/15 text-blue-400 border border-blue-500/30 flex items-center justify-center font-black">
						{username.slice(0, 1).toUpperCase()}
					</div>
					<div>
						<div class="font-bold text-white flex items-center gap-2">
							{username}
							<span class="text-[10px] font-semibold px-2 py-0.2 rounded-full {isMicrosoft ? 'bg-sky-500/15 text-sky-300 border border-sky-500/30' : 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/30'}">
								{isMicrosoft ? 'Conta Microsoft' : 'Conta Offline'}
							</span>
						</div>
						<div class="text-[10px] text-white/40 mt-0.5">
							Modelo: {skinType === "alex" ? "Slim (3px)" : "Clássico (4px)"} · Capa: {capeList.find(c => c.id === selectedCape)?.name || "Nenhuma"}
						</div>
					</div>
				</div>

				<button
					type="button"
					onclick={handleApplyToAccount}
					class="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-white font-bold text-xs transition-all cursor-pointer active:scale-95"
				>
					Aplicar
				</button>
			</div>

		</div>

	</div>

</div>
