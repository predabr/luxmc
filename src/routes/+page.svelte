<script lang="ts">
	import { fade, slide, fly } from "svelte/transition";
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { 
		Play, 
		Plus, 
		Clock, 
		Users, 
		ArrowRight, 
		Boxes, 
		Download, 
		Sparkles, 
		Zap, 
		ShieldCheck, 
		Signal, 
		Gamepad2, 
		Loader2, 
		Flame, 
		Trophy, 
		Calendar, 
		Lock, 
		Eye, 
		EyeOff,
		ExternalLink,
		CheckCircle2,
		AlertCircle,
		SlidersHorizontal,
		FolderOpen,
		Camera,
		Package,
		ChevronDown,
		Search,
		Share2,
		Trash2,
		User
	} from "lucide-svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import Animate from "$lib/components/ui/Animate.svelte";
	import FavoriteServerWidget from "$lib/components/home/FavoriteServerWidget.svelte";
	import FriendsRadarWidget from "$lib/components/home/FriendsRadarWidget.svelte";
	import NewsFeedWidget from "$lib/components/home/NewsFeedWidget.svelte";
	import ScreenshotsWidget from "$lib/components/home/ScreenshotsWidget.svelte";
	import GamerCardModal from "$lib/components/profile/GamerCardModal.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";
	import { layoutStore } from "$lib/stores/layout.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playClick, playSuccess } from "$lib/utils/sounds";
	import { 
		authDevLogin, 
		authOfflineLogin, 
		authLogin, 
		authGetClientId,
		authSetClientId,
		launchGame, 
		versionsCheckInstalled, 
		versionsDownload, 
		discordSetActivity,
		instancesOpenFolder
	} from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	let offlineName = $state("");
	let offlinePassword = $state("");
	let offlineConfirmPassword = $state("");
	let showPassword = $state(false);
	let isLoggingIn = $state(false);
	let isLoggingInMicrosoft = $state(false);
	let isEntering = $state(false);
	let authTab = $state<"login" | "register">("login");
	let mainAuthType = $state<"microsoft" | "offline">("microsoft");
	let savedAccounts = $state<string[]>([]);
	let savedAccountsMap = $state<Record<string, string>>({});
	let isLoadingHome = $state(true);
	let hoveredSavedAcc = $state<string | null>(null);
	let isLoginTransitioning = $state(false);
	let isLaunching = $state(false);
	let launchStatusText = $state("");
	let showQuickInstancePicker = $state(false);
	let showGamerCardModal = $state(false);

	const activeInstance = $derived(profiles.active || profiles.list[0] || null);

	function getLoaderColor(loader: string) {
		const l = (loader || "").toLowerCase();
		if (l.includes("fabric")) return "bg-sky-500/15 text-sky-400 border-sky-500/30";
		if (l.includes("neoforge")) return "bg-orange-500/15 text-orange-400 border-orange-500/30";
		if (l.includes("forge")) return "bg-amber-500/15 text-amber-400 border-amber-500/30";
		if (l.includes("quilt")) return "bg-purple-500/15 text-purple-400 border-purple-500/30";
		return "bg-emerald-500/15 text-emerald-400 border-emerald-500/30";
	}

	async function hashPassword(password: string): Promise<string> {
		const encoder = new TextEncoder();
		const data = encoder.encode(password + 'luxmc_salt_v1');
		const hashBuffer = await crypto.subtle.digest('SHA-256', data);
		const hashArray = Array.from(new Uint8Array(hashBuffer));
		return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
	}

	function deleteSavedAccount(name: string) {
		const accountsMapRaw = localStorage.getItem("luxmc_offline_passwords");
		if (!accountsMapRaw) return;
		const map: Record<string, string> = JSON.parse(accountsMapRaw);
		delete map[name.toLowerCase()];
		localStorage.setItem("luxmc_offline_passwords", JSON.stringify(map));
		savedAccounts = Object.keys(map);
		savedAccountsMap = map;
		toast(`Conta "${name}" removida.`, "info");
	}

	onMount(() => {
		const accountsMapRaw = localStorage.getItem("luxmc_offline_passwords");
		if (accountsMapRaw) {
			try {
				const map = JSON.parse(accountsMapRaw);
				savedAccounts = Object.keys(map);
				savedAccountsMap = map;
			} catch {}
		}

		if (!account.value) {
			const saved = localStorage.getItem("luxmc_current_account");
			if (saved) {
				try {
					account.value = JSON.parse(saved);
				} catch {}
			}
		}

		const skeletonTimer = setTimeout(() => {
			isLoadingHome = false;
		}, 350);

		return () => clearTimeout(skeletonTimer);
	});

	async function handleOfflineAuth() {
		const name = offlineName.trim();
		const pass = offlinePassword.trim();
		if (!name) {
			toast(t("home.pleaseEnterGamertag"), "error");
			return;
		}
		if (!pass) {
			toast(t("home.pleaseEnterPassword"), "error");
			return;
		}

		isLoggingIn = true;
		try {
			const accountsMapRaw = localStorage.getItem("luxmc_offline_passwords");
			const accountsMap: Record<string, string> = accountsMapRaw ? JSON.parse(accountsMapRaw) : {};
			const key = name.toLowerCase();

			if (authTab === "register") {
				const confirm = offlineConfirmPassword.trim();
				if (pass.length < 3) {
					toast(t("home.passwordMinLength"), "error");
					isLoggingIn = false;
					return;
				}
				if (pass !== confirm) {
					toast(t("home.passwordsMismatch"), "error");
					isLoggingIn = false;
					return;
				}
				if (accountsMap[key]) {
					toast(t("home.gamertagExists", { name }), "error");
					isLoggingIn = false;
					return;
				}
				accountsMap[key] = await hashPassword(pass);
				localStorage.setItem("luxmc_offline_passwords", JSON.stringify(accountsMap));
				toast(t("home.accountCreated", { name }), "success");
			} else {
				if (!accountsMap[key]) {
					toast(t("home.accountNotRegistered", { name }), "error");
					isLoggingIn = false;
					return;
				}
				const hashedPass = await hashPassword(pass);
				if (accountsMap[key] !== hashedPass) {
					toast(t("home.incorrectPassword", { name }), "error");
					isLoggingIn = false;
					return;
				}
			}

			const backendAcc = await authOfflineLogin(name).catch(() => null);

			const skinUrl = backendAcc?.skinUrl || `https://minotar.net/skin/${name}`;
			const newAcc = {
				id: backendAcc?.id || ("offline_" + Date.now()),
				username: name,
				uuid: backendAcc?.uuid || ("offline-" + key),
				minecraftToken: "",
				expiresAt: 0,
				skinUrl,
				skinVariant: "classic",
				capeUrl: null
			};
			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
			activeSkinStore.setSkin({
				id: newAcc.uuid,
				name: newAcc.username,
				url: `https://mc-heads.net/body/${newAcc.username}/300`,
				skinUrl,
				avatarUrl: `https://mc-heads.net/avatar/${newAcc.username}/100`,
				type: "steve"
			});
			toast(t("home.welcomeToLuxmc", { name }), "success");
			playSuccess();

			account.value = newAcc;
		} catch (e) {
			toast(t("home.accountError", { error: String(e) }), "error");
		} finally {
			isLoggingIn = false;
		}
	}

	let showMsClientIdModal = $state(false);
	let msClientIdInput = $state("");
	let isSavingClientId = $state(false);

	async function handleMicrosoftLogin() {
		try {
			const existingId = await authGetClientId().catch(() => "");
			if (!existingId || existingId === "00000000-0000-0000-0000-000000000000") {
				await authSetClientId("9750ebbe-21e9-4a4d-b808-f451a3e0af7f").catch(() => null);
			}
			await startMsLogin();
		} catch (e) {
			toast(String(e), "error");
		}
	}

	async function startMsLogin() {
		isLoggingInMicrosoft = true;
		try {
			toast(t("home.msAuthStarted"), "info");
			const acc = await authLogin();
			const newAcc = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken,
				expiresAt: acc.expiresAt ? (acc.expiresAt < 1e11 ? acc.expiresAt * 1000 : acc.expiresAt) : 0,
				skinUrl: acc.skinUrl ?? null,
				skinVariant: acc.skinVariant ?? null,
				capeUrl: acc.capeUrl ?? null,
			};
			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));

			activeSkinStore.setSkin({
				id: acc.uuid,
				name: acc.username,
				url: `https://mc-heads.net/body/${acc.username}/300`,
				skinUrl: acc.skinUrl || `https://minotar.net/skin/${acc.username}`,
				avatarUrl: `https://mc-heads.net/avatar/${acc.username}/100`,
				type: acc.skinVariant?.toLowerCase() === "slim" ? "alex" : "steve",
				hasCape: Boolean(acc.capeUrl),
				capeType: acc.capeUrl ? "custom" : "none",
				customCapeUrl: acc.capeUrl || ""
			});

			toast(t("home.connectedAs", { username: acc.username }), "success");
			playSuccess();

			account.value = newAcc;
		} catch (e) {
			const errStr = String(e);
			if (errStr.includes("client_id_required") || errStr.includes("unauthorized_client") || errStr.includes("AADSTS700016")) {
				showMsClientIdModal = true;
			} else {
				toast(t("home.loginAttempt", { error: errStr }), "error");
			}
		} finally {
			isLoggingInMicrosoft = false;
		}
	}

	async function saveAndLoginWithClientId() {
		const cid = msClientIdInput.trim();
		if (!cid) {
			toast("Insira um Application (client) ID válido do Azure.", "error");
			return;
		}
		isSavingClientId = true;
		try {
			await authSetClientId(cid);
			toast("Client ID da Microsoft configurado!", "success");
			showMsClientIdModal = false;
			await startMsLogin();
		} catch (e) {
			toast("Erro ao salvar Client ID: " + String(e), "error");
		} finally {
			isSavingClientId = false;
		}
	}

	async function handleDevLogin() {
		isLoggingIn = true;
		try {
			const acc = await authDevLogin().catch(async () => {
				const offline = await authOfflineLogin("DevPlayer").catch(() => null);
				return {
					id: offline?.id || "dev-00000000-0000-0000-0000-000000000001",
					username: offline?.username || "DevPlayer",
					uuid: offline?.uuid || "00000000-0000-0000-0000-000000000001",
					accessToken: offline?.accessToken || "dev-access-token",
					expiresAt: offline?.expiresAt || (Date.now() + 86400 * 1000)
				};
			});
			const skinUrl = "https://minotar.net/skin/MHF_Steve";
			const newAcc = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken,
				expiresAt: acc.expiresAt ? (acc.expiresAt < 1e11 ? acc.expiresAt * 1000 : acc.expiresAt) : 0,
				skinUrl,
				skinVariant: "classic",
				capeUrl: null
			};
			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
			activeSkinStore.setSkin({
				id: newAcc.uuid,
				name: newAcc.username,
				url: `https://mc-heads.net/body/${newAcc.username}/300`,
				skinUrl,
				avatarUrl: `https://mc-heads.net/avatar/${newAcc.username}/100`,
				type: "steve"
			});
			toast(t("home.connectedAsDev", { username: acc.username }), "success");
			playSuccess();
			account.value = newAcc;
		} catch (e) {
			toast(t("home.devLoginErrorToast", { error: String(e) }), "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleHomePlay(serverIp?: string, serverPort?: number) {
		if (isLaunching) return;

		const targetProfile = activeInstance;
		if (!targetProfile) {
			toast(t("home.noInstanceFound"), "info");
			goto("/instances?new=true");
			return;
		}

		isLaunching = true;
		launchStatusText = serverIp ? `Conectando diretamente a ${serverIp}...` : t("home.starting");

		try {
			let userUuid = account.value?.uuid;
			if (!userUuid) {
				const devAcc = await authDevLogin();
				account.value = {
					id: devAcc.id,
					username: devAcc.username,
					uuid: devAcc.uuid,
					minecraftToken: devAcc.accessToken,
					expiresAt: devAcc.expiresAt ? (devAcc.expiresAt < 1e11 ? devAcc.expiresAt * 1000 : devAcc.expiresAt) : 0
				};
				userUuid = devAcc.uuid;
			}

			const verId = targetProfile.mcVersion || "1.20.4";
			launchStatusText = t("home.checkingFiles");

			const installed = await versionsCheckInstalled(verId).catch(() => false);
			if (!installed) {
				launchStatusText = t("home.downloadingMc", { version: verId });
				await versionsDownload(verId);
			}

			launchStatusText = serverIp ? `Iniciando e conectando a ${serverIp}...` : t("home.startingMc");
			const isVulkan = typeof window !== "undefined" ? localStorage.getItem("luxmc_enable_vulkan") === "true" : false;
			const skinToPass = activeSkinStore.current.skinUrl || account.value?.skinUrl || null;
			const effectiveCape = activeSkinStore.current.hasCape
				? (activeSkinStore.current.customCapeUrl || (activeSkinStore.current.capeType && activeSkinStore.current.capeType !== "none" ? getFullCapeDataUrl(activeSkinStore.current.capeType) : null))
				: (account.value?.capeUrl || null);
			const result = await launchGame({
				versionId: verId,
				accountId: userUuid || "",
				profileId: targetProfile.id,
				enableVulkan: isVulkan,
				skinUrl: skinToPass,
				skinVariant: activeSkinStore.current.type === "alex" ? "slim" : "classic",
				capeUrl: effectiveCape,
				serverIp: serverIp || null,
				serverPort: serverPort || null
			});

			gamingStats.onGameStart();
			const tNameLower = targetProfile.name.toLowerCase();
			const modpackCover = (targetProfile.icon && (targetProfile.icon.startsWith("http://") || targetProfile.icon.startsWith("https://")))
				? targetProfile.icon
				: tNameLower.includes("better mc") || tNameLower.includes("bmc")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_better_mc.webp"
				: tNameLower.includes("all the mods") || tNameLower.includes("atm")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_atm.webp"
				: tNameLower.includes("pixelmon") || tNameLower.includes("cobblemon")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_cobblemon.webp"
				: tNameLower.includes("fabulously optimized") || tNameLower.includes("fo")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_fo.webp"
				: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png";

			discordSetActivity({
				inGame: true,
				details: targetProfile.name,
				state: `Minecraft ${verId} · ${targetProfile.loader ? targetProfile.loader.toUpperCase() : "Vanilla"}`,
				largeText: targetProfile.name,
				largeImage: modpackCover,
				smallImage: targetProfile.loader === "fabric" ? "fabric" : (targetProfile.loader === "forge" ? "curse" : "grass"),
				smallText: `Luxmc · ${targetProfile.loader || "Vanilla"}`,
				startTime: Math.floor(Date.now() / 1000)
			}).catch(() => {});

			toast(`🎮 ${t("home.mcLaunched", { version: verId, pid: String(result.pid) })}`, "success");
			profiles.setLastPlayed(targetProfile.id);
		} catch (e) {
			console.error("Home launch error:", e);
			toast(t("home.launchFailed", { error: String(e) }), "error");
		} finally {
			isLaunching = false;
			launchStatusText = "";
		}
	}

	function handleQuickServerJoin(host: string, port: number) {
		toast(`Iniciando Minecraft com entrada direta no servidor ${host}:${port}...`, "info");
		handleHomePlay(host, port);
	}

	const modpacks = [
		{
			id: "fo",
			title: "Fabulously Optimized",
			subtitle: "Pacote de alta fidelidade visual, estabilidade máxima e FPS extremo para Linux.",
			author: "robotkoer",
			downloads: "16.9M",
			bgImg: "/modpack_fo.webp",
			iconImg: "/modpack_fo_icon.png",
			badgeText: "FO"
		},
		{
			id: "cobblemon",
			title: "Cobblemon Official Modpack",
			subtitle: "Capture, treine e explore um mundo vivo com criaturas em estilo pixel art.",
			author: "CobbledStudios",
			downloads: "10.4M",
			bgImg: "/modpack_cobblemon.webp",
			iconImg: "/modpack_cobblemon_icon.png",
			badgeText: "COBBLE"
		},
		{
			id: "bmc2",
			title: "Better MC [FABRIC] - BMC2",
			subtitle: "A experiência definitiva de Minecraft com novas dimensões, chefes e masmorras.",
			author: "SHXRKIE",
			downloads: "3.3M",
			bgImg: "/modpack_better_mc.webp",
			iconImg: "/modpack_bmc_icon.webp",
			badgeText: "BMC2"
		}
	];

	async function openInstanceFolder(id?: string) {
		const targetId = id || activeInstance?.id;
		if (!targetId) {
			toast("Nenhuma instância selecionada.", "warning");
			return;
		}
		try {
			await instancesOpenFolder(targetId);
			toast("Pasta da instância aberta.", "success");
		} catch (e) {
			toast("Falha ao abrir pasta: " + String(e), "error");
		}
	}
</script>

{#if !account.value}
	<div class="flex h-full w-full items-center justify-center px-4 transition-all duration-700 {isEntering ? 'opacity-0 scale-95 blur-md pointer-events-none' : 'opacity-100 scale-100'}" in:fade={{ duration: 400 }}>
		<div class="w-full max-w-lg rounded-[2rem] bg-[#111216] border border-white/[0.08] p-8 sm:p-10 shadow-[0_32px_64px_rgba(0,0,0,0.6)] flex flex-col items-center select-none relative overflow-hidden">
			<div class="absolute -top-20 -right-20 w-72 h-72 bg-[#caa97c]/[0.07] rounded-full blur-[100px] pointer-events-none animate-pulse" style="animation-duration:4s"></div>
			<div class="absolute -bottom-20 -left-20 w-72 h-72 bg-[#6c5ce7]/[0.07] rounded-full blur-[100px] pointer-events-none animate-pulse" style="animation-duration:5s"></div>
			<div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-[#caa97c]/[0.03] rounded-full blur-[120px] pointer-events-none"></div>

			<Animate delay={0.1} duration={0.6}>
				<div class="flex flex-col items-center text-center relative z-10">
					<div class="h-28 w-28 rounded-[1.75rem] bg-gradient-to-br from-[#1a1b21] to-[#111216] border border-white/[0.12] flex items-center justify-center p-3.5 shadow-[0_16px_48px_rgba(0,0,0,0.5),0_0_0_1px_rgba(202,169,124,0.1)] relative group">
						<div class="absolute inset-0 rounded-[1.75rem] bg-gradient-to-br from-[#caa97c]/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500"></div>
						<img src="/logo.png" alt="Luxmc 3D" class="w-full h-full object-contain drop-shadow-[0_0_32px_rgba(202,169,124,0.4)] group-hover:scale-110 transition-transform duration-500 relative z-10" />
					</div>
					<h1 class="text-[1.75rem] font-black text-white tracking-tight mt-5 bg-gradient-to-r from-white via-white to-white/70 bg-clip-text text-transparent">Luxmc Launcher</h1>
					<p class="text-[11px] text-white/40 mt-1.5 font-medium tracking-wide">{t("home.subTagline")}</p>
				</div>
			</Animate>

			<Animate delay={0.25} duration={0.6}>
				<div class="w-full space-y-5 mt-7 relative z-10">
				<!-- Segmented Auth Mode Selector -->
				<div class="flex bg-[#0e0f12] p-1.5 rounded-2xl border border-white/[0.06] shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]">
					<button 
						type="button" 
						class="flex-1 py-3 px-3 rounded-xl text-xs font-bold transition-all duration-300 cursor-pointer flex items-center justify-center gap-2.5 {mainAuthType === 'microsoft' ? 'bg-gradient-to-b from-[#25262e] to-[#1e1f26] text-white shadow-[0_4px_16px_rgba(0,0,0,0.3),0_0_0_1px_rgba(108,92,231,0.3)]' : 'text-white/30 hover:text-white/60 border border-transparent hover:bg-white/[0.03]'}"
						onclick={() => mainAuthType = 'microsoft'}
					>
						<svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none">
							<rect x="1" y="1" width="10" height="10" fill="#f25022" rx="1.5"/>
							<rect x="13" y="1" width="10" height="10" fill="#7fba00" rx="1.5"/>
							<rect x="1" y="13" width="10" height="10" fill="#00a4ef" rx="1.5"/>
							<rect x="13" y="13" width="10" height="10" fill="#ffb900" rx="1.5"/>
						</svg>
						<span>Conta Microsoft</span>
					</button>
					<button 
						type="button" 
						class="flex-1 py-3 px-3 rounded-xl text-xs font-bold transition-all duration-300 cursor-pointer flex items-center justify-center gap-2.5 {mainAuthType === 'offline' ? 'bg-gradient-to-b from-[#25262e] to-[#1e1f26] text-[#caa97c] shadow-[0_4px_16px_rgba(0,0,0,0.3),0_0_0_1px_rgba(202,169,124,0.25)]' : 'text-white/30 hover:text-white/60 border border-transparent hover:bg-white/[0.03]'}"
						onclick={() => mainAuthType = 'offline'}
					>
						<Gamepad2 class="w-4 h-4 shrink-0" />
						<span>Modo Offline</span>
					</button>
				</div>

				{#if mainAuthType === 'microsoft'}
					{#if isLoggingInMicrosoft}
						<div class="w-full bg-[#13141a] border border-[#6c5ce7]/30 rounded-3xl p-6 shadow-[0_24px_48px_rgba(0,0,0,0.4)] relative overflow-hidden space-y-5" in:fade={{ duration: 250 }}>
							<div class="absolute -right-12 -top-12 w-48 h-48 bg-[#6c5ce7]/15 rounded-full blur-[60px] pointer-events-none"></div>
							<div class="absolute -left-8 -bottom-8 w-32 h-32 bg-[#a29bfe]/10 rounded-full blur-[40px] pointer-events-none"></div>

							<div class="flex items-center gap-4">
								<div class="h-14 w-14 rounded-2xl bg-gradient-to-br from-[#6c5ce7]/20 to-[#a29bfe]/10 border border-[#6c5ce7]/30 flex items-center justify-center shrink-0 shadow-inner relative">
									<Loader2 class="w-6 h-6 text-[#a29bfe] animate-spin" />
									<div class="absolute -top-1 -right-1 w-3.5 h-3.5 bg-emerald-400 rounded-full border-2 border-[#13141a] animate-ping"></div>
								</div>
								<div>
									<div class="flex items-center gap-2.5">
										<h3 class="text-sm font-black text-white">{t("home.msAuth")}</h3>
										<span class="inline-flex items-center gap-1 text-[9px] font-extrabold text-emerald-400 bg-emerald-500/10 px-2.5 py-0.5 rounded-full border border-emerald-500/20">
											<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> {t("home.msWaiting")}
										</span>
									</div>
									<p class="text-[11px] text-white/45 mt-0.5">{t("home.msCompleteInBrowser")}</p>
								</div>
							</div>

							<div class="space-y-3 py-1">
								<div class="h-1.5 w-full bg-white/[0.04] rounded-full overflow-hidden relative">
									<div class="h-full bg-gradient-to-r from-[#6c5ce7] via-[#a29bfe] to-[#6c5ce7] rounded-full animate-[shimmer_2s_ease-in-out_infinite]" style="width:65%;background-size:200% 100%"></div>
								</div>
								<p class="text-[11px] text-white/50 leading-relaxed">
									{t("home.msAccessDesc")}
								</p>
							</div>

							<div class="flex items-center justify-between pt-3 border-t border-white/[0.05]">
								<span class="text-[10px] text-white/35 flex items-center gap-1.5">
									<ExternalLink class="w-3 h-3 text-[#a29bfe]" />
									{t("home.oauthWindowOpen")}
								</span>
								<button 
									type="button" 
									class="text-xs font-bold text-white/60 hover:text-white px-4 py-2 rounded-xl bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.06] transition-all duration-200 cursor-pointer"
									onclick={() => { isLoggingInMicrosoft = false; }}
								>
									{t("home.cancel")}
								</button>
							</div>
						</div>
					{:else}
						<div class="bg-[#13141a] border border-white/[0.08] rounded-3xl p-6 space-y-5 shadow-[0_16px_32px_rgba(0,0,0,0.3)] relative overflow-hidden" in:fade={{ duration: 250 }}>
							<div class="absolute -right-12 -top-12 w-56 h-56 bg-[#6c5ce7]/[0.08] rounded-full blur-[80px] pointer-events-none"></div>
							<div class="absolute -left-10 -bottom-10 w-40 h-40 bg-[#00a4ef]/[0.06] rounded-full blur-[60px] pointer-events-none"></div>

							<div class="space-y-2 relative z-10">
								<div class="flex items-center gap-3">
									<div class="flex items-center gap-0.5">
										<div class="w-2.5 h-2.5 rounded-full bg-[#f25022]"></div>
										<div class="w-2.5 h-2.5 rounded-full bg-[#7fba00]"></div>
										<div class="w-2.5 h-2.5 rounded-full bg-[#00a4ef]"></div>
										<div class="w-2.5 h-2.5 rounded-full bg-[#ffb900]"></div>
									</div>
									<span class="font-extrabold text-white text-sm tracking-tight">Minecraft Oficial</span>
									<span class="bg-[#6c5ce7]/15 text-[#a29bfe] text-[9px] font-black px-2.5 py-0.5 rounded-full border border-[#6c5ce7]/25 uppercase tracking-widest">Original</span>
								</div>
								<p class="text-[11px] text-white/40 leading-relaxed max-w-sm">Conecte sua conta Microsoft para skins oficiais, capas, realms e servidores multijogador.</p>
							</div>

							<button 
								type="button" 
								class="w-full rounded-2xl bg-gradient-to-r from-[#1a1b24] to-[#15161e] hover:from-[#22232e] hover:to-[#1c1d26] border border-[#6c5ce7]/30 hover:border-[#6c5ce7]/70 p-5 text-white flex items-center gap-4 transition-all duration-300 active:scale-[0.98] cursor-pointer shadow-[0_8px_24px_rgba(0,0,0,0.3)] hover:shadow-[0_12px_32px_rgba(108,92,231,0.2)] relative overflow-hidden disabled:opacity-50 group"
								onclick={() => { playClick(); handleMicrosoftLogin(); }}
								disabled={isLoggingIn || isLoggingInMicrosoft}
							>
								<div class="absolute inset-0 bg-gradient-to-r from-[#6c5ce7]/0 via-[#6c5ce7]/5 to-[#6c5ce7]/0 opacity-0 group-hover:opacity-100 transition-opacity duration-500"></div>
								<div class="h-12 w-12 rounded-xl bg-gradient-to-br from-black/80 to-black/40 border border-white/[0.12] flex items-center justify-center p-2.5 shrink-0 shadow-inner relative z-10">
									<svg class="h-full w-full" viewBox="0 0 24 24" fill="none">
										<rect x="1" y="1" width="10" height="10" fill="#f25022" rx="1.5"/>
										<rect x="13" y="1" width="10" height="10" fill="#7fba00" rx="1.5"/>
										<rect x="1" y="13" width="10" height="10" fill="#00a4ef" rx="1.5"/>
										<rect x="13" y="13" width="10" height="10" fill="#ffb900" rx="1.5"/>
									</svg>
								</div>
								<div class="text-left flex-1 min-w-0 relative z-10">
									<span class="font-bold text-white text-sm whitespace-nowrap">{t("home.signInWithMicrosoft")}</span>
									<p class="text-[11px] text-white/40 font-medium truncate mt-0.5">{t("home.msOfficialAccount")}</p>
								</div>
								<ArrowRight class="w-5 h-5 text-white/30 group-hover:text-white group-hover:translate-x-1 transition-all duration-300 shrink-0 relative z-10" />
							</button>

							<div class="flex items-center justify-between pt-2.5 border-t border-white/[0.04]">
								<span class="flex items-center gap-1.5 text-[10px] text-white/30">
									<ShieldCheck class="w-3.5 h-3.5 text-emerald-400" />
									Autenticação oficial via browser
								</span>
								<button
									type="button"
									class="text-[10px] text-white/30 hover:text-amber-400 transition-colors cursor-pointer underline underline-offset-2 decoration-white/10 hover:decoration-amber-400/50"
									onclick={() => showMsClientIdModal = true}
								>
									Azure ID
								</button>
							</div>
						</div>
					{/if}

				{:else}
					<!-- Offline Login Box -->
					<div class="bg-[#13141a] border border-white/[0.08] rounded-3xl p-6 space-y-5 shadow-[0_16px_32px_rgba(0,0,0,0.3)] relative" in:fade={{ duration: 250 }}>
						<div class="flex bg-[#0e0f12] p-1 rounded-full border border-white/[0.05]">
							<button 
								type="button" 
								class="flex-1 py-2.5 rounded-full text-xs font-bold transition-all duration-300 cursor-pointer {authTab === 'login' ? 'bg-[#1e1f26] text-[#caa97c] shadow-sm border border-[#caa97c]/15' : 'text-white/30 hover:text-white/60 border border-transparent'}"
								onclick={() => authTab = 'login'}
							>
								{t("home.loginTab")}
							</button>
							<button 
								type="button" 
								class="flex-1 py-2.5 rounded-full text-xs font-bold transition-all duration-300 cursor-pointer {authTab === 'register' ? 'bg-[#1e1f26] text-[#caa97c] shadow-sm border border-[#caa97c]/15' : 'text-white/30 hover:text-white/60 border border-transparent'}"
								onclick={() => authTab = 'register'}
							>
								{t("home.createAccountTab")}
							</button>
						</div>

						{#if savedAccounts.length > 0 && authTab === 'login'}
							<div class="space-y-2.5 pt-0.5" transition:slide={{ duration: 300 }}>
								<div class="flex items-center gap-2">
									<span class="text-[10px] text-white/35 font-bold uppercase tracking-widest">{t("home.savedAccounts")}</span>
									<div class="flex-1 h-px bg-white/[0.04]"></div>
								</div>
								<div class="flex flex-wrap gap-2">
									{#each savedAccounts as accName (accName)}
										<div 
											class="relative group"
											role="presentation"
										>
											<button 
												type="button" 
												class="flex items-center gap-2.5 pl-1.5 pr-3 py-1.5 rounded-2xl border transition-all duration-300 cursor-pointer hover:scale-[1.02] active:scale-[0.98] {offlineName.toLowerCase() === accName.toLowerCase() ? 'border-[#caa97c]/40 bg-[#caa97c]/10 shadow-[0_0_16px_rgba(202,169,124,0.12)]' : 'border-white/[0.06] bg-white/[0.02] hover:border-white/[0.12] hover:bg-white/[0.04]'}"
												onclick={() => { offlineName = accName; }}
											>
												<div class="h-7 w-7 rounded-lg overflow-hidden bg-[#1a1b21] border border-white/[0.1] shrink-0">
													<img 
														src="https://mc-heads.net/avatar/{accName}/56" 
														alt={accName}
														class="w-full h-full object-cover"
														loading="lazy"
													/>
												</div>
												<span class="text-[11px] font-bold {offlineName.toLowerCase() === accName.toLowerCase() ? 'text-[#caa97c]' : 'text-white/60 group-hover:text-white/80'} transition-colors">{accName}</span>
											</button>
											<button
												type="button"
												class="absolute -top-1.5 -right-1.5 w-5 h-5 rounded-full bg-[#2a1515] border border-red-500/30 text-red-400 flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-red-500/20 transition-all duration-200 cursor-pointer z-10 shadow-md"
												onclick={(e) => { e.stopPropagation(); deleteSavedAccount(accName); }}
												title="Remover conta salva"
											>
												<Trash2 class="w-2.5 h-2.5" />
											</button>
										</div>
									{/each}
								</div>
							</div>
						{/if}

						<div class="flex items-center gap-2.5 text-xs font-bold text-white/70">
							<div class="h-8 w-8 rounded-xl bg-[#caa97c]/10 border border-[#caa97c]/20 flex items-center justify-center">
								<Gamepad2 class="w-4 h-4 text-[#caa97c]" />
							</div>
							<span>{authTab === 'login' ? t("home.accessOfflineAccount") : t("home.registerNewAccount")}</span>
						</div>

						<div class="space-y-3">
							<div class="relative">
								<input 
									type="text" 
									placeholder={t("home.gamertagPlaceholder")} 
									aria-label="Gamertag"
									bind:value={offlineName}
									class="w-full bg-[#0e0f12] border border-white/[0.06] rounded-2xl pl-5 pr-4 py-3.5 text-xs font-bold text-white outline-none focus:border-[#caa97c]/50 focus:shadow-[0_0_0_3px_rgba(202,169,124,0.08)] transition-all duration-300 placeholder:text-white/20"
									maxlength="16"
									onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
								/>
							</div>

							<div class="relative">
								<input 
									type={showPassword ? "text" : "password"} 
									placeholder={t("home.accountPasswordPlaceholder")} 
									aria-label="Senha"
									bind:value={offlinePassword}
									class="w-full bg-[#0e0f12] border border-white/[0.06] rounded-2xl pl-5 pr-12 py-3.5 text-xs font-bold text-white outline-none focus:border-[#caa97c]/50 focus:shadow-[0_0_0_3px_rgba(202,169,124,0.08)] transition-all duration-300 placeholder:text-white/20"
									onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
								/>
								<button
									type="button"
									class="absolute right-3.5 top-1/2 -translate-y-1/2 text-white/30 hover:text-[#caa97c] p-1.5 transition-all duration-200 cursor-pointer"
									onclick={() => (showPassword = !showPassword)}
									title={showPassword ? t("home.hidePassword") : t("home.showPassword")}
								>
									{#if showPassword}
										<EyeOff class="w-3.5 h-3.5" />
									{:else}
										<Eye class="w-3.5 h-3.5" />
									{/if}
								</button>
							</div>

							{#if authTab === 'register'}
								<div transition:slide={{ duration: 200 }}>
									<input 
										type={showPassword ? "text" : "password"} 
										placeholder={t("home.confirmPasswordPlaceholder")} 
										aria-label="Confirmar senha"
										bind:value={offlineConfirmPassword}
										class="w-full bg-[#0e0f12] border border-white/[0.06] rounded-2xl pl-5 pr-4 py-3.5 text-xs font-bold text-white outline-none focus:border-[#caa97c]/50 focus:shadow-[0_0_0_3px_rgba(202,169,124,0.08)] transition-all duration-300 placeholder:text-white/20"
										onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
									/>
								</div>
							{/if}
						</div>

						<button 
							type="button" 
							class="w-full h-12 rounded-2xl bg-gradient-to-r from-[#d8bc98] via-[#caa97c] to-[#b89560] hover:from-[#e5cca8] hover:to-[#caa97c] text-[#111215] font-black text-xs uppercase tracking-wider flex items-center justify-center gap-2.5 transition-all duration-300 shadow-[0_8px_24px_rgba(202,169,124,0.25)] hover:shadow-[0_12px_32px_rgba(202,169,124,0.4)] hover:scale-[1.01] active:scale-[0.98] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
							onclick={() => { playClick(); handleOfflineAuth(); }}
							disabled={isLoggingIn || isLoggingInMicrosoft}
						>
							{#if isLoggingIn}
								<Loader2 class="w-4 h-4 animate-spin" /> {t("home.processing")}
							{:else}
								<Play class="w-4 h-4 fill-current" /> {authTab === 'login' ? t("home.loginAndPlay") : t("home.createAndPlay")}
							{/if}
						</button>
					</div>

					<div class="flex flex-col items-center gap-2 pt-1">
						<button 
							type="button" 
							class="group px-4 py-2.5 rounded-2xl bg-white/[0.02] hover:bg-white/[0.05] border border-white/[0.05] hover:border-amber-500/30 text-white/50 hover:text-white transition-all duration-300 flex items-center gap-2.5 cursor-pointer text-xs disabled:opacity-40"
							onclick={handleDevLogin}
							disabled={isLoggingIn || isLoggingInMicrosoft}
						>
							<span class="bg-amber-500/15 text-amber-300 text-[9px] font-black px-2.5 py-0.5 rounded-full border border-amber-500/25 uppercase tracking-widest">
								{t("home.devMode")}
							</span>
							<span class="font-semibold text-[11px] text-white/60 group-hover:text-white transition-colors">{t("home.devAccess")}</span>
							{#if isLoggingIn}
								<Loader2 class="w-3.5 h-3.5 animate-spin text-amber-400" />
							{/if}
						</button>
					</div>
				{/if}
			</div>

			<div class="text-[10px] text-white/20 text-center leading-relaxed mt-1">
				{t("home.compatibleWith")}
			</div>
			</Animate>
		</div>
	</div>

	{#if showMsClientIdModal}
		<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md" in:fade={{ duration: 200 }}>
			<div class="w-full max-w-md rounded-3xl bg-[#13141a] border border-amber-500/20 p-7 shadow-[0_32px_64px_rgba(0,0,0,0.6)] space-y-5 relative overflow-hidden" in:fly={{ y: 20, duration: 300 }}>
				<div class="absolute -right-12 -top-12 w-48 h-48 bg-amber-500/[0.06] rounded-full blur-[80px] pointer-events-none"></div>

				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-xl bg-amber-500/15 text-amber-400 flex items-center justify-center border border-amber-500/20">
							<Lock class="w-4.5 h-4.5" />
						</div>
						<div>
							<h3 class="text-sm font-black text-white">Configurar Microsoft Azure</h3>
							<p class="text-[10px] text-white/40">Application (client) ID Requerido</p>
						</div>
					</div>
					<button 
						type="button" 
						class="w-8 h-8 rounded-xl bg-white/[0.04] hover:bg-white/[0.08] text-white/40 hover:text-white flex items-center justify-center text-xs transition-all cursor-pointer border border-white/[0.06]"
						onclick={() => showMsClientIdModal = false}
					>
						✕
					</button>
				</div>

				<p class="text-xs text-white/60 leading-relaxed">
					Para autenticar com a Microsoft, insira o <span class="text-amber-400 font-bold">Application (client) ID</span> gerado no seu registro de aplicativo no Azure Portal (configurado para <em>Personal Microsoft accounts only</em>).
				</p>

				<div class="space-y-2">
					<label for="ms-client-input" class="text-[11px] font-bold text-white/70 block">Client ID (UUID):</label>
					<input 
						id="ms-client-input"
						type="text" 
						placeholder="9750ebbe-21e9-4a4d-b808-f451a3e0af7f" 
						bind:value={msClientIdInput}
						class="w-full bg-[#0e0f12] border border-white/[0.08] focus:border-amber-500/50 rounded-2xl px-4 py-3 text-xs text-white font-mono outline-none transition-all focus:shadow-[0_0_0_3px_rgba(245,158,11,0.08)]"
						onkeydown={(e) => { if (e.key === "Enter") saveAndLoginWithClientId(); }}
					/>
				</div>

				<div class="p-3.5 rounded-2xl bg-white/[0.02] border border-white/[0.04] text-[10px] text-white/45 space-y-1">
					<div class="font-bold text-white/60">Aviso do Azure Portal:</div>
					<div>Redirect URI configurado deve ser: <code class="text-amber-300 font-mono">http://localhost:8453/callback</code></div>
				</div>

				<div class="flex items-center gap-2.5 pt-1">
					<button 
						type="button" 
						class="flex-1 py-3 rounded-2xl bg-white/[0.03] hover:bg-white/[0.06] text-white/60 hover:text-white font-bold text-xs transition-all cursor-pointer border border-white/[0.06]"
						onclick={() => showMsClientIdModal = false}
					>
						Cancelar
					</button>
					<button 
						type="button" 
						class="flex-1 py-3 rounded-2xl bg-gradient-to-r from-amber-500 to-amber-400 hover:from-amber-400 hover:to-amber-300 text-black font-black text-xs transition-all cursor-pointer shadow-[0_4px_16px_rgba(245,158,11,0.25)] hover:shadow-[0_8px_24px_rgba(245,158,11,0.35)] disabled:opacity-50 flex items-center justify-center gap-1.5"
						onclick={saveAndLoginWithClientId}
						disabled={isSavingClientId}
					>
						{#if isSavingClientId}
							<Loader2 class="w-3.5 h-3.5 animate-spin" />
							Salvando...
						{:else}
							<CheckCircle2 class="w-3.5 h-3.5" />
							Salvar e Conectar
						{/if}
					</button>
				</div>
			</div>
		</div>
	{/if}

{:else}

	<div class="flex gap-8 h-full w-full select-none" in:fade={{ duration: 100 }}>
		
		<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-6">
			<!-- SKLauncher-style Top Header Bar -->
			<header class="flex items-center justify-between gap-4 py-1">
				<button 
					type="button"
					onclick={() => {
						window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true }));
					}}
					class="flex items-center gap-2.5 px-4 py-2.5 rounded-2xl bg-[#141518]/90 hover:bg-[#1c1d22] border border-white/5 hover:border-white/15 text-xs text-white/40 hover:text-white transition-all shadow-md group cursor-pointer"
					title="Paleta de Comandos (Ctrl+K)"
				>
					<Search class="w-3.5 h-3.5 text-[#caa97c] group-hover:scale-110 transition-transform" />
					<span class="font-medium hidden sm:inline">Buscar instâncias, mods ou ações...</span>
					<span class="font-medium sm:hidden">Buscar...</span>
					<kbd class="text-[10px] font-mono text-white/20 bg-white/5 border border-white/10 px-1.5 py-0.5 rounded-md">Ctrl+K</kbd>
				</button>

				<div class="flex items-center gap-2.5">
					<div class="flex items-center gap-2.5 px-3.5 py-1.5 rounded-2xl bg-[#141518]/90 border border-white/5 shadow-md">
						<div class="w-8 h-8 rounded-xl overflow-hidden bg-black/40 border border-white/10 shrink-0">
							<img 
								src={activeSkinStore.current.avatarUrl || (account.value ? "https://mc-heads.net/avatar/" + account.value.uuid + "/64" : "/logo.png")} 
								alt="Avatar" 
								class="w-full h-full object-cover"
							/>
						</div>
						<div class="text-left hidden sm:block">
							<div class="text-xs font-bold text-white leading-tight">
								{account.value?.username || "Jogador"}
							</div>
							<div class="text-[10px] font-semibold leading-tight mt-0.5">
								{#if !account.value?.minecraftToken || account.value?.id.startsWith("offline_") || account.value?.id.startsWith("offline-")}
									<span class="text-sky-400">Offline</span>
								{:else}
									<span class="text-emerald-400">Microsoft</span>
								{/if}
							</div>
						</div>
					</div>
				</div>
			</header>

			{#if isLoadingHome}
				<div class="space-y-7 animate-pulse">
					<div class="h-28 rounded-3xl bg-[#18191c] border border-white/5 p-6 flex items-center justify-between">
						<div class="space-y-2.5 w-1/3">
							<div class="h-4 w-28 bg-white/10 rounded-full"></div>
							<div class="h-7 w-52 bg-white/10 rounded-lg"></div>
							<div class="h-3 w-40 bg-white/5 rounded-full"></div>
						</div>
						<div class="h-14 w-44 bg-[#caa97c]/20 rounded-2xl"></div>
					</div>

					<div>
						<div class="h-4 w-44 bg-white/10 rounded-full mb-3"></div>
						<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
							{#each [1, 2, 3] as _}
								<div class="h-60 rounded-2xl bg-[#18191c] border border-white/5 p-4 flex flex-col justify-between">
									<div class="h-28 w-full bg-white/5 rounded-xl"></div>
									<div class="space-y-2 mt-3">
										<div class="h-4 w-3/4 bg-white/10 rounded"></div>
										<div class="h-3 w-full bg-white/5 rounded"></div>
									</div>
									<div class="h-3 w-1/2 bg-white/5 rounded mt-3"></div>
								</div>
							{/each}
						</div>
					</div>

					<div class="grid grid-cols-1 xl:grid-cols-12 gap-6">
						<div class="xl:col-span-6 space-y-2">
							<div class="h-4 w-32 bg-white/10 rounded-full mb-3"></div>
							{#each [1, 2, 3, 4, 5] as _}
								<div class="h-14 rounded-2xl bg-[#18191c] border border-white/5 p-3 flex items-center gap-3">
									<div class="h-10 w-10 rounded-xl bg-white/10 shrink-0"></div>
									<div class="space-y-1.5 flex-1">
										<div class="h-3.5 w-32 bg-white/10 rounded"></div>
										<div class="h-2.5 w-20 bg-white/5 rounded"></div>
									</div>
									<div class="h-4 w-12 bg-white/10 rounded-full"></div>
								</div>
							{/each}
						</div>
						<div class="xl:col-span-6">
							<div class="h-4 w-36 bg-white/10 rounded-full mb-3"></div>
							<div class="h-80 rounded-2xl bg-[#18191c] border border-white/5 p-5 flex flex-col justify-between">
								<div class="space-y-2">
									<div class="h-8 w-24 bg-white/10 rounded-lg"></div>
									<div class="h-3 w-32 bg-white/5 rounded"></div>
								</div>
								<div class="h-28 w-full bg-white/5 rounded-xl"></div>
								<div class="grid grid-cols-3 gap-2 pt-2">
									<div class="h-8 bg-white/5 rounded"></div>
									<div class="h-8 bg-white/5 rounded"></div>
									<div class="h-8 bg-white/5 rounded"></div>
								</div>
							</div>
						</div>
					</div>

					<div>
						<div class="h-4 w-28 bg-white/10 rounded-full mb-3"></div>
						<div class="h-44 rounded-2xl bg-[#18191c] border border-white/5"></div>
					</div>
				</div>

			{:else}

				<div class="grid grid-cols-1 xl:grid-cols-12 gap-6 items-start">
					{#each layoutStore.sections as sec (sec.id)}
						{#if sec.enabled}
							<div class="{sec.width === 'half' ? 'xl:col-span-6' : 'xl:col-span-12'} w-full transition-all duration-300">
							{#if sec.id === 'hero'}
								<!-- SKLauncher-inspired Hero Banner & Launch Block -->
								<div class="relative rounded-3xl bg-[#111216] border border-white/[0.08] shadow-[0_24px_48px_rgba(0,0,0,0.4)] overflow-hidden" in:fade={{ duration: 300 }}>
									{#if activeInstance?.banner}
										<div 
											class="absolute inset-0 bg-cover bg-center opacity-20 scale-105 blur-sm pointer-events-none"
											style="background-image: url('{activeInstance.banner}');"
										></div>
									{/if}
									<div class="absolute inset-0 bg-gradient-to-r from-[#0e0f12] via-[#111216]/95 to-[#111216]/80 pointer-events-none"></div>
									<div class="absolute -right-16 -top-16 w-72 h-72 bg-[#caa97c]/[0.07] rounded-full blur-[100px] pointer-events-none"></div>
									<div class="absolute -left-16 -bottom-16 w-56 h-56 bg-[#6c5ce7]/[0.05] rounded-full blur-[80px] pointer-events-none"></div>

										<div class="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6 p-7 relative z-10">
											<div class="space-y-2.5 max-w-xl">
												<div class="flex items-center gap-2">
													<span class="w-2.5 h-2.5 rounded-full bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.9)] animate-pulse"></span>
													<span class="text-[11px] font-black uppercase tracking-widest text-[#caa97c]">{t("home.readyToPlay")}</span>
												</div>
												
												<div class="relative">
													{#if activeInstance}
														<button 
															type="button"
															onclick={() => showQuickInstancePicker = !showQuickInstancePicker}
															class="flex items-center gap-3 text-left group cursor-pointer"
															title="Clique para alternar a instância"
														>
															<h1 class="text-2xl sm:text-3xl font-black text-white tracking-tight group-hover:text-[#caa97c] transition-colors">
																{activeInstance.name}
															</h1>
															<div class="p-1 rounded-lg bg-white/5 border border-white/10 text-white/50 group-hover:text-white transition-colors">
																<ChevronDown class="w-4 h-4 transition-transform {showQuickInstancePicker ? 'rotate-180' : ''}" />
															</div>
														</button>
													{:else}
														<h1 class="text-2xl sm:text-3xl font-black text-white tracking-tight">
															{t("home.helloUser", { name: account.value?.username || 'Jogador' })}
														</h1>
													{/if}

													{#if showQuickInstancePicker && profiles.list.length > 0}
														<div 
															class="absolute left-0 top-full mt-2 w-80 max-h-64 overflow-y-auto custom-scrollbar rounded-2xl bg-[#1c1d22] border border-white/10 shadow-2xl p-2 z-50 space-y-1"
														>
															<div class="text-[10px] font-bold text-white/40 uppercase tracking-wider px-2.5 py-1">
																Alternar Instância
															</div>
															{#each profiles.list as p}
																<button 
																	type="button"
																	onclick={() => {
																		profiles.activeId = p.id;
																		showQuickInstancePicker = false;
																	}}
																	class="w-full flex items-center justify-between p-2 rounded-xl text-left transition-all cursor-pointer {p.id === activeInstance?.id ? 'bg-[#caa97c]/15 text-white border border-[#caa97c]/30' : 'hover:bg-white/5 text-white/80'}"
																>
																	<div class="flex items-center gap-2.5 min-w-0">
																		<div class="w-8 h-8 rounded-lg bg-black/40 border border-white/10 overflow-hidden flex items-center justify-center shrink-0">
																			{#if p.icon && (p.icon.startsWith("http") || p.icon.startsWith("/") || p.icon.startsWith("data:"))}
																				<img src={p.icon} alt={p.name} class="w-full h-full object-cover" />
																			{:else}
																				<img src="/grass_block.png" alt={p.name} class="w-5 h-5 object-contain [image-rendering:pixelated]" />
																			{/if}
																		</div>
																		<div class="min-w-0">
																			<div class="text-xs font-bold truncate">{p.name}</div>
																			<div class="text-[10px] text-white/40 font-mono">{p.mcVersion}</div>
																		</div>
																	</div>
																	<span class="text-[9px] font-bold font-mono uppercase px-1.5 py-0.5 rounded border {getLoaderColor(p.loader)}">
																		{p.loader}
																	</span>
																</button>
															{/each}
														</div>
													{/if}
												</div>

												<div class="flex flex-wrap items-center gap-2 text-xs text-white/60">
													{#if activeInstance}
														<span class="font-mono text-[11px] bg-white/5 px-2.5 py-1 rounded-lg border border-white/10 text-white/90 font-medium">
															{activeInstance.mcVersion}
														</span>
														<span class="font-mono text-[11px] px-2.5 py-1 rounded-lg border font-bold uppercase {getLoaderColor(activeInstance.loader)}">
															{activeInstance.loader}
														</span>
														{#if gamingStats.formattedTotalTime}
															<span class="flex items-center gap-1.5 text-[11px] text-white/50 bg-white/5 px-2.5 py-1 rounded-lg border border-white/5 font-mono">
																<Clock class="w-3.5 h-3.5 text-[#caa97c]" />
																{gamingStats.formattedTotalTime}
															</span>
														{/if}
													{:else}
														<span class="text-white/40">{t("home.noInstanceYet")}</span>
													{/if}
												</div>
											</div>

											<div class="flex items-center gap-3 w-full lg:w-auto shrink-0">
												<button
													type="button"
													class="flex-1 lg:flex-initial h-16 px-10 rounded-2xl bg-gradient-to-r from-[#d8bc98] via-[#caa97c] to-[#b89560] hover:from-[#e5cca8] hover:to-[#caa97c] text-[#111215] font-black text-sm uppercase tracking-wider flex items-center justify-center gap-3.5 shadow-[0_8px_30px_rgba(202,169,124,0.35)] hover:shadow-[0_12px_45px_rgba(202,169,124,0.55)] hover:scale-[1.02] active:scale-[0.98] transition-all cursor-pointer disabled:opacity-70 disabled:cursor-not-allowed group"
													onclick={() => handleHomePlay()}
													disabled={isLaunching}
												>
													{#if isLaunching}
														<Loader2 class="w-5 h-5 animate-spin" />
														<span>{launchStatusText || t("home.starting").toUpperCase()}</span>
													{:else}
														<Play class="w-5 h-5 fill-current group-hover:scale-110 transition-transform" />
														<span>{t("home.playNow")}</span>
													{/if}
												</button>

												<a 
													href="/instances" 
													class="h-16 px-5 rounded-2xl bg-[#18191c] hover:bg-[#202127] border border-white/10 hover:border-white/20 text-white font-bold text-xs flex items-center gap-2.5 transition-all shadow-md group cursor-pointer shrink-0"
													title={t("home.library")}
												>
													<Boxes class="w-4 h-4 text-[#caa97c] group-hover:scale-110 transition-transform" />
													<span class="hidden sm:inline">{t("home.library")}</span>
												</a>

												<button 
													type="button"
													onclick={() => showGamerCardModal = true}
													class="h-16 px-4 rounded-2xl bg-[#18191c] hover:bg-[#202127] border border-white/10 hover:border-[#caa97c]/40 text-white font-bold text-xs flex items-center gap-2 transition-all shadow-md group cursor-pointer shrink-0"
													title="Compartilhar Card de Gamer (estilo Spotify Wrapped / Discord)"
												>
													<Share2 class="w-4 h-4 text-[#caa97c] group-hover:scale-110 transition-transform" />
													<span class="hidden md:inline">Gamer Card</span>
												</button>
											</div>
										</div>
									</div>

								{:else if sec.id === 'quickInstances'}
									<!-- Quick Instances Carousel / Bar -->
									<section>
										<div class="flex items-center justify-between mb-4">
											<h2 class="text-xs font-bold text-white uppercase tracking-widest flex items-center gap-2.5">
												<Boxes class="w-3.5 h-3.5 text-[#caa97c]" />
												Minhas Instâncias
											</h2>
											<a href="/instances" class="text-xs text-[#caa97c] hover:text-[#e0c49a] underline-offset-2 hover:underline font-bold transition-colors">Ver Todas</a>
										</div>

										{#if profiles.list.length === 0}
											<div class="rounded-3xl bg-[#111216] border border-white/[0.06] p-8 flex flex-col items-center justify-center text-center">
												<p class="text-xs text-white/40 mb-4">Nenhuma instância criada ainda.</p>
												<a href="/instances" class="px-5 py-2.5 rounded-2xl bg-gradient-to-r from-[#d8bc98] via-[#caa97c] to-[#b89560] text-[#111215] text-xs font-black hover:shadow-[0_8px_24px_rgba(202,169,124,0.3)] transition-all">
													Criar Nova Instância
												</a>
											</div>
										{:else}
											<div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
												{#each profiles.list.slice(0, 4) as inst}
													{@const isSelected = activeInstance?.id === inst.id}
													<div 
														class="flex items-center justify-between p-3.5 rounded-2xl bg-[#111216] border {isSelected ? 'border-[#caa97c]/40 bg-gradient-to-br from-[#caa97c]/[0.08] to-transparent shadow-[0_0_24px_rgba(202,169,124,0.1)]' : 'border-white/[0.06] hover:border-white/[0.12]'} transition-all duration-300 cursor-pointer group"
														onclick={() => { profiles.activeId = inst.id; }}
														role="button"
														tabindex="0"
														onkeydown={(e) => { if (e.key === 'Enter') profiles.activeId = inst.id; }}
													>
														<div class="flex items-center gap-3 min-w-0">
															<div class="h-10 w-10 rounded-xl bg-[#0e0f12] border border-white/[0.08] overflow-hidden flex items-center justify-center shrink-0">
																{#if inst.icon && (inst.icon.startsWith("http") || inst.icon.startsWith("/") || inst.icon.startsWith("data:"))}
																	<img src={inst.icon} alt={inst.name} class="w-full h-full object-cover" />
																{:else}
																	<img src="/grass_block.png" alt={inst.name} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
																{/if}
															</div>
															<div class="min-w-0">
																<h4 class="font-bold text-white text-xs truncate group-hover:text-[#caa97c] transition-colors">{inst.name}</h4>
																<div class="flex items-center gap-1.5 text-[10px] text-white/35 font-mono mt-0.5">
																	<span>{inst.mcVersion}</span>
																	<span>·</span>
																	<span class="uppercase text-[#caa97c]/70 font-bold">{inst.loader}</span>
																</div>
															</div>
														</div>
														<div class="shrink-0 ml-2">
															<button
																type="button"
																class="h-8 w-8 rounded-xl flex items-center justify-center {isSelected ? 'bg-gradient-to-r from-[#d8bc98] to-[#caa97c] text-[#111215]' : 'bg-white/[0.04] hover:bg-white/[0.08] text-white/60'} transition-all duration-300"
																title="Jogar esta instância"
																onclick={(e) => {
																	e.stopPropagation();
																	profiles.activeId = inst.id;
																	handleHomePlay();
																}}
															>
																<Play class="w-3.5 h-3.5 fill-current" />
															</button>
														</div>
													</div>
												{/each}
											</div>
										{/if}
									</section>

								{:else if sec.id === 'favoriteServer'}
									<!-- Servidor Favorito & Ping -->
									<section>
										<FavoriteServerWidget onQuickJoin={(host, port) => handleQuickServerJoin(host, port)} />
									</section>

								{:else if sec.id === 'curatedPacks'}
									<!-- Modpacks Recomendados -->
									<section>
										<div class="flex items-center justify-between mb-4">
											<h2 class="text-xs font-bold text-white uppercase tracking-widest">{t("home.exploreContent")}</h2>
											<a href="/mods" class="text-xs text-[#caa97c] hover:text-[#e0c49a] underline-offset-2 hover:underline font-bold transition-colors">{t("home.viewAll")}</a>
										</div>

										<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
											{#each modpacks as pack}
												<a href="/mods" class="group rounded-3xl bg-[#111216] border border-white/[0.06] overflow-hidden hover:border-[#caa97c]/30 hover:-translate-y-1 hover:shadow-[0_24px_48px_rgba(0,0,0,0.5)] transition-all duration-300 cursor-pointer flex flex-col justify-between">
													<div class="h-36 w-full relative bg-[#0e0f12] overflow-hidden">
														<img src={pack.bgImg} class="w-full h-full object-cover opacity-80 group-hover:scale-105 transition-transform duration-500" alt={pack.title} />
														<div class="absolute inset-0 bg-gradient-to-t from-[#111216] via-transparent to-transparent"></div>
														
														<div class="absolute top-3 right-3 h-7 w-7 rounded-xl bg-black/60 border border-white/[0.12] flex items-center justify-center shadow-md">
															<span class="font-black text-[10px] text-emerald-400">m</span>
														</div>

														<div class="absolute bottom-3 left-3 h-10 w-10 rounded-xl overflow-hidden bg-black/60 border border-white/[0.12] flex items-center justify-center shadow-md p-0.5">
															<img src={pack.iconImg} alt={pack.title} class="w-full h-full object-cover rounded-lg" />
														</div>
													</div>
													<div class="p-5 pt-3 flex-1 flex flex-col justify-between">
														<div>
															<h3 class="font-extrabold text-white text-xs truncate group-hover:text-[#caa97c] transition-colors">{pack.title}</h3>
															<p class="text-[10px] text-white/35 mt-1.5 line-clamp-2 leading-relaxed">{pack.subtitle}</p>
														</div>
														<div class="flex justify-between items-center mt-3 pt-2.5 border-t border-white/[0.04] text-[10px] font-medium text-white/35">
															<span class="flex items-center gap-1"><Users class="w-3 h-3 text-white/25"/> {pack.author}</span>
															<span class="flex items-center gap-1 font-mono"><Download class="w-3 h-3 text-white/25"/> {pack.downloads}</span>
														</div>
													</div>
												</a>
											{/each}
										</div>
									</section>

								{:else if sec.id === 'gamingStats'}
									<!-- Tempo de Jogo & Monitor -->
									<section class="flex flex-col">
										<div class="flex items-center justify-between mb-4">
											<h2 class="text-xs font-bold text-white uppercase tracking-widest flex items-center gap-2">
												{t("home.yourPlayTime")}
											</h2>
											<span class="text-[10px] text-white/30 font-mono">{t("home.realTimeTracking")}</span>
										</div>

										<div class="bg-[#111216] border border-white/[0.06] rounded-3xl p-6 shadow-sm flex flex-col justify-between hover:border-white/[0.12] transition-all duration-300">
											<div class="mb-4">
												<div class="text-2xl font-black text-white">{gamingStats.formattedLast7DaysTime || "0m"}</div>
												<div class="text-[11px] text-white/35 mt-0.5">{t("home.last7Days")}</div>
											</div>

											<div class="my-4 py-3 border-y border-white/[0.04] relative flex flex-col items-center justify-center">
												<div class="h-24 w-full flex items-end justify-between gap-2 px-1 pt-2">
													{#each gamingStats.last7Days as day}
														{@const heightPercent = gamingStats.maxMinutesInLast7 > 0 ? Math.max(8, Math.round((day.minutes / gamingStats.maxMinutesInLast7) * 100)) : 8}
														<div class="flex-1 flex flex-col items-center gap-1 group relative h-full justify-end">
															<div class="absolute -top-6 opacity-0 group-hover:opacity-100 transition-opacity bg-[#0e0f12] border border-white/[0.08] text-[10px] text-white font-mono px-1.5 py-0.5 rounded shadow-lg whitespace-nowrap pointer-events-none z-10">
																{day.formattedTime}
															</div>
															<div class="w-full bg-white/[0.03] rounded-t-sm h-20 flex items-end overflow-hidden">
																<div
																	class="w-full rounded-t-sm transition-all duration-500 {day.isToday ? 'bg-gradient-to-t from-[#b89560] to-[#caa97c] shadow-[0_0_12px_rgba(202,169,124,0.25)]' : (day.minutes > 0 ? 'bg-white/30 group-hover:bg-white/50' : 'bg-white/[0.06]')}"
																	style="height: {day.minutes > 0 ? heightPercent + '%' : '6%'};"
																></div>
															</div>
														</div>
													{/each}
												</div>

												<div class="w-full flex justify-between items-center text-[10px] text-white/35 font-medium mt-3 pt-2 border-t border-white/[0.04] px-2">
													{#each gamingStats.last7Days as day}
														<span class="{day.isToday ? 'text-white font-bold' : ''}">{day.dayLabel}</span>
													{/each}
												</div>
											</div>

											<div class="grid grid-cols-3 gap-2 pt-2 text-center">
												<div class="text-left">
													<div class="text-xs font-black text-white">{gamingStats.formattedAverageSession || "0m"}</div>
													<div class="text-[10px] text-white/35 mt-0.5">{t("home.averageSession")}</div>
												</div>
												<div class="text-left">
													<div class="text-xs font-black text-white">{gamingStats.formattedLongestSession || "0m"}</div>
													<div class="text-[10px] text-white/35 mt-0.5">{t("home.longestSession")}</div>
												</div>
												<div class="text-left">
													<div class="text-xs font-black text-white">{gamingStats.daysPlayedInLast7} de 7</div>
													<div class="text-[10px] text-white/35 mt-0.5">{t("home.daysPlayed")}</div>
												</div>
											</div>
										</div>
									</section>

							{:else if sec.id === 'friendsRadar'}
								<FriendsRadarWidget />

							{:else if sec.id === 'newsFeed'}
								<NewsFeedWidget />

							{:else if sec.id === 'screenshots'}
								<ScreenshotsWidget />

							{:else if sec.id === 'tools'}
									<!-- Ferramentas Rápidas & Organizador -->
									<section>
										<div class="flex items-center justify-between mb-4">
											<h2 class="text-xs font-bold text-white uppercase tracking-widest flex items-center gap-2.5">
												<Zap class="w-3.5 h-3.5 text-[#caa97c]" />
												Atalhos & Ferramentas Rápidas
											</h2>
											<a href="/organizer" class="text-xs text-[#caa97c] hover:text-[#e0c49a] underline-offset-2 hover:underline font-bold flex items-center gap-1 transition-colors">
												<SlidersHorizontal class="w-3 h-3" />
												Personalizar
											</a>
										</div>

										<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-3">
											<button 
												type="button"
												onclick={() => goto("/organizer")}
												class="p-4 rounded-2xl bg-[#111216] border border-white/[0.06] hover:border-[#caa97c]/30 hover:bg-gradient-to-br hover:from-[#caa97c]/[0.05] hover:to-transparent text-left transition-all duration-300 group cursor-pointer shadow-sm flex flex-col justify-between h-28"
											>
												<div class="h-9 w-9 rounded-xl bg-[#caa97c]/10 border border-[#caa97c]/15 flex items-center justify-center text-[#caa97c] group-hover:scale-110 transition-transform duration-300">
													<SlidersHorizontal class="w-4 h-4" />
												</div>
												<div>
													<h4 class="font-bold text-white text-xs group-hover:text-[#caa97c] transition-colors">Organizador</h4>
													<p class="text-[10px] text-white/35 mt-0.5">Ajustar Layout</p>
												</div>
											</button>

											<button 
												type="button"
												onclick={() => showGamerCardModal = true}
												class="p-4 rounded-2xl bg-[#111216] border border-white/[0.06] hover:border-purple-500/30 hover:bg-gradient-to-br hover:from-purple-500/[0.05] hover:to-transparent text-left transition-all duration-300 group cursor-pointer shadow-sm flex flex-col justify-between h-28"
											>
												<div class="h-9 w-9 rounded-xl bg-purple-500/10 border border-purple-500/15 flex items-center justify-center text-purple-400 group-hover:scale-110 transition-transform duration-300">
													<Share2 class="w-4 h-4" />
												</div>
												<div>
													<h4 class="font-bold text-white text-xs group-hover:text-purple-300 transition-colors">Card de Gamer</h4>
													<p class="text-[10px] text-white/35 mt-0.5">Exportar Imagem</p>
												</div>
											</button>

											<button 
												type="button"
												onclick={() => openInstanceFolder()}
												class="p-4 rounded-2xl bg-[#111216] border border-white/[0.06] hover:border-white/[0.15] hover:bg-gradient-to-br hover:from-white/[0.03] hover:to-transparent text-left transition-all duration-300 group cursor-pointer shadow-sm flex flex-col justify-between h-28"
											>
												<div class="h-9 w-9 rounded-xl bg-white/[0.04] border border-white/[0.08] flex items-center justify-center text-white/60 group-hover:scale-110 transition-transform duration-300">
													<FolderOpen class="w-4 h-4" />
												</div>
												<div>
													<h4 class="font-bold text-white text-xs group-hover:text-white transition-colors">Pasta do Jogo</h4>
													<p class="text-[10px] text-white/35 mt-0.5">Abrir .minecraft</p>
												</div>
											</button>

											<button 
												type="button"
												onclick={() => goto("/mods")}
												class="p-4 rounded-2xl bg-[#111216] border border-white/[0.06] hover:border-emerald-500/30 hover:bg-gradient-to-br hover:from-emerald-500/[0.05] hover:to-transparent text-left transition-all duration-300 group cursor-pointer shadow-sm flex flex-col justify-between h-28"
											>
												<div class="h-9 w-9 rounded-xl bg-emerald-500/10 border border-emerald-500/15 flex items-center justify-center text-emerald-400 group-hover:scale-110 transition-transform duration-300">
													<Package class="w-4 h-4" />
												</div>
												<div>
													<h4 class="font-bold text-white text-xs group-hover:text-white transition-colors">Mods & Shaders</h4>
													<p class="text-[10px] text-white/35 mt-0.5">CurseForge / Modrinth</p>
												</div>
											</button>

											<button 
												type="button"
												onclick={() => goto("/screenshots")}
												class="p-4 rounded-2xl bg-[#111216] border border-white/[0.06] hover:border-sky-500/30 hover:bg-gradient-to-br hover:from-sky-500/[0.05] hover:to-transparent text-left transition-all duration-300 group cursor-pointer shadow-sm flex flex-col justify-between h-28"
											>
												<div class="h-9 w-9 rounded-xl bg-sky-500/10 border border-sky-500/15 flex items-center justify-center text-sky-400 group-hover:scale-110 transition-transform duration-300">
													<Camera class="w-4 h-4" />
												</div>
												<div>
													<h4 class="font-bold text-white text-xs group-hover:text-white transition-colors">Screenshots</h4>
													<p class="text-[10px] text-white/35 mt-0.5">Galeria de Fotos</p>
												</div>
											</button>
										</div>
									</section>
								{/if}
							</div>
						{/if}
					{/each}
				</div>

			{/if}

		</div>

		<RightSidebar />

	</div>

{/if}

<GamerCardModal
	isOpen={showGamerCardModal}
	onClose={() => showGamerCardModal = false}
/>