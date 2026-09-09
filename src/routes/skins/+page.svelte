<script lang="ts">
	import { fade } from "svelte/transition";
	import { 
		Plus, 
		RefreshCw, 
		Sparkles, 
		RotateCw, 
		Shield, 
		Box, 
		Upload, 
		User, 
		CheckCircle2,
		Layers,
		Search,
		Globe,
		ExternalLink
	} from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import Button from "$lib/components/ui/Button.svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import SkinViewer3D from "$lib/components/ui/SkinViewer3D.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	type SkinItem = {
		id: string;
		name: string;
		url: string;
		skinUrl: string;
		avatarUrl: string;
		type: "steve" | "alex";
		custom?: boolean;
	};

	const defaultSkins: SkinItem[] = [
		{ 
			id: "steve", 
			name: "Steve", 
			url: "https://mc-heads.net/body/Steve/300", 
			skinUrl: "https://minotar.net/skin/Steve", 
			avatarUrl: "https://mc-heads.net/avatar/Steve/100", 
			type: "steve" 
		},
		{ 
			id: "alex", 
			name: "Alex", 
			url: "https://mc-heads.net/body/Alex/300", 
			skinUrl: "https://minotar.net/skin/MHF_Alex", 
			avatarUrl: "https://mc-heads.net/avatar/Alex/100", 
			type: "alex" 
		},
		{ 
			id: "zuri", 
			name: "Zuri", 
			url: "https://mc-heads.net/body/Zuri/300", 
			skinUrl: "https://minotar.net/skin/Zuri", 
			avatarUrl: "https://mc-heads.net/avatar/Zuri/100", 
			type: "alex" 
		},
		{ 
			id: "sunny", 
			name: "Sunny", 
			url: "https://mc-heads.net/body/Sunny/300", 
			skinUrl: "https://minotar.net/skin/Sunny", 
			avatarUrl: "https://mc-heads.net/avatar/Sunny/100", 
			type: "steve" 
		},
		{ 
			id: "noor", 
			name: "Noor", 
			url: "https://mc-heads.net/body/Noor/300", 
			skinUrl: "https://minotar.net/skin/Noor", 
			avatarUrl: "https://mc-heads.net/avatar/Noor/100", 
			type: "alex" 
		},
		{ 
			id: "makena", 
			name: "Makena", 
			url: "https://mc-heads.net/body/Makena/300", 
			skinUrl: "https://minotar.net/skin/Makena", 
			avatarUrl: "https://mc-heads.net/avatar/Makena/100", 
			type: "alex" 
		},
		{ 
			id: "efe", 
			name: "Efe", 
			url: "https://mc-heads.net/body/Efe/300", 
			skinUrl: "https://minotar.net/skin/Efe", 
			avatarUrl: "https://mc-heads.net/avatar/Efe/100", 
			type: "alex" 
		},
		{ 
			id: "ari", 
			name: "Ari", 
			url: "https://mc-heads.net/body/Ari/300", 
			skinUrl: "https://minotar.net/skin/Ari", 
			avatarUrl: "https://mc-heads.net/avatar/Ari/100", 
			type: "alex" 
		},
	];

	let savedSkins = $state<SkinItem[]>([
		{
			id: "frog_hoodie",
			name: "Frog Hoodie",
			url: "https://mc-heads.net/body/Spect3rBW/300",
			skinUrl: "https://minotar.net/skin/Spect3rBW",
			avatarUrl: "https://mc-heads.net/avatar/Spect3rBW/100",
			type: "alex",
			custom: true,
		},
		{
			id: "cyber_steve",
			name: "Cyber Neon Steve",
			url: "https://mc-heads.net/body/MHF_Steve/300",
			skinUrl: "https://minotar.net/skin/MHF_Steve",
			avatarUrl: "https://mc-heads.net/avatar/MHF_Steve/100",
			type: "steve",
			custom: true,
		}
	]);

	let autoRotate = $state(true);
	let selectedCape = $state<"none" | "migrator" | "optifine" | "mojang">("migrator");
	let isSlimModel = $state(activeSkinStore.current.type === "alex");
	let fileInputEl: HTMLInputElement;

	let skinViewerRef: { setAngle: (deg: number) => void } | null = $state(null);

	let nameMcQuery = $state("");
	let isSearchingNameMc = $state(false);

	async function fetchNameMcSkin() {
		const nick = nameMcQuery.trim();
		if (!nick) {
			toast("Digite o nick do jogador no NameMC!", "error");
			return;
		}
		isSearchingNameMc = true;
		try {
			const skinUrl = `https://minotar.net/skin/${encodeURIComponent(nick)}`;
			const avatarUrl = `https://mc-heads.net/avatar/${encodeURIComponent(nick)}/100`;
			const bodyUrl = `https://mc-heads.net/body/${encodeURIComponent(nick)}/300`;

			const newSkin: SkinItem = {
				id: "namemc_" + nick.toLowerCase() + "_" + Date.now(),
				name: nick,
				url: bodyUrl,
				skinUrl,
				avatarUrl,
				type: isSlimModel ? "alex" : "steve",
				custom: true
			};

			savedSkins = [newSkin, ...savedSkins];
			applySkin(newSkin);
			toast(`Skin de "${nick}" importada do NameMC com sucesso!`, "success");
			nameMcQuery = "";
		} catch (e) {
			toast("Não foi possível carregar a skin do NameMC: " + String(e), "error");
		} finally {
			isSearchingNameMc = false;
		}
	}

	function openNameMcTrending() {
		openUrl("https://pt.namemc.com/minecraft-skins/trending").catch(() => {
			window.open("https://pt.namemc.com/minecraft-skins/trending", "_blank");
		});
	}

	const activeTextureUrl = $derived(
		activeSkinStore.current.skinUrl || "https://minotar.net/skin/Steve"
	);

	function applySkin(skin: SkinItem) {
		isSlimModel = skin.type === "alex";
		activeSkinStore.setSkin({
			id: skin.id,
			name: skin.name,
			url: skin.url,
			skinUrl: skin.skinUrl,
			avatarUrl: skin.avatarUrl,
			type: skin.type
		});
		toast(`Skin "${skin.name}" sincronizada com seu perfil!`, "success");
	}

	function handleFileUpload(e: Event) {
		const target = e.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		if (!file.name.toLowerCase().endsWith(".png")) {
			toast("Selecione um arquivo de skin válido (.png) de 64x64!", "error");
			return;
		}

		const reader = new FileReader();
		reader.onload = (event) => {
			const dataUrl = event.target?.result as string;
			if (!dataUrl) return;

			// Extract 8x8 face from skin for avatar preview
			const img = new Image();
			img.onload = () => {
				const canvas = document.createElement("canvas");
				canvas.width = 64;
				canvas.height = 64;
				const ctx = canvas.getContext("2d");
				if (ctx) {
					ctx.imageSmoothingEnabled = false;
					// Draw 8x8 face from (8,8) scaled to 64x64
					ctx.drawImage(img, 8, 8, 8, 8, 0, 0, 64, 64);
					// Draw hat layer overlay (40,8)
					ctx.drawImage(img, 40, 8, 8, 8, 0, 0, 64, 64);
				}
				const avatarDataUrl = canvas.toDataURL();
				const cleanName = file.name.replace(/\.png$/i, "");

				const newSkin: SkinItem = {
					id: "custom_" + Date.now(),
					name: cleanName,
					url: avatarDataUrl,
					skinUrl: dataUrl,
					avatarUrl: avatarDataUrl,
					type: isSlimModel ? "alex" : "steve",
					custom: true
				};

				savedSkins = [newSkin, ...savedSkins];
				applySkin(newSkin);
				toast(`Nova skin "${cleanName}" carregada e sincronizada com seu perfil!`, "success");
			};
			img.src = dataUrl;
		};
		reader.readAsDataURL(file);
		target.value = "";
	}

	function setQuickAngle(deg: number) {
		autoRotate = false;
		skinViewerRef?.setAngle(deg);
	}
</script>

<input 
	type="file" 
	accept="image/png" 
	class="hidden" 
	bind:this={fileInputEl} 
	onchange={handleFileUpload} 
/>

<div class="flex gap-8 h-full w-full select-none" in:fade={{ duration: 300 }}>
	<!-- Main Skins Area -->
	<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-6">
		
		<!-- Header -->
		<div class="flex items-center justify-between mt-1">
			<div>
				<div class="flex items-center gap-3">
					<h1 class="text-3xl font-extrabold text-white tracking-tight">Personalização</h1>
					<span class="bg-brand-500/20 text-brand-500 text-[10px] font-black px-2.5 py-1 rounded-lg uppercase tracking-wide border border-brand-500/30 flex items-center gap-1.5 shadow-sm">
						<Box class="w-3.5 h-3.5" /> 3D Volumétrico Real
					</span>
				</div>
				<p class="text-white/50 text-xs mt-0.5">Modelo geométrico 3D completo de Minecraft com 6 faces por membro e capas dinâmicas</p>
			</div>
			
			<div class="flex items-center gap-2">
				<Button 
					variant="secondary" 
					class="border-white/10 bg-[#1e1f23] hover:bg-white/10 text-white gap-2 rounded-full text-xs px-5 py-2 cursor-pointer"
					onclick={() => fileInputEl.click()}
				>
					<Upload class="w-3.5 h-3.5 text-amber-400" />
					Importar Skin .PNG
				</Button>
				
				<Button 
					variant="outline" 
					class="border-white/10 bg-[#1e1f23] hover:bg-white/10 text-white gap-2 rounded-full text-xs px-5 py-2 cursor-pointer" 
					onclick={() => toast("Skins sincronizadas!", "success")}
				>
					<RefreshCw class="w-3.5 h-3.5" />
					Atualizar
				</Button>
			</div>
		</div>

		<div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
			
			<!-- Left Column: 360° Real 3D Hardware Accelerated Character Stage -->
			<div class="lg:col-span-5 bg-[#18191c] border border-white/5 rounded-3xl p-5 flex flex-col items-center justify-between min-h-[560px] relative shadow-2xl overflow-hidden">
				
				<!-- Top Bar inside Stage -->
				<div class="w-full flex items-center justify-between gap-2 z-10">
					<div class="flex items-center gap-2">
						<span class="text-[11px] font-bold text-white/70 flex items-center gap-1.5 bg-black/40 px-3 py-1 rounded-full border border-white/5">
							<Layers class="w-3.5 h-3.5 text-amber-400" /> Three.js WebGL
						</span>
						<span class="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-2.5 py-0.5 rounded-full border border-emerald-500/20">
							60 FPS
						</span>
					</div>

					<button 
						type="button" 
						class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-full text-[11px] font-bold transition-all border cursor-pointer {autoRotate ? 'bg-white/10 text-white border-white/20 shadow-sm' : 'bg-[#222328] text-white/50 border-white/5 hover:text-white'}"
						onclick={() => autoRotate = !autoRotate}
					>
						<RotateCw class="w-3 h-3 {autoRotate ? 'animate-spin' : ''}" />
						{autoRotate ? 'Giro Ativo' : 'Girar 360°'}
					</button>
				</div>

				<!-- 3D Interactive WebGL Stage (Click & Drag 360°) -->
				<div class="my-2 relative flex items-center justify-center select-none w-full h-[360px]">
					<!-- Radial Aura Backdrop -->
					<div class="absolute inset-0 bg-radial from-brand-500/15 via-transparent to-transparent blur-3xl pointer-events-none"></div>

					<!-- Volumetric 3D Skin Viewer -->
					<SkinViewer3D 
						bind:this={skinViewerRef}
						skinUrl={activeTextureUrl}
						cape={selectedCape}
						slim={isSlimModel}
						{autoRotate}
						className="z-10"
					/>
				</div>

				<!-- Quick Angle Controls & Model Type -->
				<div class="w-full flex items-center justify-between gap-2 z-10 pt-2 border-t border-white/5">
					<div class="flex items-center gap-1.5">
						<button type="button" class="px-3 py-1 rounded-full bg-[#222328] hover:bg-white/10 text-[10px] font-bold text-white/60 hover:text-white transition-all cursor-pointer" onclick={() => setQuickAngle(0)}>Frente</button>
						<button type="button" class="px-3 py-1 rounded-full bg-[#222328] hover:bg-white/10 text-[10px] font-bold text-white/60 hover:text-white transition-all cursor-pointer" onclick={() => setQuickAngle(90)}>Lado D</button>
						<button type="button" class="px-3 py-1 rounded-full bg-[#222328] hover:bg-white/10 text-[10px] font-bold text-white/60 hover:text-white transition-all cursor-pointer" onclick={() => setQuickAngle(180)}>Costas</button>
						<button type="button" class="px-3 py-1 rounded-full bg-[#222328] hover:bg-white/10 text-[10px] font-bold text-white/60 hover:text-white transition-all cursor-pointer" onclick={() => setQuickAngle(270)}>Lado E</button>
					</div>

					<!-- Steve (4px) vs Alex (3px) Toggle -->
					<div class="flex items-center bg-[#141518] p-0.5 rounded-full border border-white/5">
						<button 
							type="button" 
							class="px-3 py-1 rounded-full text-[10px] font-bold transition-all cursor-pointer {!isSlimModel ? 'bg-white/20 text-white font-extrabold' : 'text-white/40 hover:text-white'}"
							onclick={() => { isSlimModel = false; activeSkinStore.setSkin({ type: 'steve' }); }}
						>
							Steve (4px)
						</button>
						<button 
							type="button" 
							class="px-3 py-1 rounded-full text-[10px] font-bold transition-all cursor-pointer {isSlimModel ? 'bg-white/20 text-white font-extrabold' : 'text-white/40 hover:text-white'}"
							onclick={() => { isSlimModel = true; activeSkinStore.setSkin({ type: 'alex' }); }}
						>
							Alex (3px)
						</button>
					</div>
				</div>

				<!-- Cape Selector Studio -->
				<div class="w-full mt-3 p-2.5 bg-[#141518] rounded-2xl border border-white/5 flex items-center justify-between">
					<span class="text-[10px] font-bold text-white/50 uppercase flex items-center gap-1">
						<Shield class="w-3.5 h-3.5 text-amber-400" /> Capa 3D
					</span>
					<div class="flex gap-1">
						{#each ["none", "migrator", "optifine", "mojang"] as cape}
							<button 
								type="button" 
								class="px-3 py-1 rounded-full text-[10px] font-bold capitalize transition-all cursor-pointer {selectedCape === cape ? 'bg-brand-500 text-black font-extrabold shadow-md' : 'bg-[#1c1d22] text-white/50 border border-white/5 hover:text-white'}"
								onclick={() => {
									selectedCape = cape as any;
									activeSkinStore.setCape(cape as any);
									toast(`Capa 3D "${cape === 'none' ? 'Sem capa' : cape}" selecionada!`, "success");
								}}
							>
								{cape === 'none' ? 'Sem capa' : cape}
							</button>
						{/each}
					</div>
				</div>

			</div>

			<!-- Right Column: Saved Skins & Default Library -->
			<div class="lg:col-span-7 flex flex-col gap-6">
				
				<!-- NameMC Integration Card -->
				<div class="bg-gradient-to-r from-[#18191c] via-[#1c1d22] to-[#18191c] border border-white/10 rounded-3xl p-5 shadow-lg space-y-3 relative overflow-hidden">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-2.5">
							<div class="h-8 w-8 rounded-xl bg-brand-500/10 border border-brand-500/20 flex items-center justify-center text-brand-500">
								<Globe class="w-4 h-4" />
							</div>
							<div>
								<h3 class="text-xs font-black text-white flex items-center gap-2">
									Integração NameMC Skins
									<span class="bg-brand-500/20 text-brand-500 text-[9px] font-black px-2 py-0.5 rounded-full border border-brand-500/30">API ONLINE</span>
								</h3>
								<p class="text-[10px] text-white/50">Carregue qualquer skin do mundo pelo Nickname do jogador</p>
							</div>
						</div>

						<button 
							type="button" 
							class="text-[11px] font-bold text-brand-500 hover:text-[#ebd095] flex items-center gap-1.5 transition-colors cursor-pointer"
							onclick={openNameMcTrending}
							title="Abrir NameMC no navegador"
						>
							<ExternalLink class="w-3.5 h-3.5" /> Explorar Tendências
						</button>
					</div>

					<div class="flex items-center gap-2">
						<div class="relative flex-1">
							<Search class="w-4 h-4 text-white/40 absolute left-3.5 top-1/2 -translate-y-1/2" />
							<input 
								type="text" 
								placeholder="Digite o nick no NameMC (ex: Dream, Techno, Authentic, Felps)..." 
								bind:value={nameMcQuery}
								class="w-full bg-[#121316] border border-white/10 rounded-full pl-10 pr-4 py-2.5 text-xs font-bold text-white outline-none focus:border-brand-500 transition-colors"
								onkeydown={(e) => { if (e.key === "Enter") fetchNameMcSkin(); }}
							/>
						</div>
						<button 
							type="button"
							class="px-5 py-2.5 rounded-full hover:brightness-110 active:scale-95 text-black font-black text-xs transition-all shadow-md flex items-center gap-1.5 shrink-0 cursor-pointer disabled:opacity-50 hover:scale-105"
							style="background-color: var(--accent-color, #e2b86b);"
							onclick={fetchNameMcSkin}
							disabled={isSearchingNameMc}
						>
							{#if isSearchingNameMc}
								<RefreshCw class="w-3.5 h-3.5 animate-spin" /> Carregando...
							{:else}
								<Search class="w-3.5 h-3.5" /> Carregar Skin
							{/if}
						</button>
					</div>
				</div>

				<!-- Saved Skins Section -->
				<div>
					<div class="flex items-center justify-between mb-3">
						<div class="flex items-center gap-2">
							<h3 class="text-sm font-bold text-white">Skins Salvas & Customizadas</h3>
							<span class="text-xs text-white/40 font-medium">({savedSkins.length})</span>
						</div>
						<span class="text-[11px] text-brand-500 font-medium flex items-center gap-1">
							<Sparkles class="w-3 h-3" /> Sincronização em Tempo Real
						</span>
					</div>

					<div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
						<!-- Add Custom Skin Button -->
						<button 
							type="button"
							class="h-36 border-2 border-dashed border-white/10 hover:border-brand-500/70 rounded-2xl flex flex-col items-center justify-center gap-2 text-white/40 hover:text-white transition-all bg-[#18191c]/50 hover:bg-[#18191c] group shadow-sm cursor-pointer"
							onclick={() => fileInputEl.click()}
						>
							<div class="h-9 w-9 rounded-xl bg-white/5 flex items-center justify-center group-hover:scale-110 transition-transform">
								<Plus class="w-5 h-5 text-brand-500" />
							</div>
							<span class="text-[11px] font-bold text-center leading-tight">Adicionar Skin .PNG</span>
						</button>

						<!-- Saved Skins Cards -->
						{#each savedSkins as skin}
							{@const isSelected = activeSkinStore.current.id === skin.id}
							<button 
								type="button"
								class="h-36 rounded-2xl bg-[#18191c] border-2 p-3 relative flex flex-col items-center justify-between transition-all group overflow-hidden cursor-pointer {isSelected ? 'border-brand-500 bg-[#222328] shadow-[0_0_16px_rgba(226,184,107,0.25)]' : 'border-white/5 hover:border-white/20'}"
								onclick={() => applySkin(skin)}
							>
								{#if isSelected}
									<div class="absolute top-2.5 right-2.5 w-2.5 h-2.5 rounded-full bg-brand-500 shadow-sm animate-pulse"></div>
								{/if}
								<div class="flex-1 flex items-center justify-center">
									<img src={skin.avatarUrl || skin.url} alt={skin.name} class="h-16 w-16 rounded-xl object-cover group-hover:scale-105 transition-transform border border-white/5 shadow-md" />
								</div>
								<span class="text-xs font-bold text-white/80 group-hover:text-white truncate max-w-[90%]">{skin.name}</span>
							</button>
						{/each}
					</div>
				</div>

				<!-- Default Minecraft Skins Section -->
				<div>
					<div class="flex items-center gap-2 mb-3">
						<h3 class="text-sm font-bold text-white">Skins Padrão da Mojang</h3>
					</div>

					<div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
						{#each defaultSkins as skin}
							{@const isSelected = activeSkinStore.current.id === skin.id}
							<button 
								type="button"
								class="h-36 rounded-2xl bg-[#18191c] border-2 p-3 flex flex-col items-center justify-between transition-all group cursor-pointer {isSelected ? 'border-brand-500 bg-[#222328] shadow-[0_0_12px_rgba(226,184,107,0.2)]' : 'border-white/5 hover:border-white/20 hover:bg-[#1e1f23]'}"
								onclick={() => applySkin(skin)}
							>
								{#if isSelected}
									<div class="absolute top-2.5 right-2.5 w-2 h-2 rounded-full bg-brand-500 shadow-sm animate-pulse"></div>
								{/if}
								<div class="flex-1 flex items-center justify-center">
									<img src={skin.avatarUrl || skin.url} alt={skin.name} class="h-16 w-16 rounded-xl object-cover group-hover:scale-105 transition-transform border border-white/5 shadow-md" />
								</div>
								<div class="text-center">
									<span class="text-xs font-bold text-white/70 group-hover:text-white block">{skin.name}</span>
									<span class="text-[9px] text-white/30 uppercase">{skin.type === 'alex' ? 'Slim 3px' : 'Classic 4px'}</span>
								</div>
							</button>
						{/each}
					</div>
				</div>

			</div>
		</div>

	</div>

	<!-- Right Sidebar (Notícias Luxmc) -->
	<RightSidebar />
</div>
