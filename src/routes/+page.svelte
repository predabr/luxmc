<script lang="ts">
	import { fade, slide } from "svelte/transition";
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
		AlertCircle
	} from "lucide-svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { 
		authDevLogin, 
		authOfflineLogin, 
		authLogin, 
		launchGame, 
		versionsCheckInstalled, 
		versionsDownload, 
		discordSetActivity 
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
	let savedAccounts = $state<string[]>([]);
	let isLoadingHome = $state(true);
	let isLaunching = $state(false);
	let launchStatusText = $state("");

	const activeInstance = $derived(profiles.active || profiles.list[0] || null);

	async function hashPassword(password: string): Promise<string> {
		const encoder = new TextEncoder();
		const data = encoder.encode(password + 'luxmc_salt_v1');
		const hashBuffer = await crypto.subtle.digest('SHA-256', data);
		const hashArray = Array.from(new Uint8Array(hashBuffer));
		return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
	}

	onMount(() => {
		const accountsMapRaw = localStorage.getItem("luxmc_offline_passwords");
		if (accountsMapRaw) {
			try {
				const map = JSON.parse(accountsMapRaw);
				savedAccounts = Object.keys(map);
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

			const newAcc = {
				id: backendAcc?.id || ("offline_" + Date.now()),
				username: name,
				uuid: backendAcc?.uuid || ("offline-" + key),
				minecraftToken: "",
				expiresAt: 0
			};
			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
			toast(t("home.welcomeToLuxmc", { name }), "success");

			account.value = newAcc;
		} catch (e) {
			toast(t("home.accountError", { error: String(e) }), "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleMicrosoftLogin() {
		isLoggingInMicrosoft = true;
		try {
			toast(t("home.msAuthStarted"), "info");
			const acc = await authLogin();
			const newAcc = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken,
				expiresAt: acc.expiresAt ? (acc.expiresAt < 1e11 ? acc.expiresAt * 1000 : acc.expiresAt) : 0
			};
			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
			toast(t("home.connectedAs", { username: acc.username }), "success");

			account.value = newAcc;
		} catch (e) {
			const errStr = String(e);
			if (errStr.includes("00000000-0000-0000-0000-000000000000") || errStr.includes("invalid_client") || errStr.includes("AADSTS700016")) {
				toast(t("home.msApprovalWaiting"), "info");
			} else {
				toast(t("home.loginAttempt", { error: errStr }), "error");
			}
		} finally {
			isLoggingInMicrosoft = false;
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
			const newAcc = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken,
				expiresAt: acc.expiresAt ? (acc.expiresAt < 1e11 ? acc.expiresAt * 1000 : acc.expiresAt) : 0
			};
			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
			toast(t("home.connectedAsDev", { username: acc.username }), "success");
			account.value = newAcc;
		} catch (e) {
			toast(t("home.devLoginErrorToast", { error: String(e) }), "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleHomePlay() {
		if (isLaunching) return;

		const targetProfile = activeInstance;
		if (!targetProfile) {
			toast(t("home.noInstanceFound"), "info");
			goto("/instances?new=true");
			return;
		}

		isLaunching = true;
		launchStatusText = t("home.starting");

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

			launchStatusText = t("home.startingMc");
			const isVulkan = typeof window !== "undefined" ? localStorage.getItem("luxmc_enable_vulkan") !== "false" : true;
			const result = await launchGame({
				versionId: verId,
				accountId: userUuid || "",
				profileId: targetProfile.id,
				enableVulkan: isVulkan
			});

			gamingStats.onGameStart();
			discordSetActivity({
				inGame: true,
				details: `Jogando ${targetProfile.name}`,
				state: `Minecraft ${verId} · ${targetProfile.loader ? targetProfile.loader.toUpperCase() : "Vanilla"}`,
				largeText: `Minecraft ${verId}`,
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
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

	const servers = [
		{ 
			rank: "#1", 
			name: "Hypixel Network", 
			badge: "ORIGINAL", 
			online: "44.120", 
			version: "1.8.9 - 1.21.4", 
			logo: "https://mc-heads.net/head/MHF_Gold/100",
			ip: "mc.hypixel.net",
			ping: "18ms"
		},
		{ 
			rank: "#2", 
			name: "Mush MC", 
			badge: "BRASIL · PIRATA", 
			online: "8.950", 
			version: "1.8 - 1.21.4", 
			logo: "https://mc-heads.net/head/MHF_MushroomCow/100",
			ip: "mush.com.br",
			ping: "12ms"
		},
		{ 
			rank: "#3", 
			name: "2b2t Anarchy", 
			badge: "SEM REGRAS", 
			online: "1.050", 
			version: "1.20.4", 
			logo: "https://mc-heads.net/head/MHF_Obsidian/100",
			ip: "2b2t.org",
			ping: "45ms"
		},
		{ 
			rank: "#4", 
			name: "Complex Gaming", 
			badge: "PIXELMON", 
			online: "2.840", 
			version: "1.16 - 1.21.4", 
			logo: "https://mc-heads.net/head/MHF_Emerald/100",
			ip: "hub.mc-complex.com",
			ping: "24ms"
		},
		{ 
			rank: "#5", 
			name: "DonutSMP Hardcore", 
			badge: "LIFESTEAL", 
			online: "3.420", 
			version: "1.20.4 - 1.21.4", 
			logo: "https://mc-heads.net/head/MHF_TNT/100",
			ip: "donutsmp.net",
			ping: "31ms"
		}
	];
</script>

{#if !account.value}
	<div class="flex h-full w-full items-center justify-center px-4 transition-all duration-500 {isEntering ? 'opacity-0 scale-95 blur-md pointer-events-none' : 'opacity-100 scale-100'}" in:fade={{ duration: 300 }}>
		<div class="w-full max-w-md rounded-3xl bg-[#141518] border border-white/10 p-8 shadow-2xl flex flex-col items-center select-none relative overflow-hidden">
			<div class="absolute -top-12 -right-12 w-48 h-48 bg-[#caa97c]/10 rounded-full blur-3xl pointer-events-none"></div>
			<div class="absolute -bottom-12 -left-12 w-48 h-48 bg-[#6c5ce7]/10 rounded-full blur-3xl pointer-events-none"></div>

			<div class="flex flex-col items-center text-center">
				<div class="h-24 w-24 rounded-3xl bg-black/50 border border-white/10 flex items-center justify-center p-3 shadow-2xl relative group">
					<img src="/logo.png" alt="Luxmc 3D" class="w-full h-full object-contain drop-shadow-[0_0_24px_rgba(226,184,107,0.5)] group-hover:scale-105 transition-transform" />
				</div>
				<h1 class="text-2xl font-black text-white tracking-tight mt-4">Luxmc Launcher</h1>
				<p class="text-xs text-white/50 mt-1">{t("home.subTagline")}</p>
			</div>

			<div class="w-full space-y-4 mt-2">
				{#if isLoggingInMicrosoft}
					<div class="w-full bg-[#18191c] border-2 border-[#6c5ce7]/50 rounded-3xl p-5 shadow-2xl relative overflow-hidden space-y-4 animate-fade-in" in:fade={{ duration: 200 }}>
						<div class="absolute -right-8 -top-8 w-32 h-32 bg-[#6c5ce7]/20 rounded-full blur-2xl pointer-events-none"></div>

						<div class="flex items-center gap-3.5">
							<div class="h-11 w-11 rounded-2xl bg-[#6c5ce7]/20 border border-[#6c5ce7]/40 flex items-center justify-center shrink-0 shadow-inner">
								<Loader2 class="w-5 h-5 text-[#a29bfe] animate-spin" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-xs font-black text-white">{t("home.msAuth")}</h3>
									<span class="inline-flex items-center gap-1 text-[9px] font-extrabold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20">
										<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping"></span> {t("home.msWaiting")}
									</span>
								</div>
								<p class="text-[11px] text-white/50 mt-0.5">{t("home.msCompleteInBrowser")}</p>
							</div>
						</div>

						<div class="space-y-2 py-1">
							<div class="h-1.5 w-full bg-white/5 rounded-full overflow-hidden relative">
								<div class="h-full bg-gradient-to-r from-[#6c5ce7] to-[#a29bfe] w-2/3 rounded-full animate-pulse"></div>
							</div>
							<p class="text-[11px] text-white/60 leading-relaxed">
								{t("home.msAccessDesc")}
							</p>
						</div>

						<div class="flex items-center justify-between pt-2 border-t border-white/5">
							<span class="text-[10px] text-white/40 flex items-center gap-1.5">
								<ExternalLink class="w-3 h-3 text-[#a29bfe]" />
								{t("home.oauthWindowOpen")}
							</span>
							<button 
								type="button" 
								class="text-xs font-bold text-white/70 hover:text-white px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 transition-all duration-200 cursor-pointer"
								onclick={() => { isLoggingInMicrosoft = false; }}
							>
								{t("home.cancel")}
							</button>
						</div>
					</div>
				{:else}
					<div class="bg-gradient-to-b from-[#1c1d24] to-[#16171d] border-2 border-[#6c5ce7]/50 hover:border-[#6c5ce7] rounded-3xl p-4 shadow-[0_0_25px_rgba(108,92,231,0.2)] hover:shadow-[0_0_35px_rgba(108,92,231,0.35)] transition-all duration-300 relative group overflow-hidden" in:fade={{ duration: 300 }}>
						<div class="absolute -right-8 -bottom-8 w-28 h-28 bg-[#6c5ce7]/15 rounded-full blur-2xl pointer-events-none group-hover:bg-[#6c5ce7]/25 transition-all"></div>

						<button 
							type="button" 
							class="w-full h-16 rounded-2xl bg-gradient-to-r from-[#22242e] to-[#1c1e27] hover:from-[#2a2c39] hover:to-[#222530] border border-[#6c5ce7]/30 hover:border-[#6c5ce7]/60 text-white font-bold text-sm flex items-center justify-between px-4 transition-all duration-300 active:scale-[0.98] cursor-pointer shadow-xl disabled:opacity-60"
							onclick={handleMicrosoftLogin}
							disabled={isLoggingIn || isLoggingInMicrosoft}
						>
							<div class="flex items-center gap-3.5">
								<div class="h-11 w-11 rounded-2xl bg-black/60 border border-white/10 flex items-center justify-center p-2.5 shrink-0 shadow-inner">
									<svg class="h-full w-full" viewBox="0 0 24 24" fill="none">
										<rect x="1" y="1" width="10" height="10" fill="#f25022" rx="1"/>
										<rect x="13" y="1" width="10" height="10" fill="#7fba00" rx="1"/>
										<rect x="1" y="13" width="10" height="10" fill="#00a4ef" rx="1"/>
										<rect x="13" y="13" width="10" height="10" fill="#ffb900" rx="1"/>
									</svg>
								</div>
								<div class="text-left">
									<div class="flex items-center gap-2">
										<span class="font-black text-white text-sm leading-tight">{t("home.signInWithMicrosoft")}</span>
										<span class="bg-[#6c5ce7]/30 text-[#a29bfe] text-[9px] font-black px-2 py-0.5 rounded-full border border-[#6c5ce7]/40 uppercase tracking-wider">{t("home.primary")}</span>
									</div>
									<p class="text-[11px] text-white/50 font-medium mt-0.5">{t("home.msOfficialAccount")}</p>
								</div>
							</div>

							<div class="h-9 px-4 rounded-xl bg-[#6c5ce7] hover:bg-[#5b4cdb] text-white text-xs font-black flex items-center gap-2 shadow-lg shadow-[#6c5ce7]/30 group-hover:scale-105 transition-all duration-200 shrink-0">
								<span>{t("home.connect")}</span>
								<ArrowRight class="w-3.5 h-3.5" />
							</div>
						</button>
					</div>
				{/if}

			<div class="flex items-center gap-3 py-1">
				<div class="flex-1 h-[1px] bg-gradient-to-r from-transparent via-white/15 to-transparent"></div>
				<span class="text-[10px] font-bold text-white/30 uppercase tracking-wider">{t("home.orOffline")}</span>
				<div class="flex-1 h-[1px] bg-gradient-to-r from-transparent via-white/15 to-transparent"></div>
			</div>

			<div class="bg-[#18191c] border border-white/10 rounded-3xl p-5 space-y-4 shadow-lg">
					<div class="flex bg-[#121316] p-1 rounded-full border border-white/5">
						<button 
							type="button" 
							class="flex-1 py-2.5 rounded-full text-xs font-bold transition-all duration-300 cursor-pointer {authTab === 'login' ? 'bg-[#25262c] text-[#caa97c] shadow-sm border border-[#caa97c]/20 shadow-[0_0_12px_rgba(202,169,124,0.1)]' : 'text-white/40 hover:text-white/70 border border-transparent'}"
							onclick={() => authTab = 'login'}
						>
							{t("home.loginTab")}
						</button>
						<button 
							type="button" 
							class="flex-1 py-2.5 rounded-full text-xs font-bold transition-all duration-300 cursor-pointer {authTab === 'register' ? 'bg-[#25262c] text-[#caa97c] shadow-sm border border-[#caa97c]/20 shadow-[0_0_12px_rgba(202,169,124,0.1)]' : 'text-white/40 hover:text-white/70 border border-transparent'}"
							onclick={() => authTab = 'register'}
						>
							{t("home.createAccountTab")}
						</button>
					</div>

				{#if savedAccounts.length > 0 && authTab === 'login'}
					<div class="flex flex-wrap items-center gap-1.5 pt-0.5" transition:slide={{ duration: 200 }}>
						<span class="text-[10px] text-white/40 font-semibold shrink-0">{t("home.savedAccounts")}</span>
						{#each savedAccounts as accName (accName)}
							<button 
								type="button" 
								class="text-[10px] font-bold px-3 py-1.5 rounded-full border transition-all cursor-pointer hover:scale-105 active:scale-95 {offlineName.toLowerCase() === accName.toLowerCase() ? 'border-[#caa97c] bg-[#caa97c]/20 text-[#caa97c] shadow-[0_0_8px_rgba(202,169,124,0.2)]' : 'border-white/10 bg-white/5 text-white/60 hover:text-white hover:border-white/25 hover:bg-white/10'}"
								onclick={() => { offlineName = accName; }}
							>
								{accName}
							</button>
						{/each}
					</div>
				{/if}

					<div class="flex items-center gap-2 text-xs font-bold text-white/80">
						<Gamepad2 class="w-4 h-4 text-[#caa97c]" />
						<span>{authTab === 'login' ? t("home.accessOfflineAccount") : t("home.registerNewAccount")}</span>
					</div>

					<div class="space-y-3">
						<input 
							type="text" 
							placeholder={t("home.gamertagPlaceholder")} 
							bind:value={offlineName}
							class="w-full bg-[#1e1f24] border border-white/10 rounded-full px-5 py-3 text-xs font-bold text-white outline-none focus:border-[#caa97c] focus:shadow-[0_0_0_3px_rgba(202,169,124,0.1)] transition-all duration-200 placeholder:text-white/30"
							maxlength="16"
							onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
						/>

						<div class="relative">
							<input 
								type={showPassword ? "text" : "password"} 
								placeholder={t("home.accountPasswordPlaceholder")} 
								bind:value={offlinePassword}
								class="w-full bg-[#1e1f24] border border-white/10 rounded-full pl-5 pr-12 py-3 text-xs font-bold text-white outline-none focus:border-[#caa97c] focus:shadow-[0_0_0_3px_rgba(202,169,124,0.1)] transition-all duration-200 placeholder:text-white/30"
								onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
							/>
							<button
								type="button"
								class="absolute right-3.5 top-1/2 -translate-y-1/2 text-white/40 hover:text-[#caa97c] p-1.5 transition-all duration-200 cursor-pointer"
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
								bind:value={offlineConfirmPassword}
								class="w-full bg-[#1e1f24] border border-white/10 rounded-full px-5 py-3 text-xs font-bold text-white outline-none focus:border-[#caa97c] focus:shadow-[0_0_0_3px_rgba(202,169,124,0.1)] transition-all duration-200 placeholder:text-white/30"
								onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
							/>
						</div>
					{/if}
					</div>

					<button 
						type="button" 
						class="w-full h-12 rounded-xl hover:brightness-110 active:scale-[0.98] text-[#15171c] font-bold text-xs flex items-center justify-center gap-2 transition-all duration-200 shadow-md hover:shadow-lg cursor-pointer disabled:opacity-60 disabled:cursor-not-allowed"
						style="background-color: #caa97c;"
						onclick={handleOfflineAuth}
						disabled={isLoggingIn || isLoggingInMicrosoft}
					>
						{#if isLoggingIn}
							<Loader2 class="w-4 h-4 animate-spin" /> {t("home.processing")}
						{:else}
							<Play class="w-4 h-4 fill-current" /> {authTab === 'login' ? t("home.loginAndPlay") : t("home.createAndPlay")}
						{/if}
					</button>
				</div>

				<div class="flex flex-col items-center gap-2 pt-2 mt-2">
					<button 
						type="button" 
						class="group px-4 py-2.5 rounded-2xl bg-white/[0.03] hover:bg-white/[0.08] border border-white/10 hover:border-amber-500/40 text-white/60 hover:text-white transition-all duration-300 flex items-center gap-2.5 cursor-pointer text-xs disabled:opacity-50 hover:shadow-[0_0_16px_rgba(245,158,11,0.08)]"
						onclick={handleDevLogin}
						disabled={isLoggingIn || isLoggingInMicrosoft}
					>
						<span class="bg-amber-500/20 text-amber-300 text-[9px] font-black px-2 py-0.5 rounded-full border border-amber-500/30 uppercase tracking-wider">
							{t("home.devMode")}
						</span>
						<span class="font-semibold text-[11px] text-white/70 group-hover:text-white transition-colors">{t("home.devAccess")}</span>
						{#if isLoggingIn}
							<Loader2 class="w-3.5 h-3.5 animate-spin text-amber-400" />
						{/if}
					</button>
					<span class="text-[9px] text-white/30 text-center">
						{t("home.devDesc")}
					</span>
				</div>
			</div>

			<div class="text-[10px] text-white/30 text-center leading-relaxed">
				{t("home.compatibleWith")}
			</div>
		</div>
	</div>

{:else}

	<div class="flex gap-8 h-full w-full select-none" in:fade={{ duration: 100 }}>
		
		<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-7">
			
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

			<div class="relative rounded-3xl bg-gradient-to-r from-[#1c1d24] via-[#18191f] to-[#15161b] border border-white/10 p-6 shadow-xl" in:fade={{ duration: 250 }}>
				<div class="absolute -right-12 -top-12 w-56 h-56 bg-[#caa97c]/15 rounded-full blur-3xl pointer-events-none"></div>

					<div class="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-5 relative z-10">
						<div class="space-y-1.5">
							<div class="flex items-center gap-2">
								<span class="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]"></span>
								<span class="text-[11px] font-bold uppercase tracking-widest text-[#caa97c]">{t("home.readyToPlay")}</span>
							</div>
							<h1 class="text-2xl lg:text-3xl font-black text-white tracking-tight">
								{t("home.helloUser", { name: account.value?.username || 'Jogador' })}
							</h1>
							<div class="flex flex-wrap items-center gap-2.5 text-xs text-white/60 pt-0.5">
								{#if activeInstance}
									<div class="flex items-center gap-2">
										<div class="h-6 w-6 rounded-lg bg-black/40 border border-white/10 overflow-hidden flex items-center justify-center shrink-0">
											{#if activeInstance.icon && (activeInstance.icon.startsWith("http") || activeInstance.icon.startsWith("/") || activeInstance.icon.startsWith("data:"))}
												<img src={activeInstance.icon} alt={activeInstance.name} class="w-full h-full object-cover" />
											{:else}
												<img src="/grass_block.png" alt={activeInstance.name} class="w-4 h-4 object-contain [image-rendering:pixelated]" />
											{/if}
										</div>
										<span class="font-bold text-white text-xs">{activeInstance.name}</span>
									</div>
									<span class="text-white/30">•</span>
									<span class="font-mono text-[10px] bg-white/5 px-2 py-0.5 rounded-md border border-white/10 text-white/80">
										{activeInstance.mcVersion}
									</span>
									<span class="font-mono text-[10px] bg-[#caa97c]/15 text-[#caa97c] px-2 py-0.5 rounded-md border border-[#caa97c]/25 uppercase font-bold">
										{activeInstance.loader}
									</span>
								{:else}
									<span class="text-white/40">{t("home.noInstanceYet")}</span>
								{/if}
							</div>
						</div>

						<div class="flex items-center gap-3 w-full lg:w-auto shrink-0">
							<button
								type="button"
								class="flex-1 lg:flex-initial h-14 px-8 rounded-2xl bg-gradient-to-r from-[#e2b86b] via-[#caa97c] to-[#b89560] hover:from-[#ebd08f] hover:to-[#c4a16b] text-[#121316] font-black text-xs uppercase tracking-wider flex items-center justify-center gap-3 shadow-[0_4px_25px_rgba(202,169,124,0.35)] hover:shadow-[0_6px_32px_rgba(202,169,124,0.5)] active:scale-[0.98] transition-all cursor-pointer disabled:opacity-70 disabled:cursor-not-allowed"
								onclick={handleHomePlay}
								disabled={isLaunching}
							>
								{#if isLaunching}
									<Loader2 class="w-5 h-5 animate-spin" />
									<span>{launchStatusText || t("home.starting").toUpperCase()}</span>
								{:else}
									<Play class="w-5 h-5 fill-current" />
									<span>{t("home.playNow")}</span>
								{/if}
							</button>

							<a 
								href="/instances" 
								class="h-14 px-5 rounded-2xl bg-[#18191c] hover:bg-[#222329] border border-white/10 hover:border-white/25 text-white font-bold text-xs flex items-center gap-2.5 transition-all shadow-md group cursor-pointer shrink-0"
								title={t("home.library")}
							>
								<Boxes class="w-4 h-4 text-[#caa97c] group-hover:scale-110 transition-transform" />
								<span class="hidden sm:inline">{t("home.library")}</span>
							</a>
						</div>
					</div>
				</div>

				<section>
					<div class="flex items-center justify-between mb-3">
						<h2 class="text-xs font-bold text-white uppercase tracking-wider">{t("home.exploreContent")}</h2>
						<a href="/mods" class="text-xs text-[#caa97c] hover:underline font-bold">{t("home.viewAll")}</a>
					</div>

					<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
						{#each modpacks as pack}
							<a href="/mods" class="group rounded-2xl bg-[#18191c] border border-white/5 overflow-hidden hover:border-[#caa97c]/40 hover:-translate-y-1 hover:shadow-[0_12px_24px_rgba(0,0,0,0.4)] transition-all duration-300 cursor-pointer flex flex-col justify-between">
								<div class="h-32 w-full relative bg-[#222328] overflow-hidden">
									<img src={pack.bgImg} class="w-full h-full object-cover opacity-85 group-hover:scale-105 transition-transform duration-300" alt={pack.title} />
									<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-transparent to-transparent"></div>
									
									<div class="absolute top-2.5 right-2.5 h-6 w-6 rounded-full bg-black/60 border border-white/10 flex items-center justify-center shadow-md">
										<span class="font-black text-[10px] text-emerald-400">m</span>
									</div>

									<div class="absolute bottom-2 left-2.5 h-9 w-9 rounded-xl overflow-hidden bg-black/60 border border-white/10 flex items-center justify-center shadow-md p-0.5">
										<img src={pack.iconImg} alt={pack.title} class="w-full h-full object-cover rounded-lg" />
									</div>
								</div>
								<div class="p-4 pt-3 flex-1 flex flex-col justify-between">
									<div>
										<h3 class="font-extrabold text-white text-xs truncate group-hover:text-[#caa97c] transition-colors">{pack.title}</h3>
										<p class="text-[10px] text-white/40 mt-1 line-clamp-2 leading-relaxed">{pack.subtitle}</p>
									</div>
									<div class="flex justify-between items-center mt-3 pt-2 border-t border-white/5 text-[10px] font-medium text-white/40">
										<span class="flex items-center gap-1"><Users class="w-3 h-3 text-white/30"/> {pack.author}</span>
										<span class="flex items-center gap-1 font-mono"><Download class="w-3 h-3 text-white/30"/> {pack.downloads}</span>
									</div>
								</div>
							</a>
						{/each}
					</div>
				</section>

				<div class="grid grid-cols-1 xl:grid-cols-12 gap-6 items-start">
					
					<section class="xl:col-span-6">
						<div class="flex items-center justify-between mb-3">
							<h2 class="text-xs font-bold text-white uppercase tracking-wider">{t("home.recommendedServers")}</h2>
							<a href="/servers" class="text-xs text-[#caa97c] hover:underline font-bold">{t("home.viewFullList")}</a>
						</div>

						<div class="flex flex-col gap-2">
							{#each servers as srv}
								<div 
									class="flex items-center justify-between bg-[#18191c] hover:bg-[#202126] border border-white/5 hover:border-white/20 p-3 rounded-2xl transition-all cursor-pointer group shadow-sm hover:-translate-y-0.5"
									onclick={() => {
										navigator.clipboard.writeText(srv.ip);
										toast(t("home.copiedIp", { ip: srv.ip }), "success");
									}}
									role="button"
									tabindex="0"
									onkeydown={(e) => { if (e.key === 'Enter') { navigator.clipboard.writeText(srv.ip); toast(t("home.copiedIpShort", { ip: srv.ip }), "success"); } }}
									title={t("home.clickToCopyIp")}
								>
									<div class="flex items-center gap-3">
										<div class="h-10 w-10 rounded-xl bg-black/40 border border-white/5 flex items-center justify-center shrink-0">
											<img src={srv.logo} alt={srv.name} class="w-7 h-7 rounded-md object-contain" />
										</div>
										<div>
											<div class="flex items-center gap-2">
												<h4 class="font-bold text-white text-xs group-hover:text-[#caa97c] transition-colors">{srv.name}</h4>
												<span class="bg-white/10 text-white/80 text-[8px] font-extrabold px-2 py-0.5 rounded-full border border-white/10 uppercase">{srv.badge}</span>
											</div>
											<div class="flex items-center gap-2 text-[10px] text-white/40 font-medium mt-0.5">
												<span class="text-white/60 font-medium flex items-center gap-1"><Users class="w-2.5 h-2.5" /> {srv.online}</span>
												<span>•</span>
												<span class="font-mono text-white/40">{srv.version}</span>
											</div>
										</div>
									</div>
									<div class="text-right flex items-center gap-2">
										<span class="text-emerald-400 font-mono text-[10px] font-bold bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20">{srv.ping}</span>
										<span class="text-white/30 font-mono text-xs font-bold">{srv.rank}</span>
									</div>
								</div>
							{/each}
						</div>
					</section>

					<section class="xl:col-span-6 flex flex-col">
						<div class="flex items-center justify-between mb-3">
							<h2 class="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-2">
								{t("home.yourPlayTime")}
							</h2>
							<span class="text-[10px] text-white/40 font-mono">{t("home.realTimeTracking")}</span>
						</div>

						<div class="bg-[#18191c] border border-white/5 rounded-2xl p-5 shadow-sm flex flex-col justify-between hover:border-white/15 transition-all">
							
							<div class="mb-4">
								<div class="text-2xl font-black text-white">{gamingStats.formattedTodayTime || "0m"}</div>
								<div class="text-[11px] text-white/40 mt-0.5">{t("home.last7Days")}</div>
							</div>

							<div class="my-4 py-6 border-y border-white/5 relative flex flex-col items-center justify-center">
								<div class="flex items-center justify-center py-2 text-white/30 text-xs font-medium">
									{t("home.noPlayTimeYet")}
								</div>

								<div class="w-full flex justify-between items-center text-[10px] text-white/40 font-medium mt-4 pt-2 border-t border-white/5 px-2">
									<span>Qui</span>
									<span>Sex</span>
									<span>Sáb</span>
									<span>Dom</span>
									<span>Seg</span>
									<span>Ter</span>
									<span class="text-white font-bold">Hoje</span>
								</div>
							</div>

							<div class="grid grid-cols-3 gap-2 pt-2 text-center">
								<div class="text-left">
									<div class="text-xs font-black text-white">{gamingStats.formattedLastSession || "0m"}</div>
									<div class="text-[10px] text-white/40 mt-0.5">{t("home.averageSession")}</div>
								</div>
								<div class="text-left">
									<div class="text-xs font-black text-white">{gamingStats.formattedTotalTime || "0m"}</div>
									<div class="text-[10px] text-white/40 mt-0.5">{t("home.longestSession")}</div>
								</div>
								<div class="text-left">
									<div class="text-xs font-black text-white">0 de 7</div>
									<div class="text-[10px] text-white/40 mt-0.5">{t("home.daysPlayed")}</div>
								</div>
							</div>

						</div>

					</section>

				</div>

				<section>
					<h2 class="text-xs font-bold text-white uppercase tracking-wider mb-3">{t("home.news")}</h2>
					<div class="rounded-2xl bg-[#18191c] border border-white/5 overflow-hidden shadow-md group cursor-pointer hover:border-white/20 transition-all">
						<div class="h-44 w-full relative bg-gradient-to-r from-purple-950/60 via-[#18191c] to-amber-950/40">
							<img src="https://images.unsplash.com/photo-1627856013091-fed6e4e30025?w=1000&auto=format&fit=crop&q=80" class="w-full h-full object-cover opacity-60 group-hover:scale-105 transition-transform duration-500" alt="News Banner" />
							<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-[#18191c]/50 to-transparent"></div>
							<div class="absolute bottom-4 left-5 right-5">
								<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest">27 DE AGO. DE 2026</span>
								<h3 class="text-base font-extrabold text-white mt-1 group-hover:text-[#caa97c] transition-colors">New on Java Realms: Mischiefs & Secrets</h3>
								<p class="text-xs text-white/60 mt-1 line-clamp-1">9 new and exciting maps have been released this month!</p>
							</div>
						</div>
					</div>
				</section>

			{/if}

		</div>

		<RightSidebar />

	</div>

{/if}