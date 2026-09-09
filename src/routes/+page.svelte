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
						class="w-full h-11 rounded-full hover:scale-[1.02] active:scale-95 text-black font-black text-xs flex items-center justify-center gap-2 transition-all shadow-lg cursor-pointer"
						style="background-color: var(--accent-color, #e2b86b);"
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
					class="w-full h-12 rounded-full bg-[#18191c] hover:bg-[#202127] border border-white/10 hover:border-white/20 text-white font-bold text-xs flex items-center justify-center gap-3 transition-all active:scale-98 cursor-pointer shadow-md"
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
	<div class="flex gap-8 h-full w-full select-none" in:fade={{ duration: 300 }}>
		
		<!-- MAIN CONTENT AREA -->
		<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-7">
			
			<!-- Top Welcome Header -->
			<div class="flex items-center justify-between mt-1">
				<div>
					<h1 class="text-3xl font-extrabold text-white tracking-tight flex items-center gap-3">
						Bem-vindo ao Luxmc, {account.value?.username || 'Gamer'}
					</h1>
					<p class="text-white/50 text-xs mt-1">Minecraft otimizado com alto desempenho e máxima fluidez no Linux.</p>
				</div>

				<a 
					href="/instances" 
					class="bg-[#18191c] border border-white/10 hover:border-white/20 rounded-full px-5 py-2.5 flex items-center gap-3 text-white transition-all group shadow-md cursor-pointer"
				>
					<div class="h-8 w-8 rounded-full bg-white/5 flex items-center justify-center group-hover:scale-110 transition-transform">
						<Boxes class="w-4 h-4 text-brand-500" />
					</div>
					<div class="text-left leading-tight">
						<div class="font-extrabold text-xs">Minhas Instâncias</div>
						<div class="text-[10px] text-white/40 font-medium">Jogar agora</div>
					</div>
				</a>
			</div>

			<!-- Destaques de Modpacks Populares -->
			<section>
				<div class="flex items-center justify-between mb-3">
					<h2 class="text-xs font-bold text-white uppercase tracking-wider">Modpacks em Destaque</h2>
					<a href="/mods" class="text-xs text-brand-500 hover:underline font-bold">Ver todos</a>
				</div>

				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					{#each modpacks as pack}
						<a href="/mods" class="group rounded-2xl bg-[#18191c] border border-white/5 overflow-hidden hover:border-brand-500/40 transition-all cursor-pointer shadow-md flex flex-col justify-between">
							<div class="h-32 w-full relative bg-[#222328] overflow-hidden">
								<img src={pack.bgImg} class="w-full h-full object-cover opacity-80 group-hover:scale-105 transition-transform duration-500" alt={pack.title} />
								<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-transparent to-transparent"></div>
								
								<!-- Badge Overlay -->
								<div class="absolute bottom-[-10px] left-3 h-11 w-11 rounded-xl overflow-hidden {pack.badgeColor} flex items-center justify-center font-black text-[10px] shadow-xl p-0.5">
									<img src={pack.iconImg} alt={pack.title} class="w-full h-full object-cover rounded-lg" />
								</div>
							</div>
							<div class="p-4 pt-4">
								<h3 class="font-extrabold text-white text-sm truncate group-hover:text-brand-500 transition-colors">{pack.title}</h3>
								<p class="text-[11px] text-white/50 mt-1 line-clamp-2 leading-relaxed">{pack.subtitle}</p>
								<div class="flex justify-between items-center mt-3 text-[10px] font-bold text-white/40">
									<span class="flex items-center gap-1"><Users class="w-3 h-3"/> {pack.author}</span>
									<span class="flex items-center gap-1"><Download class="w-3 h-3"/> {pack.downloads}</span>
								</div>
							</div>
						</a>
					{/each}
				</div>
			</section>

			<!-- Servidores Populares & Tempo de Jogo -->
			<div class="grid grid-cols-1 xl:grid-cols-12 gap-6 items-start">
				
				<!-- Servidores Populares -->
				<section class="xl:col-span-6">
					<div class="flex items-center justify-between mb-3">
						<h2 class="text-xs font-bold text-white uppercase tracking-wider">Servidores Recomendados</h2>
						<a href="/servers" class="text-xs text-brand-500 hover:underline font-bold">Ver lista completa</a>
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
											<h4 class="font-bold text-white text-xs group-hover:text-brand-500 transition-colors">{srv.name}</h4>
											<span class="bg-brand-500/10 text-brand-500 text-[8px] font-extrabold px-2 py-0.5 rounded-full uppercase border border-brand-500/20">{srv.badge}</span>
										</div>
										<div class="flex items-center gap-2 text-[10px] text-white/40 font-medium mt-0.5">
											<span class="text-emerald-400 font-bold">{srv.online} jogadores</span>
											<span>•</span>
											<span class="font-mono text-white/50">{srv.ip}</span>
										</div>
									</div>
								</div>
								<div class="text-right">
									<span class="text-emerald-400 font-mono font-bold text-xs flex items-center gap-1 justify-end">
										<Signal class="w-3 h-3" /> {srv.ping}
									</span>
									<span class="text-white/20 text-[9px] block">Copiar IP</span>
								</div>
							</div>
						{/each}
					</div>
				</section>

				<!-- Estatísticas de Jogo (Cards Clássicos Limpos) -->
				<section class="xl:col-span-6 flex flex-col">
					<div class="flex items-center justify-between mb-3">
						<h2 class="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-2">
							<Clock class="w-3.5 h-3.5 text-brand-500" /> Estatísticas de Jogo
						</h2>
						{#if gamingStats.isPlaying}
							<span class="text-[10px] text-emerald-400 font-bold flex items-center gap-1.5 animate-pulse bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20">
								<span class="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,1)]"></span>
								Minecraft em Execução
							</span>
						{:else}
							<span class="text-[10px] text-white/40 font-mono">Registro em Tempo Real</span>
						{/if}
					</div>

					<!-- Daily Playtime Highlight Card -->
					<div class="bg-gradient-to-r from-[#1c1d22] via-[#1a1b20] to-[#151619] border border-brand-500/25 rounded-2xl p-4 mb-3 flex items-center justify-between shadow-md relative overflow-hidden">
						<div class="absolute -right-4 -bottom-4 w-28 h-28 bg-brand-500/10 rounded-full blur-2xl pointer-events-none"></div>
						<div class="flex items-center gap-3.5 z-10">
							<div class="w-11 h-11 rounded-xl bg-brand-500/15 border border-brand-500/30 flex items-center justify-center shrink-0">
								<Clock class="w-5 h-5 text-brand-500" />
							</div>
							<div>
								<div class="text-[10px] uppercase font-black tracking-wider text-brand-500">Tempo de Jogo Diário</div>
								<div class="text-sm font-extrabold text-white mt-0.5">
									Você jogou <span class="text-brand-500 font-black">{gamingStats.formattedTodayTime}</span> hoje
								</div>
							</div>
						</div>
						{#if gamingStats.isPlaying}
							<div class="z-10 flex items-center gap-1.5 px-3 py-1 rounded-xl bg-emerald-500/15 border border-emerald-500/30 text-emerald-400 text-xs font-bold animate-pulse">
								<span class="w-2 h-2 rounded-full bg-emerald-400"></span>
								Ativo agora ({gamingStats.activeSessionMinutes}m)
							</div>
						{/if}
					</div>

					<div class="grid grid-cols-2 gap-3">
						<!-- Total Playtime Card -->
						<div class="bg-[#18191c] border border-white/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm hover:border-white/10 transition-all">
							<div class="w-11 h-11 rounded-xl bg-brand-500/10 border border-brand-500/20 flex items-center justify-center shrink-0">
								<Trophy class="w-5 h-5 text-brand-500" />
							</div>
							<div class="min-w-0">
								<div class="text-[10px] text-white/40 font-bold uppercase tracking-wider">Tempo Total</div>
								<div class="text-lg font-black text-white truncate mt-0.5">{gamingStats.formattedTotalTime}</div>
							</div>
						</div>

						<!-- Last Session Card -->
						<div class="bg-[#18191c] border border-white/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm hover:border-white/10 transition-all">
							<div class="w-11 h-11 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center shrink-0">
								<Flame class="w-5 h-5 text-emerald-400" />
							</div>
							<div class="min-w-0">
								<div class="text-[10px] text-white/40 font-bold uppercase tracking-wider">Última Sessão</div>
								<div class="text-lg font-black text-white truncate mt-0.5">{gamingStats.formattedLastSession}</div>
							</div>
						</div>

						<!-- Total Launches Card -->
						<div class="bg-[#18191c] border border-white/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm hover:border-white/10 transition-all">
							<div class="w-11 h-11 rounded-xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center shrink-0">
								<Zap class="w-5 h-5 text-blue-400" />
							</div>
							<div class="min-w-0">
								<div class="text-[10px] text-white/40 font-bold uppercase tracking-wider">Inicializações</div>
								<div class="text-lg font-black text-white truncate mt-0.5">{gamingStats.totalLaunches} vezes</div>
							</div>
						</div>

						<!-- Active/Favorite Version Card -->
						<div class="bg-[#18191c] border border-white/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm hover:border-white/10 transition-all">
							<div class="w-11 h-11 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center shrink-0">
								<Gamepad2 class="w-5 h-5 text-purple-400" />
							</div>
							<div class="min-w-0">
								<div class="text-[10px] text-white/40 font-bold uppercase tracking-wider">Versão Principal</div>
								<div class="text-lg font-black text-white truncate mt-0.5">{profiles.active?.mcVersion || "1.21.4"}</div>
							</div>
						</div>
					</div>

					<!-- Quick Instance Info Strip -->
					<div class="bg-[#18191c] border border-white/5 rounded-2xl p-3.5 flex items-center justify-between mt-3 shadow-sm">
						<div class="flex items-center gap-2.5">
							<Boxes class="w-4 h-4 text-white/40" />
							<span class="text-xs text-white/70 font-bold">Total de Instâncias Instaladas:</span>
						</div>
						<span class="text-xs font-mono font-black text-brand-500 bg-brand-500/10 px-3 py-1 rounded-full border border-brand-500/20">
							{profiles.list.length} {profiles.list.length === 1 ? 'instância' : 'instâncias'}
						</span>
					</div>
				</section>

			</div>

			<!-- Big Luxmc News Banner at Bottom of Main Area -->
			<section>
				<h2 class="text-xs font-bold text-white uppercase tracking-wider mb-3">Destaque da Comunidade Luxmc</h2>
				<div class="rounded-2xl bg-[#18191c] border border-white/5 overflow-hidden shadow-md">
					<div class="h-44 w-full relative bg-gradient-to-r from-amber-950/50 to-purple-950/40">
						<img src="https://images.unsplash.com/photo-1627856013091-fed6e4e30025?w=1000&auto=format&fit=crop&q=80" class="w-full h-full object-cover opacity-60" alt="News Banner" />
						<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-[#18191c]/50 to-transparent"></div>
						<div class="absolute bottom-4 left-5 right-5">
							<span class="text-[10px] font-bold text-brand-500 uppercase tracking-widest">LANÇAMENTO OFICIAL</span>
							<h3 class="text-base font-extrabold text-white mt-1">Luxmc v0.2.0 Beta: Pipeline Vulkan & Alta Performance Linux</h3>
							<p class="text-xs text-white/60 mt-1 line-clamp-1">Suporte nativo a Vulkan (Mesa Zink), autenticação rápida, integração NameMC e mais de 100 servidores.</p>
						</div>
					</div>
				</div>
			</section>

		</div>

		<!-- RIGHT SIDEBAR (Notícias & Luxmc Hub) -->
		<RightSidebar />

	</div>

{/if}