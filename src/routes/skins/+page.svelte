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
		ExternalLink,
		Shirt,
		ShoppingBag,
		Heart,
		Download,
		Tag,
		Flame,
		Star,
		Check
	} from "lucide-svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import Button from "$lib/components/ui/Button.svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import SkinViewer3D from "$lib/components/ui/SkinViewer3D.svelte";
	import { activeSkinStore, type CapeType } from "$lib/stores/skin.svelte";
	import { getCapePreviewDataUrl } from "$lib/utils/capeTextures";
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

	type MarketplaceSkin = {
		id: string;
		name: string;
		category: "pvp" | "aesthetic" | "anime" | "medieval" | "creators";
		categoryLabel: string;
		author: string;
		downloads: string;
		likes: string;
		tags: string[];
		type: "steve" | "alex";
		avatarUrl: string;
		skinUrl: string;
		url: string;
	};

	type CapeItem = {
		id: CapeType;
		name: string;
		event: string;
		description: string;
		rarity: "Lendária" | "Mítica" | "Rara" | "Exclusiva" | "Especial" | "Popular";
		badgeColor: string;
		borderGradient: string;
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
		}
	];

	const marketplaceSkins: MarketplaceSkin[] = [
		{
			id: "community_techno",
			name: "Technoblade Memorial",
			category: "creators",
			categoryLabel: "Criadores",
			author: "Blood God Legacy",
			downloads: "452k",
			likes: "98.2k",
			tags: ["PvP", "Coroa", "Lenda", "Memorial"],
			type: "steve",
			avatarUrl: "https://mc-heads.net/avatar/Technoblade/100",
			skinUrl: "https://minotar.net/skin/Technoblade",
			url: "https://mc-heads.net/body/Technoblade/300"
		},
		{
			id: "community_dream",
			name: "Dream Speedrunner",
			category: "creators",
			categoryLabel: "Criadores",
			author: "Dream Team",
			downloads: "389k",
			likes: "72.4k",
			tags: ["Speedrun", "Verde", "Manhunt"],
			type: "steve",
			avatarUrl: "https://mc-heads.net/avatar/Dream/100",
			skinUrl: "https://minotar.net/skin/Dream",
			url: "https://mc-heads.net/body/Dream/300"
		},
		{
			id: "community_viniccius13",
			name: "Viniccius13 Redstone",
			category: "creators",
			categoryLabel: "Criadores",
			author: "Redstone Gang",
			downloads: "294k",
			likes: "64.1k",
			tags: ["Redstone", "Brasil", "Casa Automática"],
			type: "steve",
			avatarUrl: "https://mc-heads.net/avatar/Viniccius13/100",
			skinUrl: "https://minotar.net/skin/Viniccius13",
			url: "https://mc-heads.net/body/Viniccius13/300"
		},
		{
			id: "community_forever",
			name: "Forever Player",
			category: "creators",
			categoryLabel: "Criadores",
			author: "QSMP / Brasil",
			downloads: "210k",
			likes: "45.7k",
			tags: ["QSMP", "Brasil", "Hardcore"],
			type: "steve",
			avatarUrl: "https://mc-heads.net/avatar/ForeverPlayer/100",
			skinUrl: "https://minotar.net/skin/ForeverPlayer",
			url: "https://mc-heads.net/body/ForeverPlayer/300"
		},
		{
			id: "community_gojo",
			name: "Gojo Satoru (Limitless)",
			category: "anime",
			categoryLabel: "Anime & Geek",
			author: "OtakuCraft",
			downloads: "312k",
			likes: "84.3k",
			tags: ["Jujutsu Kaisen", "Venda", "Infinito"],
			type: "steve",
			avatarUrl: "https://mc-heads.net/avatar/Gojo/100",
			skinUrl: "https://minotar.net/skin/Gojo",
			url: "https://mc-heads.net/body/Gojo/300"
		},
		{
			id: "community_luffy",
			name: "Monkey D. Luffy Gear 5",
			category: "anime",
			categoryLabel: "Anime & Geek",
			author: "GrandLine",
			downloads: "275k",
			likes: "71.0k",
			tags: ["One Piece", "Nika", "Gear 5", "Branco"],
			type: "alex",
			avatarUrl: "https://mc-heads.net/avatar/Luffy/100",
			skinUrl: "https://minotar.net/skin/Luffy",
			url: "https://mc-heads.net/body/Luffy/300"
		},
		{
			id: "community_tanjiro",
			name: "Tanjiro Kamado",
			category: "anime",
			categoryLabel: "Anime & Geek",
			author: "DemonSlayer",
			downloads: "198k",
			likes: "52.4k",
			tags: ["Kimetsu", "Haori", "Fogo", "Espada"],
			type: "steve",
			avatarUrl: "https://mc-heads.net/avatar/Tanjiro/100",
			skinUrl: "https://minotar.net/skin/Tanjiro",
			url: "https://mc-heads.net/body/Tanjiro/300"
		},
		{
			id: "community_void_slayer",
			name: "Void Slayer Ninja",
			category: "pvp",
			categoryLabel: "PvP / Tryhard",
			author: "HypixelPvP",
			downloads: "180k",
			likes: "41.2k",
			tags: ["Bedwars", "Dark", "Neon Purple", "Tryhard"],
			type: "alex",
			avatarUrl: "https://mc-heads.net/avatar/Murd/100",
			skinUrl: "https://minotar.net/skin/Murd",
			url: "https://mc-heads.net/body/Murd/300"
		},
		{
			id: "community_frost_assassin",
			name: "Frostbite Assassin",
			category: "pvp",
			categoryLabel: "PvP / Tryhard",
			author: "GlacierMC",
			downloads: "165k",
			likes: "38.9k",
			tags: ["Gelo", "Ciano", "PvP", "Máscara"],
			type: "steve",
			avatarUrl: "https://mc-heads.net/avatar/Frost/100",
			skinUrl: "https://minotar.net/skin/Frost",
			url: "https://mc-heads.net/body/Frost/300"
		},
		{
			id: "community_sakura_matcha",
			name: "Sakura Matcha Hoodie",
			category: "aesthetic",
			categoryLabel: "Aesthetic & Pastel",
			author: "CozyStudio",
			downloads: "145k",
			likes: "39.8k",
			tags: ["Pastel", "Moletom", "Rosa", "Cozy"],
			type: "alex",
			avatarUrl: "https://mc-heads.net/avatar/Cherry/100",
			skinUrl: "https://minotar.net/skin/Cherry",
			url: "https://mc-heads.net/body/Cherry/300"
		},
		{
			id: "community_cloud_dreamer",
			name: "Cloud Dreamer Pastel",
			category: "aesthetic",
			categoryLabel: "Aesthetic & Pastel",
			author: "AestheticVibes",
			downloads: "120k",
			likes: "31.5k",
			tags: ["Nuvens", "Azul Pastel", "Fone", "Casual"],
			type: "alex",
			avatarUrl: "https://mc-heads.net/avatar/Cloud/100",
			skinUrl: "https://minotar.net/skin/Cloud",
			url: "https://mc-heads.net/body/Cloud/300"
		},
		{
			id: "community_paladin",
			name: "Paladino do Sol Arcano",
			category: "medieval",
			categoryLabel: "Medieval & RPG",
			author: "EldoriaRPG",
			downloads: "115k",
			likes: "28.2k",
			tags: ["Armadura", "Ouro", "Cavaleiro", "RPG"],
			type: "steve",
			avatarUrl: "https://mc-heads.net/avatar/Knight/100",
			skinUrl: "https://minotar.net/skin/Knight",
			url: "https://mc-heads.net/body/Knight/300"
		},
		{
			id: "community_dark_mage",
			name: "Arquimago do Nether",
			category: "medieval",
			categoryLabel: "Medieval & RPG",
			author: "NetherGuild",
			downloads: "108k",
			likes: "26.4k",
			tags: ["Nether", "Mago", "Fogo", "Manto"],
			type: "steve",
			avatarUrl: "https://mc-heads.net/avatar/Wizard/100",
			skinUrl: "https://minotar.net/skin/Wizard",
			url: "https://mc-heads.net/body/Wizard/300"
		}
	];

	const capeCatalog: CapeItem[] = [
		{
			id: "luxmc",
			name: "Capa Luxmc Ouro Real",
			event: "Exclusiva Oficial Luxmc",
			description: "Veludo obsidiano com bordas de ouro e o brasão estelar dourado 'L' do Luxmc Launcher.",
			rarity: "Exclusiva",
			badgeColor: "bg-amber-500/20 text-amber-300 border-amber-500/30",
			borderGradient: "from-amber-500/30 to-yellow-600/10"
		},
		{
			id: "migrator",
			name: "Capa do Migrador",
			event: "Migração Mojang ➔ Microsoft",
			description: "Capa vinho imperial com o clássico brasão dourado em relevo romano.",
			rarity: "Rara",
			badgeColor: "bg-rose-500/20 text-rose-300 border-rose-500/30",
			borderGradient: "from-red-600/30 to-amber-700/10"
		},
		{
			id: "optifine",
			name: "Capa OptiFine Clássica",
			event: "OptiFine Donation Cape",
			description: "A lendária capa vermelha vibrante com o monograma 'OF' branco mundialmente conhecido.",
			rarity: "Popular",
			badgeColor: "bg-red-500/20 text-red-300 border-red-500/30",
			borderGradient: "from-red-500/30 to-rose-700/10"
		},
		{
			id: "mojang",
			name: "Capa Mojang Studios",
			event: "Equipe Mojang / Staff",
			description: "Vermelho escarlate com o emblemático símbolo geométrico dos criadores do Minecraft.",
			rarity: "Lendária",
			badgeColor: "bg-red-600/20 text-red-300 border-red-600/30",
			borderGradient: "from-red-700/30 to-rose-950/20"
		},
		{
			id: "minecon2011",
			name: "Minecon 2011",
			event: "Las Vegas, EUA",
			description: "A primeira capa histórica da Minecon com a icônica picareta dourada sobre fundo carmesim.",
			rarity: "Mítica",
			badgeColor: "bg-amber-500/20 text-amber-300 border-amber-500/30",
			borderGradient: "from-amber-600/30 to-red-800/10"
		},
		{
			id: "minecon2012",
			name: "Minecon 2012",
			event: "Disneyland Paris, França",
			description: "Azul marinho imperial com picareta de ouro distribuída na primeira Minecon europeia.",
			rarity: "Mítica",
			badgeColor: "bg-blue-500/20 text-blue-300 border-blue-500/30",
			borderGradient: "from-blue-600/30 to-indigo-800/10"
		},
		{
			id: "minecon2013",
			name: "Minecon 2013",
			event: "Orlando, Flórida",
			description: "Verde floresta profundo com a ilustração do pistão mecânico da lendária Redstone Update.",
			rarity: "Mítica",
			badgeColor: "bg-emerald-500/20 text-emerald-300 border-emerald-500/30",
			borderGradient: "from-emerald-600/30 to-green-800/10"
		},
		{
			id: "minecon2015",
			name: "Minecon 2015",
			event: "Londres, Reino Unido",
			description: "Ciano escuro com a face protetora do Iron Golem e a rosa vermelha dos aldeões.",
			rarity: "Mítica",
			badgeColor: "bg-teal-500/20 text-teal-300 border-teal-500/30",
			borderGradient: "from-teal-600/30 to-cyan-800/10"
		},
		{
			id: "minecon2016",
			name: "Minecon 2016",
			event: "Anaheim, Califórnia",
			description: "Roxo etéreo do End com o olhar penetrante e partículas místicas do Enderman.",
			rarity: "Mítica",
			badgeColor: "bg-purple-500/20 text-purple-300 border-purple-500/30",
			borderGradient: "from-purple-600/30 to-fuchsia-900/10"
		},
		{
			id: "cherry",
			name: "Capa Flor de Cerejeira",
			event: "Atualização 1.20 Trails & Tales",
			description: "Rosa sakura suave com pétalas caindo inspirada nas florestas floridas de cerejeiras.",
			rarity: "Especial",
			badgeColor: "bg-pink-500/20 text-pink-300 border-pink-500/30",
			borderGradient: "from-pink-500/30 to-rose-400/10"
		},
		{
			id: "vanilla",
			name: "Capa Vanilla 15 Anos",
			event: "15º Aniversário Minecraft",
			description: "Design dividido entre crepúsculo estrelado e sol radiante para donos do Java e Bedrock.",
			rarity: "Rara",
			badgeColor: "bg-indigo-500/20 text-indigo-300 border-indigo-500/30",
			borderGradient: "from-indigo-600/30 to-amber-600/10"
		},
		{
			id: "tiktok",
			name: "Capa TikTok Glitch",
			event: "Comemoração 15 Anos",
			description: "Preto acetinado com o logotipo musical em efeito de aberração cromática ciano e magenta.",
			rarity: "Especial",
			badgeColor: "bg-cyan-500/20 text-cyan-300 border-cyan-500/30",
			borderGradient: "from-cyan-500/30 to-rose-600/10"
		},
		{
			id: "twitch",
			name: "Capa Twitch Purple",
			event: "Drops Comemorativos 15 Anos",
			description: "Roxo elétrico com o clássico balão de diálogo 'Glitch' dos streams de Minecraft.",
			rarity: "Especial",
			badgeColor: "bg-purple-600/20 text-purple-400 border-purple-600/30",
			borderGradient: "from-purple-600/30 to-violet-800/10"
		}
	];

	let activeTab = $state<"wardrobe" | "marketplace" | "capes">("wardrobe");
	let selectedMarketplaceCategory = $state<string>("todos");
	let marketplaceSearch = $state<string>("");

	let savedSkins = $state<SkinItem[]>([
		{
			id: "cyber_steve",
			name: "Cyber Neon Steve",
			url: "https://mc-heads.net/body/MHF_Steve/300",
			skinUrl: "https://minotar.net/skin/MHF_Steve",
			avatarUrl: "https://mc-heads.net/avatar/MHF_Steve/100",
			type: "steve",
			custom: true
		}
	]);

	let autoRotate = $state(true);
	let selectedCape = $state<CapeType>(activeSkinStore.current.capeType || "none");
	let isSlimModel = $state(activeSkinStore.current.type === "alex");
	let fileInputEl: HTMLInputElement;
	let skinViewerRef: { 
		setAngle: (deg: number) => void;
		zoomIn?: () => void;
		zoomOut?: () => void;
		resetView?: () => void;
	} | null = $state(null);

	let nameMcQuery = $state("");
	let isSearchingNameMc = $state(false);

	const activeTextureUrl = $derived(
		activeSkinStore.current.skinUrl || "https://minotar.net/skin/Steve"
	);

	const filteredMarketplaceSkins = $derived(
		marketplaceSkins.filter((s) => {
			const matchesCategory = selectedMarketplaceCategory === "todos" || s.category === selectedMarketplaceCategory;
			const q = marketplaceSearch.toLowerCase().trim();
			const matchesSearch = !q || 
				s.name.toLowerCase().includes(q) || 
				s.author.toLowerCase().includes(q) || 
				s.tags.some(t => t.toLowerCase().includes(q));
			return matchesCategory && matchesSearch;
		})
	);

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

			savedSkins = [newSkin, ...savedSkins].slice(0, 25);
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

	function equipMarketplaceSkin(skin: MarketplaceSkin) {
		const skinItem: SkinItem = {
			id: skin.id,
			name: skin.name,
			url: skin.url,
			skinUrl: skin.skinUrl,
			avatarUrl: skin.avatarUrl,
			type: skin.type,
			custom: true
		};
		if (!savedSkins.some(s => s.id === skin.id)) {
			savedSkins = [skinItem, ...savedSkins].slice(0, 25);
		}
		applySkin(skinItem);
		activeTab = "wardrobe";
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

			const img = new Image();
			img.onload = () => {
				const canvas = document.createElement("canvas");
				canvas.width = 64;
				canvas.height = 64;
				const ctx = canvas.getContext("2d");
				if (ctx) {
					ctx.imageSmoothingEnabled = false;
					ctx.drawImage(img, 8, 8, 8, 8, 0, 0, 64, 64);
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

				savedSkins = [newSkin, ...savedSkins].slice(0, 25);
				applySkin(newSkin);
				toast(`Nova skin "${cleanName}" carregada e sincronizada com seu perfil!`, "success");
			};
			img.src = dataUrl;
		};
		reader.readAsDataURL(file);
		target.value = "";
	}

	let capeFileInputEl: HTMLInputElement;

	function handleCapeUpload(e: Event) {
		const target = e.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		if (!file.name.toLowerCase().endsWith(".png")) {
			toast("Selecione um arquivo de capa válido (.png)!", "error");
			return;
		}

		const reader = new FileReader();
		reader.onload = (event) => {
			const dataUrl = event.target?.result as string;
			if (!dataUrl) return;

			selectedCape = "custom";
			activeSkinStore.setCape("custom", dataUrl);
			autoRotate = false;
			skinViewerRef?.setAngle(175);
			toast("Capa personalizada importada e equipada com sucesso!", "success");
		};
		reader.readAsDataURL(file);
		target.value = "";
	}

	function setModelType(slim: boolean) {
		isSlimModel = slim;
		activeSkinStore.setSkin({ type: slim ? 'alex' : 'steve' });
	}

	function setQuickAngle(deg: number) {
		autoRotate = false;
		skinViewerRef?.setAngle(deg);
	}

	function selectCape(cape: CapeType) {
		selectedCape = cape;
		activeSkinStore.setCape(cape);
		if (cape === "none") {
			toast("Capa desequipada!", "info");
			return;
		}
		const found = capeCatalog.find(c => c.id === cape);
		const name = found ? found.name : (cape === "custom" ? "Capa Personalizada" : "Sem Capa");
		autoRotate = false;
		skinViewerRef?.setAngle(175);
		toast(`Capa "${name}" atualizada no modelo 3D!`, "success");
	}

	async function downloadSkinFile(name: string, skinUrl: string) {
		if (!skinUrl) return;
		try {
			const res = await fetch(skinUrl);
			const blob = await res.blob();
			const blobUrl = URL.createObjectURL(blob);
			const a = document.createElement("a");
			a.href = blobUrl;
			a.download = `${name.toLowerCase().replace(/[^a-z0-9]/g, "_")}.png`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			setTimeout(() => URL.revokeObjectURL(blobUrl), 10000);
			toast(`Skin "${name}" descarregada com sucesso!`, "success");
		} catch {
			const a = document.createElement("a");
			a.href = skinUrl;
			a.download = `${name.toLowerCase().replace(/[^a-z0-9]/g, "_")}.png`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			toast(`Download da skin "${name}" iniciado!`, "info");
		}
	}
</script>

<input 
	type="file" 
	accept="image/png" 
	class="hidden" 
	bind:this={fileInputEl} 
	onchange={handleFileUpload} 
/>

<input 
	type="file" 
	accept="image/png" 
	class="hidden" 
	bind:this={capeFileInputEl} 
	onchange={handleCapeUpload} 
/>

<div class="flex gap-8 h-full w-full select-none" in:fade={{ duration: 300 }}>
	<!-- Main Skins & Marketplace Area -->
	<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-6">
		
		<!-- Header with Navigation Tabs -->
		<div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mt-1 border-b border-white/5 pb-4">
			<div>
				<div class="flex items-center gap-3">
					<h1 class="text-3xl font-extrabold text-white tracking-tight">Personalização</h1>
					<span class="bg-brand-500/20 text-brand-500 text-[10px] font-black px-2.5 py-1 rounded-lg uppercase tracking-wide border border-brand-500/30 flex items-center gap-1.5 shadow-sm">
						<Box class="w-3.5 h-3.5" /> 3D Volumétrico Real
					</span>
				</div>
				<p class="text-white/50 text-xs mt-0.5">Guarda-roupa 3D, catálogo de 13 capas históricas e marketplace da comunidade Minecraft</p>
			</div>
			
			<div class="flex items-center gap-2">
				<Button 
					variant="secondary" 
					class="border-white/10 bg-[#1e1f24] hover:bg-[#282930] hover:border-white/20 text-white gap-2 rounded-xl text-xs px-4 py-2 cursor-pointer transition-colors shadow-sm"
					onclick={() => fileInputEl.click()}
				>
					<Upload class="w-3.5 h-3.5 text-amber-400" />
					Importar .PNG
				</Button>

				<Button 
					variant="secondary" 
					class="border-white/10 bg-[#1e1f24] hover:bg-[#282930] hover:border-white/20 text-white gap-2 rounded-xl text-xs px-4 py-2 cursor-pointer transition-colors shadow-sm"
					onclick={() => capeFileInputEl.click()}
				>
					<Shield class="w-3.5 h-3.5 text-brand-400" />
					Importar Capa
				</Button>

				<Button 
					variant="secondary" 
					class="border-white/10 bg-[#1e1f24] hover:bg-[#282930] hover:border-white/20 text-white gap-2 rounded-xl text-xs px-4 py-2 cursor-pointer transition-colors shadow-sm"
					onclick={() => setQuickAngle(175)}
				>
					<RotateCw class="w-3.5 h-3.5 text-blue-400" />
					Ver Costas
				</Button>

				<Button 
					variant="outline" 
					class="border-white/10 bg-[#1e1f24] hover:bg-[#282930] hover:border-white/20 text-white gap-2 rounded-xl text-xs px-4 py-2 cursor-pointer transition-colors shadow-sm" 
					onclick={() => downloadSkinFile(activeSkinStore.current.name || "skin", activeTextureUrl)}
				>
					<Download class="w-3.5 h-3.5 text-emerald-400" />
					Baixar .PNG
				</Button>
				
				<Button 
					variant="outline" 
					class="border-white/10 bg-[#1e1f24] hover:bg-[#282930] hover:border-white/20 text-white gap-2 rounded-xl text-xs px-4 py-2 cursor-pointer transition-colors shadow-sm" 
					onclick={() => toast("Personalização sincronizada com sucesso!", "success")}
				>
					<RefreshCw class="w-3.5 h-3.5" />
					Atualizar
				</Button>
			</div>
		</div>

		<!-- Main 3-Tab Navigator (Guarda-roupa / Capas HD / Marketplace) -->
		<div class="flex items-center gap-2 bg-[#121316] p-1.5 rounded-2xl border border-white/5 w-fit">
			<button 
				type="button" 
				class="flex items-center gap-2 px-5 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {activeTab === 'wardrobe' ? 'bg-[#caa97c] text-black shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
				onclick={() => activeTab = "wardrobe"}
			>
				<Shirt class="w-4 h-4" /> Guarda-roupa
			</button>
			<button 
				type="button" 
				class="flex items-center gap-2 px-5 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {activeTab === 'capes' ? 'bg-[#caa97c] text-black shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
				onclick={() => {
					activeTab = "capes";
					setQuickAngle(175);
				}}
			>
				<Shield class="w-4 h-4" /> Capas 3D ({capeCatalog.length})
			</button>
			<button 
				type="button" 
				class="flex items-center gap-2 px-5 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {activeTab === 'marketplace' ? 'bg-[#caa97c] text-black shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
				onclick={() => activeTab = "marketplace"}
			>
				<ShoppingBag class="w-4 h-4" /> Catálogo da Comunidade
			</button>
		</div>

		<!-- Tab: Wardrobe & Capes View with Split Layout -->
		<div class="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start" class:hidden={activeTab === "marketplace"}>
				
				<!-- Left Column: 3D Stage & Rig Settings (5 cols) -->
				<div class="lg:col-span-5 bg-[#18191c] border border-white/5 rounded-3xl p-5 flex flex-col items-center shadow-xl relative overflow-hidden">
					<!-- Top Rig Quick-Toggles -->
					<div class="w-full flex items-center justify-between pb-3 border-b border-white/5">
						<div class="flex items-center gap-1.5 bg-[#121316] p-1 rounded-xl border border-white/5">
							<button 
								type="button" 
								class="px-3 py-1 rounded-xl text-xs font-bold transition-all cursor-pointer {!isSlimModel ? 'bg-[#caa97c] text-black shadow-sm' : 'text-white/50 hover:text-white'}"
								onclick={() => setModelType(false)}
							>
								Classic (4px)
							</button>
							<button 
								type="button" 
								class="px-3 py-1 rounded-xl text-xs font-bold transition-all cursor-pointer {isSlimModel ? 'bg-[#caa97c] text-black shadow-sm' : 'text-white/50 hover:text-white'}"
								onclick={() => setModelType(true)}
							>
								Slim (3px)
							</button>
						</div>

						<button 
							type="button" 
							class="flex items-center gap-1.5 px-3 py-1 rounded-xl text-xs font-semibold border border-white/10 transition-colors cursor-pointer {autoRotate ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40' : 'bg-white/5 text-white/60 hover:bg-white/10'}"
							onclick={() => autoRotate = !autoRotate}
						>
							<RotateCw class="w-3 h-3 {autoRotate ? 'animate-spin' : ''}" />
							{autoRotate ? 'Giro Ativo' : 'Girar 360°'}
						</button>
					</div>

					<!-- 3D Interactive WebGL Stage (Click & Drag 360°) -->
					<div class="my-2 relative flex items-center justify-center select-none w-full h-[460px]">
						<!-- Radial Aura Backdrop -->
						<div class="absolute inset-0 bg-radial from-[#caa97c]/15 via-transparent to-transparent blur-3xl pointer-events-none"></div>

						<!-- Volumetric 3D Skin Viewer -->
						<SkinViewer3D 
							bind:this={skinViewerRef}
							skinUrl={activeTextureUrl}
							cape={selectedCape}
							customCapeUrl={activeSkinStore.current.customCapeUrl}
							slim={isSlimModel}
							{autoRotate}
							active={activeTab === 'wardrobe' || activeTab === 'capes'}
							className="z-10"
						/>

						<!-- Floating Zoom & Reset Overlay Controls -->
						<div class="absolute bottom-3 right-3 flex flex-col gap-1.5 z-20">
							<button 
								type="button" 
								class="w-8 h-8 rounded-xl bg-black/60 hover:bg-[#caa97c] hover:text-black text-white/80 border border-white/10 flex items-center justify-center text-sm font-bold transition-all shadow-lg active:scale-95 cursor-pointer"
								onclick={() => skinViewerRef?.zoomIn?.()}
								title="Aproximar (Zoom +)"
							>
								+
							</button>
							<button 
								type="button" 
								class="w-8 h-8 rounded-xl bg-black/60 hover:bg-[#caa97c] hover:text-black text-white/80 border border-white/10 flex items-center justify-center text-sm font-bold transition-all shadow-lg active:scale-95 cursor-pointer"
								onclick={() => skinViewerRef?.zoomOut?.()}
								title="Afastar (Zoom -)"
							>
								-
							</button>
							<button 
								type="button" 
								class="w-8 h-8 rounded-xl bg-black/60 hover:bg-[#caa97c] hover:text-black text-white/80 border border-white/10 flex items-center justify-center text-xs font-bold transition-all shadow-lg active:scale-95 cursor-pointer"
								onclick={() => skinViewerRef?.resetView?.()}
								title="Restaurar Visão Padrão"
							>
								⟲
							</button>
						</div>
					</div>

					<!-- Quick Angle Controls & Model Type -->
					<div class="w-full flex items-center justify-between gap-2 z-10 pt-2 border-t border-white/5">
						<div class="flex items-center gap-1.5">
							<button type="button" class="px-2.5 py-1 rounded-lg bg-[#222328] hover:bg-[#2e2f38] text-[10px] font-semibold text-white/70 hover:text-white transition-all cursor-pointer" onclick={() => setQuickAngle(0)}>Frente</button>
							<button type="button" class="px-2.5 py-1 rounded-lg bg-[#222328] hover:bg-[#2e2f38] text-[10px] font-semibold text-white/70 hover:text-white transition-all cursor-pointer" onclick={() => setQuickAngle(90)}>Lado D</button>
							<button type="button" class="px-2.5 py-1 rounded-lg bg-[#222328] hover:bg-[#2e2f38] text-[10px] font-semibold text-white/70 hover:text-white transition-all cursor-pointer" onclick={() => setQuickAngle(180)}>Costas</button>
							<button type="button" class="px-2.5 py-1 rounded-lg bg-[#222328] hover:bg-[#2e2f38] text-[10px] font-semibold text-white/70 hover:text-white transition-all cursor-pointer" onclick={() => setQuickAngle(270)}>Lado E</button>
						</div>

						<!-- Steve (4px) vs Alex (3px) Toggle -->
						<div class="flex items-center bg-[#141518] p-0.5 rounded-xl border border-white/5">
							<button 
								type="button" 
								class="px-2.5 py-1 rounded-lg text-[10px] font-semibold transition-all cursor-pointer {!isSlimModel ? 'bg-[#2e2f38] text-white font-bold shadow-sm' : 'text-white/40 hover:text-white'}"
								onclick={() => { isSlimModel = false; activeSkinStore.setSkin({ type: 'steve' }); }}
							>
								Steve (4px)
							</button>
							<button 
								type="button" 
								class="px-2.5 py-1 rounded-lg text-[10px] font-semibold transition-all cursor-pointer {isSlimModel ? 'bg-[#2e2f38] text-white font-bold shadow-sm' : 'text-white/40 hover:text-white'}"
								onclick={() => { isSlimModel = true; activeSkinStore.setSkin({ type: 'alex' }); }}
							>
								Alex (3px)
							</button>
						</div>
					</div>

					<!-- Quick Cape Strip -->
					<div class="w-full mt-3 p-2.5 bg-[#141518] rounded-2xl border border-white/5 flex items-center justify-between">
						<span class="text-[10px] font-bold text-white/50 uppercase flex items-center gap-1">
							<Shield class="w-3.5 h-3.5 text-amber-400" /> Capa Ativa
						</span>
						<div class="flex items-center gap-1.5">
							<span class="text-xs font-bold text-white capitalize bg-white/5 px-2.5 py-1 rounded-xl border border-white/5">
								{selectedCape === "none" ? "Sem capa" : selectedCape}
							</span>
							<button 
								type="button" 
								class="px-2.5 py-1 rounded-xl text-[10px] font-bold bg-[#caa97c]/20 text-brand-400 border border-brand-500/30 hover:bg-[#caa97c] hover:text-black transition-all cursor-pointer"
								onclick={() => activeTab = "capes"}
							>
								Ver Catálogo (13) →
							</button>
						</div>
					</div>

				</div>

				<!-- Right Column: Saved Skins, NameMC & Default Library -->
				<div class="lg:col-span-7 flex flex-col gap-6" class:hidden={activeTab !== "wardrobe"}>
					
					<!-- NameMC Integration Card -->
					<div class="bg-gradient-to-r from-[#18191c] via-[#1c1d22] to-[#18191c] border border-white/10 rounded-3xl p-5 shadow-lg space-y-3 relative overflow-hidden">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2.5">
								<div class="h-8 w-8 rounded-xl bg-brand-500/10 border border-brand-500/20 flex items-center justify-center text-brand-500">
									<Globe class="w-4 h-4" />
								</div>
								<div>
									<h3 class="text-xs font-black text-white flex items-center gap-2">
										Importador NameMC Skins
										<span class="bg-brand-500/20 text-brand-500 text-[9px] font-black px-2 py-0.5 rounded-full border border-brand-500/30">API ATIVA</span>
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
									placeholder="Digite o nick no NameMC (ex: Dream, Techno, Felps, Authentic)..." 
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
										<img src={skin.avatarUrl || skin.url} alt={skin.name} class="h-16 w-16 rounded-xl object-cover group-hover:scale-105 transition-transform border border-white/5 shadow-md" loading="lazy" decoding="async" />
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
										<img src={skin.avatarUrl || skin.url} alt={skin.name} class="h-16 w-16 rounded-xl object-cover group-hover:scale-105 transition-transform border border-white/5 shadow-md" loading="lazy" decoding="async" />
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

				<!-- Right Column when activeTab === "capes": Capes Catalog -->
				<div class="lg:col-span-7 flex flex-col gap-6" class:hidden={activeTab !== "capes"}>
					<!-- Capes Banner -->
					<div class="bg-gradient-to-r from-[#1b1c22] via-[#22232a] to-[#1b1c22] border border-white/10 rounded-3xl p-5 shadow-xl relative overflow-hidden flex items-center justify-between gap-4">
						<div class="space-y-1">
							<div class="flex items-center gap-2">
								<span class="px-2.5 py-0.5 rounded-full bg-brand-500/20 text-brand-400 text-[10px] font-black uppercase tracking-wider border border-brand-500/30 flex items-center gap-1">
									<Sparkles class="w-3 h-3" /> Físicas & Volumétricas
								</span>
								<span class="text-white/40 text-[11px] font-mono">13 Modelos Incluídos</span>
							</div>
							<h2 class="text-base font-extrabold text-white">Catálogo Completo de Capas 3D</h2>
							<p class="text-xs text-white/60 leading-relaxed">
								Equipe qualquer capa oficial ou importe seu próprio arquivo .PNG para visualizar em tempo real no modelo 3D.
							</p>
						</div>

						<div class="flex items-center gap-2 shrink-0">
							<button 
								type="button" 
								class="px-3.5 py-1.5 rounded-xl text-xs font-bold bg-brand-500/20 text-brand-300 border border-brand-500/30 hover:bg-brand-500 hover:text-black transition-all cursor-pointer flex items-center gap-1.5 shadow-sm"
								onclick={() => capeFileInputEl.click()}
							>
								<Upload class="w-3.5 h-3.5" />
								Importar Capa (.PNG)
							</button>
							{#if selectedCape !== "none"}
								<button 
									type="button" 
									class="px-3.5 py-1.5 rounded-xl text-xs font-bold bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500 hover:text-white transition-all cursor-pointer"
									onclick={() => selectCape("none")}
								>
									Remover Capa
								</button>
							{/if}
						</div>
					</div>

					<!-- Capes Grid -->
					<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
						{#if activeSkinStore.current.customCapeUrl}
							{@const isCustomEquipped = selectedCape === "custom"}
							<div class="bg-[#18191c] border-2 rounded-3xl p-4 flex flex-col justify-between transition-all group relative overflow-hidden {isCustomEquipped ? 'border-brand-500 bg-[#1f2026] shadow-[0_0_20px_rgba(226,184,107,0.2)]' : 'border-white/5 hover:border-white/20'}">
								<div class="flex items-center justify-between gap-2 mb-2">
									<span class="text-[10px] font-black uppercase px-2.5 py-0.5 rounded-lg border bg-amber-500/20 text-amber-300 border-amber-500/30">
										Personalizada
									</span>
									<span class="text-[11px] font-medium text-white/40">Arquivo Local</span>
								</div>

								<div class="h-24 rounded-2xl bg-gradient-to-br from-amber-500/20 to-yellow-600/10 border border-white/10 flex items-center justify-center relative overflow-hidden my-2">
									<div class="flex items-center gap-3 z-10">
										<div class="w-10 h-16 rounded-md bg-black/50 border border-white/20 flex items-center justify-center shadow-lg overflow-hidden shrink-0">
											<img src={activeSkinStore.current.customCapeUrl} alt="Capa Customizada" class="w-full h-full object-contain [image-rendering:pixelated]" />
										</div>
										<div>
											<h4 class="text-xs font-extrabold text-white">Sua Capa Personalizada</h4>
											<p class="text-[10px] text-white/50">Arquivo PNG importado</p>
										</div>
									</div>
								</div>

								<p class="text-xs text-white/60 leading-relaxed my-1.5">
									Sua textura de capa importada via arquivo do computador.
								</p>

								<div class="mt-2 pt-2.5 border-t border-white/5 flex items-center justify-between">
									{#if isCustomEquipped}
										<span class="text-xs font-bold text-emerald-400 flex items-center gap-1.5">
											<CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" /> Equipada
										</span>
										<button 
											type="button" 
											class="px-2.5 py-1 rounded-xl text-xs font-bold text-white/40 hover:text-red-400 hover:bg-red-500/10 transition-all cursor-pointer"
											onclick={() => selectCape("none")}
										>
											Desequipar
										</button>
									{:else}
										<span class="text-[10px] text-white/30 font-medium">Textura Personalizada</span>
										<button 
											type="button" 
											class="px-3.5 py-1.5 rounded-xl text-xs font-bold bg-brand-500/15 text-brand-400 border border-brand-500/30 hover:bg-brand-500 hover:text-black transition-all cursor-pointer shadow-sm"
											onclick={() => selectCape("custom")}
										>
											Equipar Capa
										</button>
									{/if}
								</div>
							</div>
						{/if}

						{#each capeCatalog as cape}
							{@const isEquipped = selectedCape === cape.id}
							<div class="bg-[#18191c] border-2 rounded-3xl p-4 flex flex-col justify-between transition-all group relative overflow-hidden {isEquipped ? 'border-brand-500 bg-[#1f2026] shadow-[0_0_20px_rgba(226,184,107,0.2)]' : 'border-white/5 hover:border-white/20'}">
								<div class="flex items-center justify-between gap-2 mb-2">
									<span class="text-[10px] font-black uppercase px-2.5 py-0.5 rounded-lg border {cape.badgeColor}">
										{cape.rarity}
									</span>
									<span class="text-[11px] font-medium text-white/40">{cape.event}</span>
								</div>

								<div class="h-24 rounded-2xl bg-gradient-to-br {cape.borderGradient} border border-white/10 flex items-center justify-center relative overflow-hidden my-2">
									<div class="flex items-center gap-3 z-10">
										<div class="w-10 h-16 rounded-md bg-black/60 border border-white/20 flex items-center justify-center shadow-lg overflow-hidden shrink-0">
											{#if getCapePreviewDataUrl(cape.id)}
												<img src={getCapePreviewDataUrl(cape.id)} alt={cape.name} class="w-full h-full object-contain [image-rendering:pixelated]" />
											{:else}
												<Shield class="w-4 h-4 text-amber-300" />
											{/if}
										</div>
										<div>
											<h4 class="text-xs font-extrabold text-white">{cape.name}</h4>
											<p class="text-[10px] text-white/50">{cape.event}</p>
										</div>
									</div>
								</div>

								<p class="text-xs text-white/60 leading-relaxed my-1.5 line-clamp-2">
									{cape.description}
								</p>

								<div class="mt-2 pt-2.5 border-t border-white/5 flex items-center justify-between">
									{#if isEquipped}
										<span class="text-xs font-bold text-emerald-400 flex items-center gap-1.5">
											<CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" /> Equipada
										</span>
										<button 
											type="button" 
											class="px-2.5 py-1 rounded-xl text-xs font-bold text-white/40 hover:text-red-400 hover:bg-red-500/10 transition-all cursor-pointer"
											onclick={() => selectCape("none")}
										>
											Desequipar
										</button>
									{:else}
										<span class="text-[10px] text-white/30 font-medium">Textura HD 3D</span>
										<button 
											type="button" 
											class="px-3.5 py-1.5 rounded-xl text-xs font-bold bg-brand-500/15 text-brand-400 border border-brand-500/30 hover:bg-brand-500 hover:text-black transition-all cursor-pointer shadow-sm"
											onclick={() => selectCape(cape.id)}
										>
											Equipar Capa
										</button>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				</div>

			</div>

		{#if activeTab === "marketplace"}
			<div class="space-y-6">
				<!-- Filter Categories & Search Bar -->
				<div class="flex flex-col md:flex-row items-stretch md:items-center justify-between gap-3 bg-[#18191c] p-4 rounded-3xl border border-white/5">
					<!-- Category Pills -->
					<div class="flex items-center gap-1.5 overflow-x-auto custom-scrollbar pb-1 md:pb-0">
						{#each [
							{ id: "todos", label: "Todas" },
							{ id: "creators", label: "Criadores" },
							{ id: "pvp", label: "PvP & Tryhard" },
							{ id: "anime", label: "Anime & Geek" },
							{ id: "aesthetic", label: "Aesthetic" },
							{ id: "medieval", label: "Medieval & RPG" }
						] as cat}
							<button 
								type="button" 
								class="px-3.5 py-1.5 rounded-xl text-xs font-bold whitespace-nowrap transition-all cursor-pointer {selectedMarketplaceCategory === cat.id ? 'bg-[#caa97c] text-black shadow-sm' : 'bg-[#202126] text-white/60 hover:text-white hover:bg-[#282930]'}"
								onclick={() => selectedMarketplaceCategory = cat.id}
							>
								{cat.label}
							</button>
						{/each}
					</div>

					<!-- Search input inside Marketplace -->
					<div class="relative min-w-[240px]">
						<Search class="w-3.5 h-3.5 text-white/40 absolute left-3.5 top-1/2 -translate-y-1/2" />
						<input 
							type="text" 
							placeholder="Buscar skin por nome, autor ou tag..." 
							bind:value={marketplaceSearch}
							class="w-full bg-[#121316] border border-white/10 rounded-full pl-9 pr-4 py-2 text-xs text-white outline-none focus:border-brand-500 transition-colors"
						/>
					</div>
				</div>

				<!-- Skins Grid -->
				<div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">
					{#each filteredMarketplaceSkins as skin}
						<div class="bg-[#18191c] border border-white/5 hover:border-brand-500/40 rounded-3xl p-4 flex flex-col justify-between transition-all group hover:shadow-xl hover:-translate-y-0.5 relative overflow-hidden">
							
							<!-- Top Badge Row -->
							<div class="flex items-center justify-between gap-2 mb-3">
								<span class="text-[10px] font-bold px-2.5 py-0.5 rounded-full bg-brand-500/10 text-brand-400 border border-brand-500/20">
									{skin.categoryLabel}
								</span>
								<div class="flex items-center gap-2 text-[11px] text-white/40 font-mono">
									<span class="flex items-center gap-1"><Download class="w-3 h-3 text-emerald-400" /> {skin.downloads}</span>
									<span class="flex items-center gap-1"><Heart class="w-3 h-3 text-rose-400" /> {skin.likes}</span>
								</div>
							</div>

							<!-- Body Preview -->
							<div class="flex items-center justify-center my-3 relative h-44">
								<div class="absolute inset-0 bg-radial from-brand-500/10 via-transparent to-transparent rounded-full blur-xl pointer-events-none group-hover:from-brand-500/20 transition-all"></div>
								<img 
									src={skin.url} 
									alt={skin.name} 
									class="h-40 object-contain z-10 drop-shadow-[0_10px_16px_rgba(0,0,0,0.6)] group-hover:scale-105 transition-transform" 
									loading="lazy"
									decoding="async"
								/>
							</div>

							<!-- Info & Action -->
							<div class="space-y-2 mt-2 pt-3 border-t border-white/5">
								<div>
									<h4 class="text-sm font-bold text-white truncate">{skin.name}</h4>
									<p class="text-[11px] text-white/40">Por <span class="text-white/70 font-medium">{skin.author}</span> • {skin.type === 'alex' ? 'Slim 3px' : 'Classic 4px'}</p>
								</div>

								<!-- Tags -->
								<div class="flex flex-wrap gap-1">
									{#each skin.tags.slice(0, 3) as tag}
										<span class="text-[9px] px-2 py-0.5 rounded-md bg-white/5 text-white/50 font-medium">#{tag}</span>
									{/each}
								</div>

								<!-- Equip & Download Buttons -->
								<div class="flex items-center gap-2 mt-2">
									<button 
										type="button" 
										class="flex-1 py-2 rounded-xl font-bold text-xs flex items-center justify-center gap-1.5 transition-all cursor-pointer bg-brand-500/15 text-brand-400 border border-brand-500/30 hover:bg-brand-500 hover:text-black shadow-sm"
										onclick={() => equipMarketplaceSkin(skin)}
									>
										<Check class="w-3.5 h-3.5" />
										Equipar
									</button>
									<button
										type="button"
										class="p-2 rounded-xl bg-white/5 hover:bg-white/10 text-white/70 hover:text-white border border-white/10 transition-colors cursor-pointer"
										onclick={() => downloadSkinFile(skin.name, skin.skinUrl)}
										title="Baixar arquivo .PNG da skin"
									>
										<Download class="w-3.5 h-3.5 text-emerald-400" />
									</button>
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}

	</div>

	<RightSidebar />
</div>
