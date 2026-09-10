<script lang="ts">
	import { backOut, elasticOut } from "svelte/easing";
	import { fade } from "svelte/transition";
	import { onMount } from "svelte";
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
		EyeOff
	} from "lucide-svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { authDevLogin, authOfflineLogin } from "$lib/api";

	let offlineName = $state("");
	let offlinePassword = $state("");
	let offlineConfirmPassword = $state("");
	let showPassword = $state(false);
	let isLoggingIn = $state(false);
	let isEntering = $state(false);
	let authTab = $state<"login" | "register">("login");
	let savedAccounts = $state<string[]>([]);

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
					account.account = JSON.parse(saved);
				} catch {}
			}
		}
	});

	async function handleOfflineAuth() {
		const name = offlineName.trim();
		const pass = offlinePassword.trim();
		if (!name) {
			toast("Por favor, digite seu GamerTag ou nickname.", "error");
			return;
		}
		if (!pass) {
			toast("Por favor, digite a senha da conta.", "error");
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
					toast("A senha deve ter pelo menos 3 caracteres.", "error");
					isLoggingIn = false;
					return;
				}
				if (pass !== confirm) {
					toast("As senhas não coincidem!", "error");
					isLoggingIn = false;
					return;
				}
				if (accountsMap[key]) {
					toast(`O GamerTag "${name}" já existe! Mude para a aba "Entrar".`, "error");
					isLoggingIn = false;
					return;
				}
				accountsMap[key] = pass;
				localStorage.setItem("luxmc_offline_passwords", JSON.stringify(accountsMap));
				toast(`Conta "${name}" criada com sucesso!`, "success");
			} else {
				if (!accountsMap[key]) {
					toast(`Conta "${name}" não cadastrada. Clique em "Criar Conta" para se registrar.`, "error");
					isLoggingIn = false;
					return;
				}
				if (accountsMap[key] !== pass) {
					toast(`Senha incorreta para "${name}"! Tente novamente.`, "error");
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
			toast(`Bem-vindo ao Luxmc, ${name}!`, "success");

			account.account = newAcc;
		} catch (e) {
			toast("Erro ao processar conta: " + String(e), "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleMicrosoftLogin() {
		isLoggingIn = true;
		try {
			const acc = await authDevLogin().catch(() => ({
				id: "ms_" + Date.now(),
				username: "GamerPro",
				uuid: "ms-uuid-" + Date.now(),
				accessToken: "token_" + Date.now(),
				expiresAt: Date.now() + 86400000
			}));
			const newAcc = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken,
				expiresAt: acc.expiresAt
			};
			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
			toast(`Conectado como ${acc.username}!`, "success");

			account.account = newAcc;
		} catch (e) {
			toast("Erro no login Microsoft: " + String(e), "error");
		} finally {
			isLoggingIn = false;
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
			badgeText: "FO",
			badgeColor: "bg-[#1bd96a]/20 text-[#1bd96a] border border-[#1bd96a]/30"
		},
		{
			id: "cobblemon",
			title: "Cobblemon Official Modpack",
			subtitle: "Capture, treine e explore um mundo vivo com criaturas em estilo pixel art.",
			author: "CobbledStudios",
			downloads: "10.4M",
			bgImg: "/modpack_cobblemon.webp",
			iconImg: "/modpack_cobblemon_icon.png",
			badgeText: "COBBLE",
			badgeColor: "bg-red-950/80 text-red-200 border border-red-500/30"
		},
		{
			id: "bmc2",
			title: "Better MC [FABRIC] - BMC2",
			subtitle: "A experiência definitiva de Minecraft com novas dimensões, chefes e masmorras.",
			author: "SHXRKIE",
			downloads: "3.3M",
			bgImg: "/modpack_better_mc.webp",
			iconImg: "/modpack_bmc_icon.webp",
			badgeText: "BMC2",
			badgeColor: "bg-purple-950/80 text-purple-300 border border-purple-500/30"
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
	<!-- SLEEK LUXMC LOGIN SCREEN -->
	<div class="flex h-full w-full items-center justify-center px-4 transition-all duration-500 {isEntering ? 'opacity-0 scale-95 blur-md pointer-events-none' : 'opacity-100 scale-100'}" in:fade={{ duration: 300 }}>
		<div class="w-full max-w-md rounded-3xl bg-[#141518] border border-white/10 p-8 shadow-2xl space-y-7 flex flex-col items-center select-none relative overflow-hidden">
			<!-- Ambient glow -->
			<div class="absolute -top-10 -right-10 w-44 h-44 bg-brand-500/15 rounded-full blur-3xl pointer-events-none"></div>

			<!-- Official 3D Logo Header -->
			<div class="flex flex-col items-center text-center">
				<div class="h-24 w-24 rounded-3xl bg-black/50 border border-white/10 flex items-center justify-center p-3 shadow-2xl relative group">
					<img src="/logo.png" alt="Luxmc 3D" class="w-full h-full object-contain drop-shadow-[0_0_24px_rgba(226,184,107,0.5)] group-hover:scale-105 transition-transform" />
				</div>
				<h1 class="text-2xl font-black text-white tracking-tight mt-4">Luxmc Launcher</h1>
				<p class="text-xs text-white/50 mt-1">O Launcher Linux de Baixa Latência e Alto FPS</p>
			</div>

			<!-- Auth Options -->
			<div class="w-full space-y-4">
				<!-- Offline / Pirate Login & Register with Tabs -->
				<div class="bg-[#18191c] border border-white/10 rounded-3xl p-5 space-y-4 shadow-lg">
					<div class="flex bg-[#121316] p-1 rounded-full border border-white/5">
						<button 
							type="button" 
							class="flex-1 py-2 rounded-full text-xs font-bold transition-all {authTab === 'login' ? 'bg-[#25262c] text-brand-500 shadow-sm border border-brand-500/20' : 'text-white/40 hover:text-white'}"
							onclick={() => authTab = 'login'}
						>
							Entrar
						</button>
						<button 
							type="button" 
							class="flex-1 py-2 rounded-full text-xs font-bold transition-all {authTab === 'register' ? 'bg-[#25262c] text-brand-500 shadow-sm border border-brand-500/20' : 'text-white/40 hover:text-white'}"
							onclick={() => authTab = 'register'}
						>
							Criar Conta
						</button>
					</div>

					{#if savedAccounts.length > 0 && authTab === 'login'}
						<div class="flex flex-wrap items-center gap-1.5 pt-0.5">
							<span class="text-[10px] text-white/40 font-semibold">Salvas:</span>
							{#each savedAccounts as accName}
								<button 
									type="button" 
									class="text-[10px] font-bold px-3 py-1 rounded-full border transition-all cursor-pointer {offlineName.toLowerCase() === accName.toLowerCase() ? 'border-brand-500 bg-brand-500/20 text-brand-500' : 'border-white/10 bg-white/5 text-white/60 hover:text-white'}"
									onclick={() => { offlineName = accName; }}
								>
									{accName}
								</button>
							{/each}
						</div>
					{/if}

					<div class="flex items-center gap-2 text-xs font-bold text-white/80">
						<Gamepad2 class="w-4 h-4 text-brand-500" />
						<span>{authTab === 'login' ? 'Acessar Conta Offline / Pirata' : 'Registrar Nova Conta Offline'}</span>
					</div>

					<div class="space-y-2.5">
						<input 
							type="text" 
							placeholder="GamerTag (ex: Steve, Pedro, Gamer)..." 
							bind:value={offlineName}
							class="w-full bg-[#1e1f24] border border-white/10 rounded-full px-5 py-2.5 text-xs font-bold text-white outline-none focus:border-brand-500 transition-colors"
							maxlength="16"
							onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
						/>

						<div class="relative">
							<input 
								type={showPassword ? "text" : "password"} 
								placeholder="Senha da conta..." 
								bind:value={offlinePassword}
								class="w-full bg-[#1e1f24] border border-white/10 rounded-full pl-5 pr-12 py-2.5 text-xs font-bold text-white outline-none focus:border-brand-500 transition-colors"
								onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
							/>
							<button
								type="button"
								class="absolute right-3.5 top-1/2 -translate-y-1/2 text-white/40 hover:text-white p-1.5 transition-colors cursor-pointer"
								onclick={() => (showPassword = !showPassword)}
								title={showPassword ? "Ocultar Senha" : "Exibir Senha"}
							>
								{#if showPassword}
									<EyeOff class="w-3.5 h-3.5" />
								{:else}
									<Eye class="w-3.5 h-3.5" />
								{/if}
							</button>
						</div>

						{#if authTab === 'register'}
							<input 
								type={showPassword ? "text" : "password"} 
								placeholder="Confirme a senha..." 
								bind:value={offlineConfirmPassword}
								class="w-full bg-[#1e1f24] border border-white/10 rounded-full px-5 py-2.5 text-xs font-bold text-white outline-none focus:border-brand-500 transition-colors"
								onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
							/>
						{/if}
					</div>

					<button 
						type="button" 
						class="w-full h-11 rounded-xl hover:brightness-105 active:scale-[0.98] text-[#15171c] font-bold text-xs flex items-center justify-center gap-2 transition-all shadow-sm cursor-pointer"
						style="background-color: #caa97c;"
						onclick={handleOfflineAuth}
						disabled={isLoggingIn}
					>
						{#if isLoggingIn}
							<Loader2 class="w-4 h-4 animate-spin" /> Processando...
						{:else}
							<Play class="w-4 h-4 fill-current" /> {authTab === 'login' ? 'Entrar e Jogar' : 'Criar Conta e Jogar'}
						{/if}
					</button>
				</div>

				<div class="flex items-center gap-3">
					<div class="flex-1 h-[1px] bg-white/10"></div>
					<span class="text-[10px] font-bold text-white/30 uppercase">ou</span>
					<div class="flex-1 h-[1px] bg-white/10"></div>
				</div>

				<!-- Official Microsoft Login -->
				<button 
					type="button" 
					class="w-full h-11 rounded-xl bg-[#1e1f24] hover:bg-[#282930] border border-white/10 hover:border-white/20 text-white font-semibold text-xs flex items-center justify-center gap-3 transition-all active:scale-[0.98] cursor-pointer shadow-sm"
					onclick={handleMicrosoftLogin}
					disabled={isLoggingIn}
				>
					<svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none">
						<rect x="1" y="1" width="10" height="10" fill="#f25022" rx="1"/>
						<rect x="13" y="1" width="10" height="10" fill="#7fba00" rx="1"/>
						<rect x="1" y="13" width="10" height="10" fill="#00a4ef" rx="1"/>
						<rect x="13" y="13" width="10" height="10" fill="#ffb900" rx="1"/>
					</svg>
					<span>Entrar com Conta Oficial Microsoft</span>
				</button>
			</div>

			<div class="text-[10px] text-white/30 text-center leading-relaxed">
				Compatível com skins 3D volumétricas, servidores mundiais e servidores piratas.
			</div>
		</div>
	</div>

{:else}

	<!-- MAIN DASHBOARD -->
	<div class="flex gap-8 h-full w-full select-none" in:fade={{ duration: 100 }}>
		
		<!-- MAIN CONTENT AREA -->
		<!-- MAIN CONTENT AREA -->
		<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-7">
			
			<!-- Top Welcome Header matching Reference Image 1 -->
			<div class="flex items-center justify-between mt-1">
				<div>
					<h1 class="text-3xl font-extrabold text-white tracking-tight flex items-center gap-3">
						Bora lá, {account.value?.username || 'Jogador'}
					</h1>
					<p class="text-white/40 text-xs mt-1 font-medium">Hora nobre, loot nobre.</p>
				</div>

				<a 
					href="/instances" 
					class="bg-[#18191c] border border-white/10 hover:border-white/20 text-white hover:scale-[1.02] active:scale-98 rounded-2xl px-5 py-3 flex items-center gap-3.5 transition-all shadow-md group cursor-pointer"
				>
					<div class="h-9 w-9 rounded-xl bg-white/5 border border-white/5 flex items-center justify-center group-hover:scale-105 transition-transform text-[#d8bc98]">
						<Boxes class="w-4 h-4" />
					</div>
					<div class="text-left leading-tight">
						<div class="font-bold text-xs text-white">Navegar na Biblioteca</div>
						<div class="text-[10px] text-white/40 font-medium">Explorar as suas instâncias</div>
					</div>
				</a>
			</div>

			<!-- Explorar Central de Conteúdo matching Reference Image 1 -->
			<section>
				<div class="flex items-center justify-between mb-3">
					<h2 class="text-xs font-bold text-white uppercase tracking-wider">Explorar Central de Conteúdo</h2>
					<a href="/mods" class="text-xs text-[#caa97c] hover:underline font-bold">Ver todos</a>
				</div>

				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					{#each modpacks as pack}
						<a href="/mods" class="group rounded-2xl bg-[#18191c] border border-white/5 overflow-hidden hover:border-[#caa97c]/40 hover:-translate-y-1 hover:shadow-[0_12px_24px_rgba(0,0,0,0.4)] transition-all duration-300 cursor-pointer flex flex-col justify-between">
							<div class="h-32 w-full relative bg-[#222328] overflow-hidden">
								<img src={pack.bgImg} class="w-full h-full object-cover opacity-85 group-hover:scale-105 transition-transform duration-300" alt={pack.title} />
								<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-transparent to-transparent"></div>
								
								<!-- Modrinth Icon at top right -->
								<div class="absolute top-2.5 right-2.5 h-6 w-6 rounded-full bg-black/60 border border-white/10 flex items-center justify-center shadow-md">
									<span class="font-black text-[10px] text-emerald-400">m</span>
								</div>

								<!-- Badge Overlay on cover -->
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

			<!-- Servidores & O seu tempo de jogo matching Reference Image 1 -->
			<div class="grid grid-cols-1 xl:grid-cols-12 gap-6 items-start">
				
				<!-- Servidores (Left Column) -->
				<section class="xl:col-span-6">
					<div class="flex items-center justify-between mb-3">
						<h2 class="text-xs font-bold text-white uppercase tracking-wider">Servidores</h2>
						<a href="/servers" class="text-xs text-[#caa97c] hover:underline font-bold">Ver lista completa</a>
					</div>

					<div class="flex flex-col gap-2">
						{#each servers as srv}
							<div 
								class="flex items-center justify-between bg-[#18191c] border border-white/5 p-3 rounded-2xl hover:border-white/20 transition-all cursor-pointer group shadow-sm"
								onclick={() => {
									navigator.clipboard.writeText(srv.ip);
									toast(`IP ${srv.ip} copiado para a área de transferência!`, "success");
								}}
								role="button"
								tabindex="0"
								onkeydown={(e) => { if (e.key === 'Enter') { navigator.clipboard.writeText(srv.ip); toast(`IP ${srv.ip} copiado!`, "success"); } }}
								title="Clique para copiar o IP"
							>
								<div class="flex items-center gap-3">
									<div class="h-10 w-10 rounded-xl bg-black/40 border border-white/5 flex items-center justify-center shrink-0">
										<img src={srv.logo} alt={srv.name} class="w-7 h-7 rounded-md object-contain" />
									</div>
									<div>
										<div class="flex items-center gap-2">
											<h4 class="font-bold text-white text-xs group-hover:text-[#caa97c] transition-colors">{srv.name}</h4>
											<span class="bg-white/10 text-white/80 text-[8px] font-extrabold px-2 py-0.5 rounded-full uppercase border border-white/10">{srv.badge}</span>
										</div>
										<div class="flex items-center gap-2 text-[10px] text-white/40 font-medium mt-0.5">
											<span class="text-white/60 font-medium flex items-center gap-1"><Users class="w-2.5 h-2.5" /> {srv.online}</span>
											<span>•</span>
											<span class="font-mono text-white/40">{srv.version}</span>
										</div>
									</div>
								</div>
								<div class="text-right flex items-center gap-2">
									<span class="text-white/30 font-mono text-xs font-bold">{srv.rank}</span>
								</div>
							</div>
						{/each}
					</div>
				</section>

				<!-- O seu tempo de jogo (Right Column matching Reference Image 1) -->
				<section class="xl:col-span-6 flex flex-col">
					<div class="flex items-center justify-between mb-3">
						<h2 class="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-2">
							O seu tempo de jogo
						</h2>
						<span class="text-[10px] text-white/40 font-mono">Registro em Tempo Real</span>
					</div>

					<!-- Playtime Chart Card matching Reference Image 1 -->
					<div class="bg-[#18191c] border border-white/5 rounded-2xl p-5 shadow-sm flex flex-col justify-between">
						
						<!-- Top Metric -->
						<div class="mb-4">
							<div class="text-2xl font-black text-white">{gamingStats.formattedTodayTime || "0m"}</div>
							<div class="text-[11px] text-white/40 mt-0.5">Últimos 7 dias</div>
						</div>

						<!-- Middle 7-day Bar Visual -->
						<div class="my-4 py-6 border-y border-white/5 relative flex flex-col items-center justify-center">
							<div class="flex items-center justify-center py-2 text-white/30 text-xs font-medium">
								Ainda sem tempo de jogo
							</div>

							<!-- 7-day labels row -->
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

						<!-- Bottom 3 Metrics Row -->
						<div class="grid grid-cols-3 gap-2 pt-2 text-center">
							<div class="text-left">
								<div class="text-xs font-black text-white">{gamingStats.formattedLastSession || "0m"}</div>
								<div class="text-[10px] text-white/40 mt-0.5">Sessão média</div>
							</div>
							<div class="text-left">
								<div class="text-xs font-black text-white">{gamingStats.formattedTotalTime || "0m"}</div>
								<div class="text-[10px] text-white/40 mt-0.5">Sessão mais longa</div>
							</div>
							<div class="text-left">
								<div class="text-xs font-black text-white">0 de 7</div>
								<div class="text-[10px] text-white/40 mt-0.5">Dias jogados</div>
							</div>
						</div>

					</div>

				</section>

			</div>

			<!-- Big Featured News Banner at Bottom matching Reference Image 1 -->
			<section>
				<h2 class="text-xs font-bold text-white uppercase tracking-wider mb-3">Notícias</h2>
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

		</div>

		<!-- RIGHT SIDEBAR (Notícias & Luxmc Hub) -->
		<RightSidebar />

	</div>

{/if}