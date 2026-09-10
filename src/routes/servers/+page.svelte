<script lang="ts">
	import { fade } from "svelte/transition";
	import { onMount, onDestroy } from "svelte";
	import { Search, RefreshCw, Star, Users, ExternalLink, Globe, ShieldCheck, ArrowUpDown, Copy, Signal } from "lucide-svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { serverPing } from "$lib/api";

	type ServerEntry = {
		id: string;
		rank: number;
		name: string;
		address: string;
		online: number;
		version: string;
		badges: string[];
		logo: string;
		bannerText: string;
		bannerColor: string;
		fav?: boolean;
	};

	let searchQuery = $state("");
	let selectedTag = $state<string | null>(null);
	let selectedRegion = $state<string>("all");
	let liveServerData = $state<Record<string, { online: number; max: number; ping: number }>>({});
	let pingInterval: ReturnType<typeof setInterval> | null = null;

	const servers: ServerEntry[] = [
		{
			id: "hypixel",
			rank: 1,
			name: "Hypixel Network",
			address: "mc.hypixel.net",
			online: 44120,
			version: "1.8.9 - 1.21.4",
			badges: ["Bedwars", "Skyblock", "Minigames", "Original"],
			logo: "https://mc-heads.net/head/MHF_Gold/100",
			bannerText: "HYPIXEL · O MAIOR SERVIDOR DO MUNDO",
			bannerColor: "from-amber-900 via-yellow-950 to-stone-900",
		},
		{
			id: "mush",
			rank: 2,
			name: "Mush MC",
			address: "mush.com.br",
			online: 8950,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "Bedwars", "PvP", "HG", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_MushroomCow/100",
			bannerText: "MUSH MC · MAIOR REDE DO BRASIL",
			bannerColor: "from-red-900 via-rose-950 to-zinc-900",
		},
		{
			id: "2b2t",
			rank: 3,
			name: "2b2t Anarchy",
			address: "2b2t.org",
			online: 1050,
			version: "1.20.4",
			badges: ["Anarchy", "Sem Regras", "Sobrevivência", "Original"],
			logo: "https://mc-heads.net/head/MHF_Obsidian/100",
			bannerText: "2B2T · THE OLDEST ANARCHY SERVER",
			bannerColor: "from-zinc-900 via-neutral-950 to-black",
		},
		{
			id: "redesky",
			rank: 4,
			name: "Rede Sky",
			address: "redesky.com",
			online: 2150,
			version: "1.8 - 1.20.4",
			badges: ["Brasil", "RankUP", "Minigames", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Lapis/100",
			bannerText: "REDE SKY · CLÁSSICO BRASIL",
			bannerColor: "from-blue-900 via-sky-950 to-slate-900",
		},
		{
			id: "donutsmp",
			rank: 5,
			name: "DonutSMP Hardcore",
			address: "donutsmp.net",
			online: 3420,
			version: "1.20.4 - 1.21.4",
			badges: ["SMP", "Hardcore", "Lifesteal", "Original"],
			logo: "https://mc-heads.net/head/MHF_TNT/100",
			bannerText: "DONUT SMP · LIFESTEAL & PVP",
			bannerColor: "from-amber-950 via-orange-900 to-stone-900",
		},
		{
			id: "complex",
			rank: 6,
			name: "Complex Gaming",
			address: "hub.mc-complex.com",
			online: 2840,
			version: "1.16 - 1.21.4",
			badges: ["Pixelmon", "Skyblock", "Survival", "Original"],
			logo: "https://mc-heads.net/head/MHF_Emerald/100",
			bannerText: "COMPLEX · PIXELMON & NETWORK",
			bannerColor: "from-emerald-950 via-teal-900 to-slate-900",
		},
		{
			id: "pika",
			rank: 7,
			name: "PikaNetwork",
			address: "pika.host",
			online: 4120,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "OP Prison", "Pirata"],
			logo: "https://mc-heads.net/head/Pikachu/100",
			bannerText: "PIKANETWORK · BEDWARS & PRISON",
			bannerColor: "from-yellow-900 via-amber-950 to-neutral-900",
		},
		{
			id: "mineberry",
			rank: 8,
			name: "MineBerry PvP",
			address: "play.mineberry.net",
			online: 1850,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "Minigames", "PvP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Cake/100",
			bannerText: "MINEBERRY · PVP & SURVIVAL",
			bannerColor: "from-rose-950 via-red-900 to-slate-900",
		},
		{
			id: "jartex",
			rank: 9,
			name: "JartexNetwork",
			address: "top.jartex.fun",
			online: 2650,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "Skyblock", "Lifesteal", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Iron/100",
			bannerText: "JARTEX · LIFESTEAL & SKYBLOCK",
			bannerColor: "from-cyan-950 via-blue-900 to-slate-900",
		},
		{
			id: "manacube",
			rank: 10,
			name: "ManaCube Network",
			address: "play.manacube.com",
			online: 2310,
			version: "1.8 - 1.21.4",
			badges: ["Parkour", "Olympus", "Survival", "Original"],
			logo: "https://mc-heads.net/head/MHF_Diamond/100",
			bannerText: "MANACUBE · PARKOUR & ISLANDS",
			bannerColor: "from-indigo-950 via-purple-900 to-slate-900",
		},
		{
			id: "cubecraft",
			rank: 11,
			name: "CubeCraft Games",
			address: "play.cubecraft.net",
			online: 3100,
			version: "1.8 - 1.21.4",
			badges: ["EggWars", "Lucky Islands", "Skyblock", "Original"],
			logo: "https://mc-heads.net/head/MHF_Chest/100",
			bannerText: "CUBECRAFT · EGGWARS CLÁSSICO",
			bannerColor: "from-sky-950 via-blue-900 to-stone-900",
		},
		{
			id: "wynncraft",
			rank: 12,
			name: "Wynncraft MMORPG",
			address: "play.wynncraft.com",
			online: 1950,
			version: "1.12 - 1.21.4",
			badges: ["MMORPG", "Quests", "Custom Mobs", "Original"],
			logo: "https://mc-heads.net/head/MHF_Enderman/100",
			bannerText: "WYNNCRAFT · O MAIOR RPG DO MINECRAFT",
			bannerColor: "from-emerald-950 via-green-900 to-zinc-900",
		},
		{
			id: "redebattle",
			rank: 13,
			name: "Rede Battle",
			address: "redebattle.com.br",
			online: 980,
			version: "1.8 - 1.20.4",
			badges: ["Brasil", "Factions", "RankUP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Sword/100",
			bannerText: "REDE BATTLE · FACTIONS & RANKUP BR",
			bannerColor: "from-orange-950 via-red-900 to-slate-900",
		},
		{
			id: "craftlandia",
			rank: 14,
			name: "Craftlandia",
			address: "craftlandia.com.br",
			online: 1450,
			version: "1.5.2 - 1.8.9",
			badges: ["Brasil", "Survival Clássico", "Nostalgia", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Herobrine/100",
			bannerText: "CRAFTLANDIA · DESDE 2011",
			bannerColor: "from-yellow-950 via-amber-900 to-black",
		},
		{
			id: "pixelmonbr",
			rank: 15,
			name: "Pixelmon Brasil",
			address: "jogar.pixelmonbrasil.com.br",
			online: 870,
			version: "1.16.5",
			badges: ["Brasil", "Pixelmon", "Modpack", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Lava/100",
			bannerText: "PIXELMON BRASIL · TORNEIOS & GINÁSIOS",
			bannerColor: "from-red-950 via-amber-950 to-stone-900",
		},
		{
			id: "twerion",
			rank: 16,
			name: "XY.TWERION.NET",
			address: "xy.twerion.net",
			online: 830,
			version: "1.8.9 - 1.21.4",
			badges: ["Bedwars", "Skyblock", "Lifesteal", "Pirata"],
			logo: "https://mc-heads.net/head/Twerion/100",
			bannerText: "TWERION · SKYBLOCK & BEDWARS",
			bannerColor: "from-purple-900 via-indigo-900 to-slate-900",
		},
		{
			id: "gommehd",
			rank: 17,
			name: "GommeHD.net",
			address: "gommehd.net",
			online: 3200,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "CityBuild", "Original"],
			logo: "https://mc-heads.net/head/MHF_Zombie/100",
			bannerText: "GOMMEHD · MAIOR REDE EUROPEIA",
			bannerColor: "from-blue-950 via-cyan-900 to-neutral-900",
		},
		{
			id: "blocksmc",
			rank: 18,
			name: "BlocksMC Network",
			address: "blocksmc.com",
			online: 2180,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "Skywars", "PvP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Blaze/100",
			bannerText: "BLOCKSMC · MINIGAMES & RANKEDS",
			bannerColor: "from-stone-900 via-orange-950 to-slate-900",
		},
		{
			id: "universocraft",
			rank: 19,
			name: "UniversoCraft",
			address: "mc.universocraft.com",
			online: 9100,
			version: "1.8 - 1.21.4",
			badges: ["Minigames", "ArenaPvP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Skeleton/100",
			bannerText: "UNIVERSOCRAFT · MAIOR REDE HISPANA",
			bannerColor: "from-violet-950 via-purple-900 to-black",
		},
		{
			id: "cosmicpvp",
			rank: 20,
			name: "CosmicPvP",
			address: "cosmicpvp.com",
			online: 640,
			version: "1.8.9",
			badges: ["Factions", "Custom Enchants", "PvP", "Original"],
			logo: "https://mc-heads.net/head/MHF_Ghast/100",
			bannerText: "COSMICPVP · HARDCORE FACTIONS",
			bannerColor: "from-purple-950 via-fuchsia-950 to-black",
		},
		{
			id: "lemoncloud",
			rank: 21,
			name: "LemonCloud",
			address: "play.lemoncloud.net",
			online: 1120,
			version: "1.12 - 1.21.4",
			badges: ["Skyblock", "Survival", "OP Prison", "Original"],
			logo: "https://mc-heads.net/head/MHF_Slime/100",
			bannerText: "LEMONCLOUD · SURVIVAL & SKYBLOCK",
			bannerColor: "from-yellow-950 via-lime-950 to-stone-900",
		},
		{
			id: "applecraft",
			rank: 22,
			name: "Applecraft",
			address: "play.applecraft.org",
			online: 890,
			version: "1.20.4 - 1.21.4",
			badges: ["Survival", "GriefPrevention", "Original"],
			logo: "https://mc-heads.net/head/MHF_Apple/100",
			bannerText: "APPLECRAFT · SEM GRIEF & ECONOMIA",
			bannerColor: "from-red-950 via-rose-950 to-slate-900",
		},
		{
			id: "herobrine",
			rank: 23,
			name: "Herobrine.org",
			address: "herobrine.org",
			online: 1400,
			version: "1.8 - 1.21.4",
			badges: ["Survival", "Bedwars", "Earth", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Herobrine/100",
			bannerText: "HEROBRINE · SURVIVAL & EARTH",
			bannerColor: "from-zinc-950 via-red-950 to-black",
		},
		{
			id: "extremecraft",
			rank: 24,
			name: "ExtremeCraft Network",
			address: "play.extremecraft.net",
			online: 980,
			version: "1.8 - 1.21.4",
			badges: ["Towny", "Survival", "SkyGrid", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Creeper/100",
			bannerText: "EXTREMECRAFT · TOWNY & SKYGRID",
			bannerColor: "from-green-950 via-emerald-950 to-black",
		},
		{
			id: "opblocks",
			rank: 25,
			name: "OPBlocks Network",
			address: "hub.opblocks.com",
			online: 1350,
			version: "1.8 - 1.21.4",
			badges: ["Candy Prison", "Pixelmon", "Original"],
			logo: "https://mc-heads.net/head/MHF_PigZombie/100",
			bannerText: "OPBLOCKS · CANDY PRISON & SKYBLOCK",
			bannerColor: "from-pink-950 via-fuchsia-950 to-stone-900",
		},
		{
			id: "stardix",
			rank: 26,
			name: "StarDix Brasil",
			address: "jogar.stardix.com",
			online: 3100,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "FullPvP", "RankUP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Star/100",
			bannerText: "STARDIX · FULLPVP & RANKUP BRASIL",
			bannerColor: "from-amber-950 via-yellow-900 to-black",
		},
		{
			id: "landix",
			rank: 27,
			name: "Rede Landix",
			address: "jogar.landix.com.br",
			online: 850,
			version: "1.8 - 1.20.4",
			badges: ["Brasil", "RankUP", "Economia", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_IronGolem/100",
			bannerText: "LANDIX · ECONOMIA REAL & RANKUP",
			bannerColor: "from-blue-950 via-indigo-950 to-zinc-900",
		},
		{
			id: "rederevo",
			rank: 28,
			name: "Rede Revo",
			address: "jogar.rederevo.com",
			online: 1240,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "Bedwars", "RankUP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Blaze/100",
			bannerText: "REDE REVO · BEDWARS COMPETITIVO",
			bannerColor: "from-red-950 via-orange-950 to-black",
		},
		{
			id: "skycraft",
			rank: 29,
			name: "SkyCraft Brasil",
			address: "skycraft.com.br",
			online: 720,
			version: "1.8 - 1.20.4",
			badges: ["Brasil", "Minigames", "Skyblock", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Diamond/100",
			bannerText: "SKYCRAFT · CLÁSSICO BRASILEIRO",
			bannerColor: "from-cyan-950 via-blue-950 to-stone-900",
		},
		{
			id: "royalemc",
			rank: 30,
			name: "RoyaleMC",
			address: "jogar.royalemc.com.br",
			online: 640,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "Factions", "PvP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Crown/100",
			bannerText: "ROYALEMC · FACTIONS & COMBATE",
			bannerColor: "from-purple-950 via-violet-950 to-black",
		},
		{
			id: "craftsapim",
			rank: 31,
			name: "CraftSapim Survival",
			address: "jogar.craftsapim.com.br",
			online: 510,
			version: "1.20.4 - 1.21.4",
			badges: ["Brasil", "Survival", "Slimefun", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Slime/100",
			bannerText: "CRAFTSAPIM · SURVIVAL SLIMEFUN",
			bannerColor: "from-emerald-950 via-green-950 to-zinc-900",
		},
		{
			id: "minemen",
			rank: 32,
			name: "Minemen Club",
			address: "minemen.club",
			online: 2650,
			version: "1.8.9",
			badges: ["PvP", "Treino", "PotPvP", "Original"],
			logo: "https://mc-heads.net/head/MHF_Sword/100",
			bannerText: "MINEMEN CLUB · PRACTICE & DUELS",
			bannerColor: "from-slate-900 via-zinc-900 to-black",
		},
		{
			id: "purpleprison",
			rank: 33,
			name: "Purple Prison",
			address: "purpleprison.net",
			online: 2100,
			version: "1.8 - 1.21.4",
			badges: ["OP Prison", "Parkour", "Original"],
			logo: "https://mc-heads.net/head/MHF_Enderman/100",
			bannerText: "PURPLE PRISON · O MAIOR PRISON DO MUNDO",
			bannerColor: "from-purple-950 via-fuchsia-950 to-neutral-950",
		},
		{
			id: "netherite",
			rank: 34,
			name: "NetheriteMC",
			address: "play.netheritemc.com",
			online: 1450,
			version: "1.20.4 - 1.21.4",
			badges: ["Lifesteal", "BoxPvP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Netherite/100",
			bannerText: "NETHERITEMC · LIFESTEAL & BOXPVP",
			bannerColor: "from-zinc-950 via-stone-900 to-black",
		},
		{
			id: "holyhcf",
			rank: 35,
			name: "HolyHCF Hardcore",
			address: "holyhcf.net",
			online: 920,
			version: "1.8.9",
			badges: ["HCF", "Hardcore", "Factions", "Original"],
			logo: "https://mc-heads.net/head/MHF_Redstone/100",
			bannerText: "HOLYHCF · HARDCORE FACTIONS CLÁSSICO",
			bannerColor: "from-red-950 via-rose-950 to-black",
		},
		{
			id: "vipermc",
			rank: 36,
			name: "ViperMC Network",
			address: "vipermc.net",
			online: 1100,
			version: "1.8.9",
			badges: ["HCF", "PvP", "Original"],
			logo: "https://mc-heads.net/head/MHF_Spider/100",
			bannerText: "VIPERMC · A LENDA DO HCF",
			bannerColor: "from-green-950 via-teal-950 to-black",
		},
		{
			id: "stray",
			rank: 37,
			name: "Stray Practice",
			address: "stray.gg",
			online: 840,
			version: "1.8.9 - 1.21.4",
			badges: ["PvP", "Duels", "Ranked", "Original"],
			logo: "https://mc-heads.net/head/MHF_Bow/100",
			bannerText: "STRAY · MODERN PRACTICE & DUELS",
			bannerColor: "from-sky-950 via-cyan-950 to-slate-900",
		},
		{
			id: "lunar",
			rank: 38,
			name: "Lunar Practice",
			address: "lunar.gg",
			online: 1350,
			version: "1.8.9",
			badges: ["PvP", "Bedwars", "Original"],
			logo: "https://mc-heads.net/head/MHF_Moon/100",
			bannerText: "LUNAR · DUELS & BEDWARS",
			bannerColor: "from-indigo-950 via-blue-950 to-black",
		},
		{
			id: "moxmc",
			rank: 39,
			name: "Mox MC",
			address: "moxmc.net",
			online: 1200,
			version: "1.20.4 - 1.21.4",
			badges: ["Towny", "Survival", "Original"],
			logo: "https://mc-heads.net/head/MHF_OakLog/100",
			bannerText: "MOX MC · TOWNY & SURVIVAL SMP",
			bannerColor: "from-amber-950 via-stone-900 to-black",
		},
		{
			id: "tubnet",
			rank: 40,
			name: "TubNet",
			address: "tubnet.gg",
			online: 950,
			version: "1.20.4",
			badges: ["Minigames", "Crossplay", "Original"],
			logo: "https://mc-heads.net/head/MHF_Cake/100",
			bannerText: "TUBNET · CROSSPLAY BEDROCK & JAVA",
			bannerColor: "from-orange-950 via-red-950 to-zinc-900",
		},
		{
			id: "insanitycraft",
			rank: 41,
			name: "InsanityCraft",
			address: "play.insanitycraft.net",
			online: 1100,
			version: "1.12 - 1.21.4",
			badges: ["Factions", "Survival", "Original"],
			logo: "https://mc-heads.net/head/MHF_Ghast/100",
			bannerText: "INSANITYCRAFT · FACTIONS & SURVIVAL",
			bannerColor: "from-neutral-950 via-red-950 to-black",
		},
		{
			id: "vulcan",
			rank: 42,
			name: "VulcanMC",
			address: "play.vulcanmc.net",
			online: 780,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "Practice", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Lava/100",
			bannerText: "VULCANMC · BEDWARS & ARENAS",
			bannerColor: "from-red-950 via-amber-950 to-stone-900",
		},
		{
			id: "vortex",
			rank: 43,
			name: "Vortex Network",
			address: "mc.vortexnetwork.net",
			online: 1600,
			version: "1.12 - 1.21.4",
			badges: ["Space Prison", "Skyblock", "Original"],
			logo: "https://mc-heads.net/head/MHF_Beacon/100",
			bannerText: "VORTEX · SPACE PRISON & SKYBLOCK",
			bannerColor: "from-blue-950 via-purple-950 to-black",
		},
		{
			id: "fadecloud",
			rank: 44,
			name: "FadeCloud",
			address: "fadecloud.com",
			online: 1300,
			version: "1.12 - 1.21.4",
			badges: ["OP Prison", "Skyblock", "Original"],
			logo: "https://mc-heads.net/head/MHF_Cloud/100",
			bannerText: "FADECLOUD · SKYBLOCK & PRISON",
			bannerColor: "from-sky-950 via-indigo-950 to-slate-900",
		},
		{
			id: "wildercraft",
			rank: 45,
			name: "WilderCraft Survival",
			address: "play.wildercraft.net",
			online: 650,
			version: "1.21.4",
			badges: ["Survival", "Vanilla+", "Original"],
			logo: "https://mc-heads.net/head/MHF_Sunflower/100",
			bannerText: "WILDERCRAFT · SURVIVAL PURO & COMUNIDADE",
			bannerColor: "from-green-950 via-emerald-950 to-black",
		},
		{
			id: "manacube",
			rank: 46,
			name: "ManaCube Network",
			address: "play.manacube.com",
			online: 2100,
			version: "1.8 - 1.21.4",
			badges: ["Parkour", "Survival", "Skyblock", "Original"],
			logo: "https://mc-heads.net/head/MHF_Potion/100",
			bannerText: "MANACUBE · PARKOUR & SURVIVAL",
			bannerColor: "from-cyan-950 via-blue-950 to-slate-900",
		},
		{
			id: "cubecraft",
			rank: 47,
			name: "CubeCraft Games",
			address: "play.cubecraft.net",
			online: 3200,
			version: "1.12 - 1.21.4",
			badges: ["EggWars", "SkyWars", "Minigames", "Original"],
			logo: "https://mc-heads.net/head/MHF_Cactus/100",
			bannerText: "CUBECRAFT · EGGWARS & SKYWARS",
			bannerColor: "from-sky-950 via-teal-950 to-stone-900",
		},
		{
			id: "gommehd",
			rank: 48,
			name: "GommeHD Network",
			address: "gommehd.net",
			online: 2600,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "TTT", "Europa", "Original"],
			logo: "https://mc-heads.net/head/MHF_PigZombie/100",
			bannerText: "GOMMEHD · O MAIOR SERVIDOR EUROPEU",
			bannerColor: "from-amber-950 via-orange-950 to-neutral-900",
		},
		{
			id: "jartex",
			rank: 49,
			name: "JartexNetwork",
			address: "play.jartexnetwork.com",
			online: 3900,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "Skyblock", "Practice", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Spider/100",
			bannerText: "JARTEX · BEDWARS & PRISON PIRATA",
			bannerColor: "from-purple-950 via-violet-950 to-zinc-900",
		},
		{
			id: "blocksmc",
			rank: 50,
			name: "BlocksMC Network",
			address: "blocksmc.com",
			online: 4800,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "PvP", "Practice", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_IronGolem/100",
			bannerText: "BLOCKSMC · MAIOR REDE PIRATA GLOBAL",
			bannerColor: "from-rose-950 via-red-950 to-slate-900",
		},
		{
			id: "cosmicpvp",
			rank: 51,
			name: "CosmicPvP",
			address: "play.cosmicpvp.com",
			online: 1150,
			version: "1.8.9",
			badges: ["Factions", "PvP", "Hardcore", "Original"],
			logo: "https://mc-heads.net/head/MHF_Enderman/100",
			bannerText: "COSMICPVP · HARDCORE FACTIONS CLÁSSICO",
			bannerColor: "from-fuchsia-950 via-purple-950 to-black",
		},
		{
			id: "nethergames",
			rank: 52,
			name: "NetherGames",
			address: "play.nethergames.org",
			online: 1750,
			version: "1.20 - 1.21.4",
			badges: ["Bedwars", "Duels", "Minigames", "Original"],
			logo: "https://mc-heads.net/head/MHF_Netherrack/100",
			bannerText: "NETHERGAMES · BEDWARS & ARENAS",
			bannerColor: "from-red-950 via-stone-900 to-black",
		},
		{
			id: "lemoncloud",
			rank: 53,
			name: "LemonCloud",
			address: "play.lemoncloud.org",
			online: 1420,
			version: "1.12 - 1.21.4",
			badges: ["OP Prison", "Survival", "Skyblock", "Original"],
			logo: "https://mc-heads.net/head/MHF_Sunflower/100",
			bannerText: "LEMONCLOUD · OP PRISON & SURVIVAL",
			bannerColor: "from-yellow-950 via-amber-900 to-stone-900",
		},
		{
			id: "minehut",
			rank: 54,
			name: "Minehut Hub",
			address: "minehut.com",
			online: 5400,
			version: "1.21.4",
			badges: ["Custom Servers", "Sandbox", "Minigames", "Original"],
			logo: "https://mc-heads.net/head/MHF_Chest/100",
			bannerText: "MINEHUT · CRIE E JOGUE SERVIDORES LIVRES",
			bannerColor: "from-blue-950 via-indigo-950 to-neutral-900",
		},
		{
			id: "wynncraft",
			rank: 55,
			name: "Wynncraft MMORPG",
			address: "play.wynncraft.com",
			online: 2950,
			version: "1.20.2 - 1.21.4",
			badges: ["MMORPG", "Quests", "Dungeons", "Original"],
			logo: "https://mc-heads.net/head/MHF_Gold/100",
			bannerText: "WYNNCRAFT · O MAIOR MMORPG DO MINECRAFT",
			bannerColor: "from-amber-950 via-emerald-950 to-stone-950",
		},
		{
			id: "earthmc",
			rank: 56,
			name: "EarthMC",
			address: "play.earthmc.net",
			online: 920,
			version: "1.20.4",
			badges: ["Towny", "Geopolítica", "Survival", "Original"],
			logo: "https://mc-heads.net/head/MHF_Grass/100",
			bannerText: "EARTHMC · MAPA REAL DA TERRA EM ESCALA",
			bannerColor: "from-blue-950 via-green-950 to-zinc-900",
		},
		{
			id: "originrealms",
			rank: 57,
			name: "Origin Realms",
			address: "play.originrealms.com",
			online: 1100,
			version: "1.20.4 - 1.21.4",
			badges: ["Vanilla+", "RPG", "Texturas Custom", "Original"],
			logo: "https://mc-heads.net/head/MHF_Oak/100",
			bannerText: "ORIGIN REALMS · VANILLA EXPANDIDO",
			bannerColor: "from-emerald-950 via-lime-950 to-neutral-900",
		},
		{
			id: "craftlandia",
			rank: 58,
			name: "Craftlandia Clássico",
			address: "jogar.craftlandia.com.br",
			online: 1650,
			version: "1.5.2 - 1.8.9",
			badges: ["Brasil", "Survival", "Clássico", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Diamond/100",
			bannerText: "CRAFTLANDIA · PIONEIRO NO BRASIL",
			bannerColor: "from-cyan-950 via-blue-950 to-black",
		},
		{
			id: "rederevo",
			rank: 59,
			name: "Rede Revo",
			address: "jogar.rederevo.com.br",
			online: 850,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "RankUP", "PvP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Redstone/100",
			bannerText: "REDE REVO · RANKUP & ECONOMIA",
			bannerColor: "from-rose-950 via-red-950 to-neutral-900",
		},
		{
			id: "starmade",
			rank: 60,
			name: "StarMade Brasil",
			address: "mc.starmade.com.br",
			online: 620,
			version: "1.20.4 - 1.21.4",
			badges: ["Brasil", "Survival", "Economia", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Star/100",
			bannerText: "STARMADE · SURVIVAL COM ECONOMIA REAL",
			bannerColor: "from-indigo-950 via-purple-950 to-slate-900",
		},
		{
			id: "pixelmonrealms",
			rank: 61,
			name: "Pixelmon Realms",
			address: "play.pixelmonrealms.com",
			online: 890,
			version: "1.16.5",
			badges: ["Pixelmon", "Pokecenter", "Torneios", "Original"],
			logo: "https://mc-heads.net/head/MHF_Pokeball/100",
			bannerText: "PIXELMON REALMS · JORNADA POKÉMON",
			bannerColor: "from-red-950 via-slate-900 to-black",
		},
		{
			id: "mcprison",
			rank: 62,
			name: "MC-Prison",
			address: "mc-prison.com",
			online: 740,
			version: "1.12 - 1.21.4",
			badges: ["OP Prison", "Mineração", "Economia", "Original"],
			logo: "https://mc-heads.net/head/MHF_Iron/100",
			bannerText: "MC-PRISON · PRISÃO CLÁSSICA & MINERAÇÃO",
			bannerColor: "from-zinc-950 via-stone-900 to-black",
		},
		{
			id: "applemc",
			rank: 63,
			name: "AppleMC",
			address: "play.applemc.fun",
			online: 1850,
			version: "1.19 - 1.21.4",
			badges: ["SMP", "Lifesteal", "Survival", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Apple/100",
			bannerText: "APPLEMC · LIFESTEAL & SMP COMPETITIVO",
			bannerColor: "from-red-950 via-rose-900 to-zinc-950",
		},
		{
			id: "craftsapling",
			rank: 64,
			name: "CraftSapling",
			address: "play.craftsapling.com",
			online: 510,
			version: "1.21.4",
			badges: ["Skyblock", "Vanilla+", "Original"],
			logo: "https://mc-heads.net/head/MHF_Sapling/100",
			bannerText: "CRAFTSAPLING · SKYBLOCK VANILLA",
			bannerColor: "from-emerald-950 via-green-900 to-neutral-900",
		},
		{
			id: "blossomcraft",
			rank: 65,
			name: "BlossomCraft",
			address: "play.blossomcraft.org",
			online: 780,
			version: "1.20 - 1.21.4",
			badges: ["Survival", "SMP", "Construção", "Original"],
			logo: "https://mc-heads.net/head/MHF_Flower/100",
			bannerText: "BLOSSOMCRAFT · SMP COMUNIDADE & CONSTRUÇÃO",
			bannerColor: "from-pink-950 via-rose-950 to-slate-900",
		},
		{
			id: "moxmc",
			rank: 66,
			name: "MoxMC Network",
			address: "moxmc.net",
			online: 1400,
			version: "1.8 - 1.21.4",
			badges: ["KitPvP", "OP Prison", "Original"],
			logo: "https://mc-heads.net/head/MHF_Sword/100",
			bannerText: "MOXMC · KITPVP & ARENAS",
			bannerColor: "from-cyan-950 via-teal-950 to-black",
		},
		{
			id: "minewind",
			rank: 67,
			name: "Minewind Anarchy",
			address: "play.minewind.com",
			online: 640,
			version: "1.20 - 1.21.4",
			badges: ["Anarchy", "PvP", "Sobrevivência", "Original"],
			logo: "https://mc-heads.net/head/MHF_Blaze/100",
			bannerText: "MINEWIND · ANARQUIA CONTROLADA",
			bannerColor: "from-purple-950 via-zinc-950 to-black",
		},
		{
			id: "herobrine",
			rank: 68,
			name: "Herobrine.org",
			address: "herobrine.org",
			online: 2300,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "Survival", "Skyblock", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Herobrine/100",
			bannerText: "HEROBRINE · REDE MULTIJOGOS PIRATA",
			bannerColor: "from-blue-950 via-slate-900 to-black",
		},
		{
			id: "extremecraft",
			rank: 69,
			name: "ExtremeCraft",
			address: "play.extremecraft.net",
			online: 1120,
			version: "1.8 - 1.21.4",
			badges: ["Survival", "Factions", "Skygrid", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Creeper/100",
			bannerText: "EXTREMECRAFT · SURVIVAL & FACTIONS",
			bannerColor: "from-emerald-950 via-stone-900 to-black",
		},
		{
			id: "hypemc",
			rank: 70,
			name: "HypeMC Brasil",
			address: "jogar.hypemc.com.br",
			online: 790,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "Bedwars", "PvP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Gold/100",
			bannerText: "HYPEMC · O MELHOR BEDWARS BRASILEIRO",
			bannerColor: "from-amber-950 via-orange-950 to-black",
		},
		{
			id: "lemoncloud",
			rank: 71,
			name: "LemonCloud",
			address: "play.lemoncloud.org",
			online: 1140,
			version: "1.12 - 1.21.4",
			badges: ["Skyblock", "OP Prison", "Survival"],
			logo: "https://mc-heads.net/head/MHF_Yellow/100",
			bannerText: "LEMONCLOUD · SKYBLOCK & PRISON",
			bannerColor: "from-yellow-950 via-amber-950 to-black",
		},
		{
			id: "primemc",
			rank: 72,
			name: "PrimeMC",
			address: "play.primemc.net",
			online: 820,
			version: "1.8 - 1.21.4",
			badges: ["Prison", "Skyblock", "Original"],
			logo: "https://mc-heads.net/head/MHF_Iron/100",
			bannerText: "PRIMEMC · CLASSIC PRISON & ISLANDS",
			bannerColor: "from-blue-950 via-slate-900 to-black",
		},
		{
			id: "snapcraft",
			rank: 73,
			name: "Snapcraft Network",
			address: "mc.snapcraft.net",
			online: 940,
			version: "1.8 - 1.21.4",
			badges: ["Survival", "Zombie", "Creative"],
			logo: "https://mc-heads.net/head/MHF_Enderman/100",
			bannerText: "SNAPCRAFT · ZOMBIE & SURVIVAL",
			bannerColor: "from-purple-950 via-zinc-900 to-black",
		},
		{
			id: "craftlandia",
			rank: 74,
			name: "CraftLandia Legacy",
			address: "legacy.craftlandia.com.br",
			online: 1350,
			version: "1.5.2 - 1.8",
			badges: ["Brasil", "Survival", "Economia", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_OakLog/100",
			bannerText: "CRAFTLANDIA · O PRIMEIRO DO BRASIL",
			bannerColor: "from-emerald-950 via-green-950 to-black",
		},
		{
			id: "redelike",
			rank: 75,
			name: "Rede Like Brasil",
			address: "redelike.com.br",
			online: 620,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "Bedwars", "PvP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Diamond/100",
			bannerText: "REDE LIKE · BEDWARS & ARENA",
			bannerColor: "from-cyan-950 via-blue-950 to-black",
		},
		{
			id: "rededark",
			rank: 76,
			name: "Rede Dark",
			address: "jogar.rededark.com",
			online: 580,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "RankUP", "Factions", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Obsidian/100",
			bannerText: "REDE DARK · RANKUP E MAQUINAS",
			bannerColor: "from-violet-950 via-zinc-950 to-black",
		},
		{
			id: "veltpvp",
			rank: 77,
			name: "Velt PvP",
			address: "play.veltpvp.com",
			online: 1420,
			version: "1.7 - 1.8",
			badges: ["HCF", "Practice", "PvP", "Hardcore"],
			logo: "https://mc-heads.net/head/MHF_Redstone/100",
			bannerText: "VELT PVP · COMPETITIVE HARDCORE",
			bannerColor: "from-red-950 via-stone-900 to-black",
		},
		{
			id: "vipermc",
			rank: 78,
			name: "Viper MC",
			address: "play.vipermc.net",
			online: 1100,
			version: "1.7 - 1.8",
			badges: ["HCF", "Practice", "Hardcore"],
			logo: "https://mc-heads.net/head/MHF_Spider/100",
			bannerText: "VIPERMC · HCF LEADERBOARDS",
			bannerColor: "from-green-950 via-emerald-950 to-black",
		},
		{
			id: "mineheroes",
			rank: 79,
			name: "MineHeroes",
			address: "play.mineheroes.net",
			online: 730,
			version: "1.8 - 1.21.4",
			badges: ["Factions", "Skyblock", "Prison"],
			logo: "https://mc-heads.net/head/MHF_Gold/100",
			bannerText: "MINEHEROES · CUSTOM ENCHANTS",
			bannerColor: "from-amber-950 via-stone-900 to-black",
		},
		{
			id: "purpurmc",
			rank: 80,
			name: "Purpur SMP",
			address: "purpurmc.org",
			online: 410,
			version: "1.20 - 1.21.4",
			badges: ["Vanilla", "SMP", "Comunidade"],
			logo: "https://mc-heads.net/head/MHF_Purpur/100",
			bannerText: "PURPUR · HIGH PERFORMANCE VANILLA",
			bannerColor: "from-fuchsia-950 via-purple-950 to-black",
		},
		{
			id: "pokefind",
			rank: 81,
			name: "PokeFind Vanilla Pokemon",
			address: "play.pokefind.co",
			online: 690,
			version: "1.12 - 1.21.4",
			badges: ["Pokemon", "MMO", "Sem Mods"],
			logo: "https://mc-heads.net/head/MHF_Apple/100",
			bannerText: "POKEFIND · POKEMON SEM PRECISAR DE MODS",
			bannerColor: "from-red-950 via-rose-950 to-black",
		},
		{
			id: "universemc",
			rank: 82,
			name: "UniverseMC",
			address: "play.universemc.us",
			online: 880,
			version: "1.8 - 1.21.4",
			badges: ["Practice", "KitPvP", "Duels"],
			logo: "https://mc-heads.net/head/MHF_Lapis/100",
			bannerText: "UNIVERSEMC · 1V1 DUELS & RANKED",
			bannerColor: "from-blue-950 via-indigo-950 to-black",
		},
		{
			id: "vulcannetwork",
			rank: 83,
			name: "Vulcan Network",
			address: "play.vulcanmc.net",
			online: 540,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "Duels", "Ranked"],
			logo: "https://mc-heads.net/head/MHF_Lava/100",
			bannerText: "VULCAN NETWORK · FAST PACED BEDWARS",
			bannerColor: "from-orange-950 via-red-950 to-black",
		},
		{
			id: "fadecloud",
			rank: 84,
			name: "FadeCloud",
			address: "fadecloud.com",
			online: 910,
			version: "1.8 - 1.21.4",
			badges: ["Prison", "Skyblock", "Custom"],
			logo: "https://mc-heads.net/head/MHF_Ghast/100",
			bannerText: "FADECLOUD · OP PRISON EXPERIENCE",
			bannerColor: "from-sky-950 via-slate-900 to-black",
		},
		{
			id: "skyblocknet",
			rank: 85,
			name: "Original Skyblock",
			address: "skyblock.net",
			online: 480,
			version: "1.8 - 1.21.4",
			badges: ["Skyblock", "Original", "Vanilla"],
			logo: "https://mc-heads.net/head/MHF_Grass/100",
			bannerText: "SKYBLOCK.NET · O PIONEIRO DO SKYBLOCK",
			bannerColor: "from-emerald-950 via-stone-900 to-black",
		},
		{
			id: "moxmc",
			rank: 86,
			name: "Mox MC",
			address: "moxmc.net",
			online: 1250,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "Prison", "Parkour"],
			logo: "https://mc-heads.net/head/MHF_TNT/100",
			bannerText: "MOX MC · BEDWARS & PARKOUR ESCAPE",
			bannerColor: "from-rose-950 via-purple-950 to-black",
		},
		{
			id: "minemakers",
			rank: 87,
			name: "MineMakers",
			address: "play.minemakers.net",
			online: 670,
			version: "1.16 - 1.21.4",
			badges: ["Minigames", "France", "Original"],
			logo: "https://mc-heads.net/head/MHF_Chest/100",
			bannerText: "MINEMAKERS · INNOVATIVE MINIGAMES",
			bannerColor: "from-blue-950 via-zinc-900 to-black",
		},
		{
			id: "gommehd",
			rank: 88,
			name: "GommeHD.net",
			address: "gommehd.net",
			online: 4200,
			version: "1.8 - 1.21.4",
			badges: ["Bedwars", "TTT", "CityBuild", "Europa"],
			logo: "https://mc-heads.net/head/MHF_Pig/100",
			bannerText: "GOMMEHD · MAIOR REDE DA EUROPA",
			bannerColor: "from-blue-950 via-cyan-950 to-black",
		},
		{
			id: "batihost",
			rank: 89,
			name: "Batihost Network",
			address: "play.batihost.com",
			online: 730,
			version: "1.8 - 1.21.4",
			badges: ["Survival", "Factions", "Turkey"],
			logo: "https://mc-heads.net/head/MHF_Iron/100",
			bannerText: "BATIHOST · SURVIVAL TOP RANK",
			bannerColor: "from-red-950 via-zinc-950 to-black",
		},
		{
			id: "redesky",
			rank: 90,
			name: "Rede Sky Brasil",
			address: "jogar.redesky.com",
			online: 810,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "Survival", "RankUP", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Feather/100",
			bannerText: "REDE SKY · SURVIVAL EVOLUÍDO",
			bannerColor: "from-cyan-950 via-teal-950 to-black",
		},
		{
			id: "landstown",
			rank: 91,
			name: "Landstown RPG",
			address: "jogar.landstown.com.br",
			online: 390,
			version: "1.16 - 1.21.4",
			badges: ["Brasil", "Survival RPG", "Cidades", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_OakLog/100",
			bannerText: "LANDSTOWN · CIDADES MEDIEVAIS & RPG",
			bannerColor: "from-amber-950 via-yellow-950 to-black",
		},
		{
			id: "rederevo",
			rank: 92,
			name: "Rede Revo",
			address: "jogar.rederevo.com",
			online: 440,
			version: "1.8 - 1.21.4",
			badges: ["Brasil", "FullPvP", "Factions", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Sword/100",
			bannerText: "REDE REVO · FULLPVP CLÁSSICO",
			bannerColor: "from-stone-900 via-zinc-900 to-black",
		},
		{
			id: "originrealms",
			rank: 93,
			name: "Origin Realms",
			address: "play.originrealms.com",
			online: 1650,
			version: "1.20 - 1.21.4",
			badges: ["MMORPG", "Custom Blocks", "ResourcePack"],
			logo: "https://mc-heads.net/head/MHF_Bookshelf/100",
			bannerText: "ORIGIN REALMS · THE JAVA VANILLA MMO",
			bannerColor: "from-emerald-950 via-teal-950 to-black",
		},
		{
			id: "westeroscraft",
			rank: 94,
			name: "WesterosCraft",
			address: "mc.westeroscraft.com",
			online: 320,
			version: "1.20.1",
			badges: ["RPG", "Construção", "Westeros"],
			logo: "https://mc-heads.net/head/MHF_Castle/100",
			bannerText: "WESTEROSCRAFT · GAME OF THRONES EM ESCALA",
			bannerColor: "from-red-950 via-stone-900 to-black",
		},
		{
			id: "massivecraft",
			rank: 95,
			name: "MassiveCraft",
			address: "massivecraft.com",
			online: 510,
			version: "1.16 - 1.21.4",
			badges: ["Factions", "Roleplay", "Original"],
			logo: "https://mc-heads.net/head/MHF_Crown/100",
			bannerText: "MASSIVECRAFT · OS INVENTORES DE FACTIONS",
			bannerColor: "from-purple-950 via-zinc-900 to-black",
		},
		{
			id: "hivelegacy",
			rank: 96,
			name: "The Hive Java Archive",
			address: "play.hivemc.com",
			online: 890,
			version: "1.8 - 1.20.4",
			badges: ["Minigames", "HideAndSeek", "Comunidade"],
			logo: "https://mc-heads.net/head/MHF_Beehive/100",
			bannerText: "THE HIVE · CLÁSSICOS MINIGAMES",
			bannerColor: "from-amber-950 via-yellow-950 to-black",
		},
		{
			id: "piratecraft",
			rank: 97,
			name: "PirateCraft Ships",
			address: "mc.piratemc.com",
			online: 360,
			version: "1.16 - 1.21.4",
			badges: ["Piratas", "Navios", "Canhões", "Survival"],
			logo: "https://mc-heads.net/head/MHF_Skeleton/100",
			bannerText: "PIRATECRAFT · BATALHAS NAVAIS E MARINHA",
			bannerColor: "from-cyan-950 via-blue-950 to-black",
		},
		{
			id: "pixelmonbrasil",
			rank: 98,
			name: "Pixelmon Brasil",
			address: "jogar.pixelmonbrasil.com.br",
			online: 1120,
			version: "1.16.5",
			badges: ["Brasil", "Pixelmon", "Pokemon", "Pirata"],
			logo: "https://mc-heads.net/head/MHF_Pickachu/100",
			bannerText: "PIXELMON BRASIL · TORNEIOS E GINÁSIOS",
			bannerColor: "from-red-950 via-yellow-950 to-black",
		},
		{
			id: "earthmc",
			rank: 99,
			name: "EarthMC Geopolitics",
			address: "play.earthmc.net",
			online: 1980,
			version: "1.20 - 1.21.4",
			badges: ["Geopolítico", "Mapa da Terra", "Towny"],
			logo: "https://mc-heads.net/head/MHF_Earth/100",
			bannerText: "EARTHMC · O MAPA REAL DA TERRA EM 1:500",
			bannerColor: "from-blue-950 via-emerald-950 to-black",
		},
		{
			id: "luxmchub",
			rank: 100,
			name: "Luxmc Community Hub",
			address: "hub.luxmc.io",
			online: 2450,
			version: "1.21.4",
			badges: ["Comunidade Oficial", "Linux", "Vulkan", "Zero Ping"],
			logo: "https://mc-heads.net/head/MHF_Gold/100",
			bannerText: "LUXMC HUB · SERVIDOR OFICIAL DOS JOGADORES LINUX",
			bannerColor: "from-brand-500/30 via-stone-900 to-black",
		}
	];

	const tags = $derived.by(() => {
		const tagMap: Record<string, number> = {};
		for (const srv of servers) {
			for (const b of srv.badges) {
				tagMap[b] = (tagMap[b] || 0) + 1;
			}
		}
		return Object.entries(tagMap)
			.map(([name, count]) => ({ name, count }))
			.sort((a, b) => b.count - a.count)
			.slice(0, 16);
	});

	const filteredServers = $derived(
		servers.filter((srv) => {
			const matchesSearch = 
				srv.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
				srv.address.toLowerCase().includes(searchQuery.toLowerCase()) ||
				srv.badges.some(b => b.toLowerCase().includes(searchQuery.toLowerCase()));
			
			const matchesTag = !selectedTag || srv.badges.includes(selectedTag);

			const matchesRegion = 
				selectedRegion === "all" ? true :
				selectedRegion === "br" ? srv.badges.includes("Brasil") :
				!srv.badges.includes("Brasil");

			return matchesSearch && matchesTag && matchesRegion;
		})
	);

	function copyServerIp(ip: string) {
		navigator.clipboard.writeText(ip);
		toast(`IP ${ip} copiado para a área de transferência!`, "success");
	}

	let isRefreshingPings = $state(false);

	async function refreshPings() {
		if (isRefreshingPings) return;
		isRefreshingPings = true;
		const targets = filteredServers.slice(0, 15);
		await Promise.allSettled(
			targets.map(async (s) => {
				try {
					const data = await serverPing(s.address, 25565);
					if (data) {
						liveServerData[s.id] = {
							online: data.playersOnline,
							max: data.playersMax || 1000,
							ping: data.latencyMs ?? 32,
						};
					}
				} catch {
					// Keep fallback
				}
			})
		);
		isRefreshingPings = false;
		toast("Pings dos servidores atualizados com sucesso!", "success");
	}

	onMount(() => {
		void refreshPings();
		pingInterval = setInterval(refreshPings, 45000);
	});

	onDestroy(() => {
		if (pingInterval) clearInterval(pingInterval);
	});
</script>

<div class="flex gap-8 h-full w-full select-none" in:fade={{ duration: 300 }}>
	
	<!-- Main Server Browser Content -->
	<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-6">
		
		<!-- Header -->
		<div class="flex items-center justify-between mt-1">
			<div>
				<div class="flex items-center gap-3">
					<h1 class="text-3xl font-extrabold text-white tracking-tight">Servidores de Minecraft</h1>
					<span class="bg-brand-500/20 text-brand-500 text-[10px] font-black px-2.5 py-1 rounded-lg uppercase tracking-wide border border-brand-500/30">
						{filteredServers.length} Servidores Ativos
					</span>
				</div>
				<p class="text-white/50 text-xs mt-0.5">Diretório completo de redes brasileiras e mundiais com ping em tempo real e cópia de IP com 1 clique</p>
			</div>

			<Button 
				variant="outline" 
				class="border-white/10 bg-[#1e1f23] hover:bg-white/10 text-white gap-2 rounded-xl text-xs px-4 py-2 cursor-pointer"
				disabled={isRefreshingPings}
				onclick={refreshPings}
			>
				<RefreshCw class="w-3.5 h-3.5 {isRefreshingPings ? 'animate-spin text-brand-500' : ''}" />
				{isRefreshingPings ? "Atualizando..." : "Atualizar Pings"}
			</Button>
		</div>

		<!-- Search & Region Filters -->
		<div class="flex items-center gap-3">
			<div class="relative flex-1">
				<Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40" />
				<input 
					type="text" 
					placeholder="Buscar servidor por nome, IP ou modo de jogo (ex: Mush, Hypixel, Bedwars, Pixelmon)..."
					bind:value={searchQuery}
					class="w-full bg-[#18191c] border border-white/10 rounded-2xl pl-10 pr-4 py-2.5 text-xs text-white placeholder-white/40 focus:outline-none focus:border-brand-500 transition-all shadow-inner"
				/>
			</div>

			<!-- Region Quick Selector -->
			<div class="flex items-center bg-[#18191c] p-1 rounded-full border border-white/10">
				<button 
					type="button" 
					class="px-4 py-1.5 rounded-full text-xs font-bold transition-all cursor-pointer {selectedRegion === 'all' ? 'text-black font-black' : 'text-white/50 hover:text-white'}"
					style={selectedRegion === 'all' ? 'background-color: var(--accent-color, #e2b86b);' : ''}
					onclick={() => selectedRegion = 'all'}
				>
					Todos ({servers.length})
				</button>
				<button 
					type="button" 
					class="px-4 py-1.5 rounded-full text-xs font-bold transition-all cursor-pointer {selectedRegion === 'br' ? 'text-black font-black' : 'text-white/50 hover:text-white'}"
					style={selectedRegion === 'br' ? 'background-color: var(--accent-color, #e2b86b);' : ''}
					onclick={() => selectedRegion = 'br'}
				>
					🇧🇷 Brasil (5)
				</button>
				<button 
					type="button" 
					class="px-4 py-1.5 rounded-full text-xs font-bold transition-all cursor-pointer {selectedRegion === 'intl' ? 'text-black font-black' : 'text-white/50 hover:text-white'}"
					style={selectedRegion === 'intl' ? 'background-color: var(--accent-color, #e2b86b);' : ''}
					onclick={() => selectedRegion = 'intl'}
				>
					🌎 Mundial (20)
				</button>
			</div>
		</div>

		<!-- Tags Filter Chips -->
		<div class="flex flex-wrap gap-1.5 items-center">
			<span class="text-[11px] font-bold text-white/40 uppercase mr-1">Filtrar por:</span>
			<button 
				type="button" 
				class="px-3.5 py-1 rounded-full text-[11px] font-bold transition-all cursor-pointer {!selectedTag ? 'text-black font-black' : 'bg-[#18191c] text-white/50 hover:text-white border border-white/5'}"
				style={!selectedTag ? 'background-color: var(--accent-color, #e2b86b);' : ''}
				onclick={() => selectedTag = null}
			>
				Todos
			</button>
			{#each tags as tag}
				<button 
					type="button" 
					class="px-3.5 py-1 rounded-full text-[11px] font-bold transition-all cursor-pointer {selectedTag === tag.name ? 'text-black font-black' : 'bg-[#18191c] text-white/50 hover:text-white border border-white/5'}"
					style={selectedTag === tag.name ? 'background-color: var(--accent-color, #e2b86b);' : ''}
					onclick={() => selectedTag = selectedTag === tag.name ? null : tag.name}
				>
					{tag.name} <span class="opacity-50 text-[10px]">({tag.count})</span>
				</button>
			{/each}
		</div>

		<!-- Server List Cards -->
		<div class="flex flex-col gap-3 pb-6">
			{#each filteredServers as srv}
				<div 
					class="flex items-center justify-between bg-[#18191c] hover:bg-[#1f2025] border border-white/5 hover:border-brand-500/40 p-4 rounded-3xl transition-all cursor-pointer group shadow-sm"
					onclick={() => copyServerIp(srv.address)}
					role="button"
					tabindex="0"
					onkeydown={(e) => { if (e.key === 'Enter') copyServerIp(srv.address); }}
					title="Clique para copiar o IP para a área de transferência"
				>
					<!-- Left: Rank, Logo & Info -->
					<div class="flex items-center gap-4 min-w-0">
						<span class="text-sm font-black text-white/30 group-hover:text-brand-500 w-6 text-center">
							#{srv.rank}
						</span>

						<div class="h-13 w-13 rounded-2xl bg-black/50 border border-white/10 flex items-center justify-center shrink-0 p-2 group-hover:scale-105 transition-transform shadow-md">
							<img src={srv.logo} alt={srv.name} class="w-full h-full object-contain rounded-lg" />
						</div>

						<div class="min-w-0">
							<div class="flex items-center gap-2">
								<h3 class="font-extrabold text-white text-sm group-hover:text-brand-500 transition-colors truncate">
									{srv.name}
								</h3>
								<button 
									type="button"
									class="p-1 rounded-lg hover:bg-white/10 text-white/30 hover:text-white transition-all cursor-pointer"
									onclick={(e) => { e.stopPropagation(); copyServerIp(srv.address); }}
									title="Copiar IP"
								>
									<Copy class="w-3 h-3" />
								</button>
							</div>

							<div class="flex items-center gap-2 text-xs text-white/40 mt-1">
								<span class="font-mono text-emerald-400 font-bold flex items-center gap-1">
									<Users class="w-3 h-3" />
									{liveServerData[srv.id]?.online ?? srv.online} online
								</span>
								<span>•</span>
								<span class="font-mono text-white/60">{srv.address}</span>
								<span>•</span>
								<span>{srv.version}</span>
							</div>

							<div class="flex gap-1.5 mt-2.5">
								{#each srv.badges as badge}
									<span class="bg-white/5 text-white/70 text-[9px] font-bold px-2.5 py-0.5 rounded-lg border border-white/5 uppercase tracking-wide">
										{badge}
									</span>
								{/each}
							</div>
						</div>
					</div>

					<!-- Right: Banner & Ping -->
					<div class="flex items-center gap-4 shrink-0 pl-4">
						<div class="hidden md:flex h-14 w-64 rounded-2xl bg-gradient-to-r {srv.bannerColor} border border-white/10 items-center justify-center p-3 text-center shadow-inner relative overflow-hidden group-hover:brightness-110 transition-all">
							<div class="absolute inset-0 bg-black/25 backdrop-blur-[1px]"></div>
							<span class="relative z-10 font-black text-white text-[11px] tracking-wider uppercase drop-shadow-md truncate px-2">
								{srv.bannerText}
							</span>
						</div>

						<div class="text-right">
							<span class="text-emerald-400 font-mono font-bold text-xs flex items-center gap-1 justify-end">
								<Signal class="w-3.5 h-3.5" /> {liveServerData[srv.id]?.ping ?? 24}ms
							</span>
							<span class="text-white/30 text-[10px] block mt-0.5 group-hover:text-brand-500 transition-colors font-bold">
								Copiar IP
							</span>
						</div>
					</div>
				</div>
			{/each}
		</div>

	</div>

</div>
