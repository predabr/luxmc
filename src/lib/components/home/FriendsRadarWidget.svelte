<script lang="ts">
	import { 
		Radio, 
		Users, 
		Wifi, 
		Gamepad2, 
		ExternalLink, 
		Plus, 
		Copy, 
		Check, 
		Share2, 
		Play,
		Server,
		ShieldCheck,
		Sparkles
	} from "lucide-svelte";
	import { goto } from "$app/navigation";
	import { toast } from "$lib/stores/toasts.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { launchGame, authDevLogin } from "$lib/api";
	import { upnpOpenPort, upnpClosePort, type UpnpPortMappingResult } from "$lib/api/p2p";
	import Modal from "$lib/components/ui/Modal.svelte";
	import Button from "$lib/components/ui/Button.svelte";

	type FriendStatus = {
		id: string;
		name: string;
		skinUrl: string;
		status: "in_game" | "in_server" | "online" | "idle";
		activity: string;
		detail: string;
		pingMs: number;
		serverAddress?: string;
		p2pCode?: string;
	};

	let friends = $state<FriendStatus[]>([
		{
			id: "1",
			name: "pedro_dev",
			skinUrl: "https://mc-heads.net/avatar/MHF_Steve/48",
			status: "in_server",
			activity: "MushMC Network",
			detail: "Jogando Bedwars 4v4 · Sala #12",
			pingMs: 14,
			serverAddress: "jogar.mush.com.br"
		},
		{
			id: "2",
			name: "Lucas_Miner",
			skinUrl: "https://mc-heads.net/avatar/MHF_Alex/48",
			status: "in_game",
			activity: "Mundo LAN Privado",
			detail: "Sobrevivendo no Better MC (Dia 34)",
			pingMs: 22,
			p2pCode: "LUX-7842"
		},
		{
			id: "3",
			name: "Kiro_PvP",
			skinUrl: "https://mc-heads.net/avatar/Notch/48",
			status: "in_server",
			activity: "Hypixel Network",
			detail: "SkyWars Ranked · 1.8.9",
			pingMs: 118,
			serverAddress: "mc.hypixel.net"
		},
		{
			id: "4",
			name: "AnaCraft",
			skinUrl: "https://mc-heads.net/avatar/MHF_Herobrine/48",
			status: "online",
			activity: "No Launcher",
			detail: "Customizando skins & shaders",
			pingMs: 18
		}
	]);

	let showDirectJoinModal = $state(false);
	let showHostLanModal = $state(false);
	let directJoinCode = $state("");
	let hostPort = $state(25565);
	let hostResult = $state<UpnpPortMappingResult | null>(null);
	let isHosting = $state(false);
	let hasCopiedCode = $state(false);

	async function launchWithTarget(profile: (typeof profiles.list)[0], serverIp?: string, serverPort?: number) {
		let accountId = account.value?.uuid;
		if (!accountId) {
			const dev = await authDevLogin().catch(() => null);
			accountId = dev?.uuid || "dev-offline-player";
		}
		return launchGame({
			versionId: profile.mcVersion || "1.21.4",
			accountId,
			profileId: profile.id,
			enableVulkan: profile.useVulkan === true,
			serverIp: serverIp || null,
			serverPort: serverPort || null,
			skinUrl: activeSkinStore.current.skinUrl || account.value?.skinUrl || null,
			skinVariant: activeSkinStore.current.type === "alex" ? "slim" : "classic",
			capeUrl: activeSkinStore.current.customCapeUrl || account.value?.capeUrl || null
		});
	}

	async function handleQuickJoin(friend: FriendStatus) {
		const targetProfile = profiles.active || profiles.list[0];
		if (!targetProfile) {
			toast("Crie ou selecione uma instância antes de conectar.", "error");
			return;
		}

		if (friend.serverAddress) {
			toast(`Iniciando Minecraft e conectando a ${friend.serverAddress}...`, "info");
			try {
				await launchWithTarget(targetProfile, friend.serverAddress, 25565);
			} catch (e) {
				toast(`Erro ao conectar: ${e}`, "error");
			}
		} else if (friend.p2pCode) {
			toast(`Conectando ao mundo P2P de ${friend.name} (${friend.p2pCode})...`, "info");
			try {
				await launchWithTarget(targetProfile);
			} catch (e) {
				toast(`Erro ao conectar: ${e}`, "error");
			}
		}
	}

	async function handleDirectJoinSubmit() {
		if (!directJoinCode.trim()) {
			toast("Insira um código de convite ou link válido.", "error");
			return;
		}

		const clean = directJoinCode.trim().replace("luxmc://join/", "");
		const targetProfile = profiles.active || profiles.list[0];
		if (!targetProfile) {
			toast("Nenhuma instância ativa selecionada.", "error");
			return;
		}

		toast(`Decodificando convite ${clean} e conectando direto...`, "info");
		showDirectJoinModal = false;

		try {
			if (clean.includes(":")) {
				const [host, portStr] = clean.split(":");
				const port = parseInt(portStr) || 25565;
				await launchWithTarget(targetProfile, host, port);
			} else {
				await launchWithTarget(targetProfile);
			}
		} catch (e) {
			toast(`Erro ao entrar no mundo: ${e}`, "error");
		}
	}

	async function handleStartHost() {
		isHosting = true;
		try {
			hostResult = await upnpOpenPort(hostPort);
			if (hostResult.success) {
				toast("Porta UPnP aberta com sucesso no roteador!", "success");
			} else {
				toast(hostResult.message, "error");
			}
		} catch (e) {
			toast(`Erro ao abrir porta UPnP: ${e}`, "error");
		} finally {
			isHosting = false;
		}
	}

	function copyGeneratedLink() {
		if (!hostResult?.externalIp) return;
		const link = `luxmc://join/${hostResult.externalIp}:${hostPort}`;
		navigator.clipboard.writeText(link);
		hasCopiedCode = true;
		toast("Link de convite copiado para a área de transferência!", "success");
		setTimeout(() => hasCopiedCode = false, 2500);
	}
</script>

<section class="flex flex-col gap-3">
	<!-- Header do Radar -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<div class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"></div>
			<h2 class="text-xs font-bold text-white uppercase tracking-wider flex items-center gap-1.5">
				<Radio class="w-3.5 h-3.5 text-[#caa97c]" />
				Radar de Amigos (Ghost Ping)
			</h2>
			<span class="text-[10px] bg-white/5 text-white/50 px-2 py-0.5 rounded-full font-mono font-semibold border border-white/5">
				{friends.filter(f => f.status !== 'idle').length} online
			</span>
		</div>

		<div class="flex items-center gap-2">
			<button
				type="button"
				class="text-[11px] font-bold text-[#caa97c] hover:text-[#e2b86b] transition flex items-center gap-1 bg-[#caa97c]/10 hover:bg-[#caa97c]/20 px-2.5 py-1 rounded-xl border border-[#caa97c]/20 cursor-pointer"
				onclick={() => showDirectJoinModal = true}
			>
				<ExternalLink class="w-3 h-3" />
				Entrar com Código
			</button>

			<button
				type="button"
				class="text-[11px] font-bold text-white/70 hover:text-white transition flex items-center gap-1 bg-white/5 hover:bg-white/10 px-2.5 py-1 rounded-xl border border-white/5 cursor-pointer"
				onclick={() => showHostLanModal = true}
			>
				<Share2 class="w-3 h-3 text-emerald-400" />
				Abrir Mundo (UPnP)
			</button>
		</div>
	</div>

	<!-- Cards dos Amigos -->
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
		{#each friends as friend}
			<div class="rounded-2xl bg-[#18191c] border border-white/5 hover:border-[#caa97c]/40 p-3.5 flex flex-col justify-between gap-3 transition-all hover:bg-[#1c1d22] shadow-sm group">
				<div class="flex items-start justify-between gap-2">
					<div class="flex items-center gap-2.5 min-w-0">
						<div class="relative w-10 h-10 rounded-xl overflow-hidden bg-black/40 border border-white/10 shrink-0">
							<img src={friend.skinUrl} alt={friend.name} class="w-full h-full object-cover" />
							<span class="absolute bottom-0.5 right-0.5 w-2.5 h-2.5 rounded-full border-2 border-[#18191c] {friend.status === 'in_game' || friend.status === 'in_server' ? 'bg-emerald-400' : 'bg-blue-400'}"></span>
						</div>
						<div class="min-w-0">
							<div class="flex items-center gap-1.5">
								<h3 class="text-xs font-bold text-white truncate">{friend.name}</h3>
							</div>
							<p class="text-[11px] font-semibold text-[#caa97c] truncate mt-0.5">{friend.activity}</p>
						</div>
					</div>

					<span class="text-[10px] font-mono text-emerald-400/90 bg-emerald-500/10 px-1.5 py-0.5 rounded-md border border-emerald-500/20 shrink-0 flex items-center gap-1">
						<Wifi class="w-2.5 h-2.5" />
						{friend.pingMs}ms
					</span>
				</div>

				<div class="pt-2 border-t border-white/5 flex items-center justify-between">
					<span class="text-[10px] text-white/40 truncate max-w-[150px]">{friend.detail}</span>

					{#if friend.serverAddress || friend.p2pCode}
						<button
							type="button"
							class="text-[10px] font-extrabold px-2.5 py-1 rounded-lg bg-emerald-500/15 hover:bg-emerald-500/25 text-emerald-300 border border-emerald-500/30 transition flex items-center gap-1 cursor-pointer"
							onclick={() => handleQuickJoin(friend)}
						>
							<Play class="w-2.5 h-2.5 fill-current" />
							Entrar
						</button>
					{:else}
						<span class="text-[10px] font-semibold text-white/30">Online</span>
					{/if}
				</div>
			</div>
		{/each}
	</div>
</section>

<!-- Modal: Entrar com Código Direct Join -->
<Modal isOpen={showDirectJoinModal} onClose={() => showDirectJoinModal = false} title="Conectar Direto a Amigo (Direct Join)">
	<div class="flex flex-col gap-4 text-xs">
		<p class="text-white/70 leading-relaxed">
			Insira o código de convite (ex: <code class="text-[#caa97c] font-mono">LUX-7842</code>) ou link gerado pelo launcher do seu amigo para entrar diretamente no mundo dele sem precisar de mods adicionais ou Hamachi.
		</p>

		<div class="flex flex-col gap-1.5">
			<label for="direct-join-code-input" class="text-white/60 font-semibold">Código ou Link de Convite:</label>
			<input 
				id="direct-join-code-input"
				type="text" 
				bind:value={directJoinCode}
				placeholder="Ex: luxmc://join/192.168.1.100:25565 ou LUX-7842"
				class="w-full bg-bg-subtle border border-white/10 rounded-xl px-3.5 py-2 text-white font-mono outline-none focus:border-[#caa97c]"
			/>
		</div>

		<div class="flex items-center justify-end gap-2 pt-2 border-t border-white/5">
			<Button variant="secondary" size="sm" onclick={() => showDirectJoinModal = false}>
				Cancelar
			</Button>
			<Button variant="primary" size="sm" onclick={handleDirectJoinSubmit}>
				<Play class="w-3.5 h-3.5 fill-current" />
				Conectar Agora
			</Button>
		</div>
	</div>
</Modal>

<!-- Modal: Abrir Mundo LAN via UPnP -->
<Modal isOpen={showHostLanModal} onClose={() => showHostLanModal = false} title="Hospedar Mundo para Amigos (UPnP)">
	<div class="flex flex-col gap-4 text-xs">
		<p class="text-white/70 leading-relaxed">
			O Luxmc usa o protocolo UPnP nativo para abrir automaticamente uma porta no seu roteador residencial. Seus amigos poderão entrar no seu mundo diretamente pela internet sem configurar nada!
		</p>

		<div class="flex items-center gap-3">
			<div class="flex-1 flex flex-col gap-1.5">
				<label for="host-port-input" class="text-white/60 font-semibold">Porta LAN do Minecraft:</label>
				<input 
					id="host-port-input"
					type="number" 
					bind:value={hostPort}
					class="w-full bg-bg-subtle border border-white/10 rounded-xl px-3.5 py-2 text-white font-mono outline-none"
				/>
			</div>

			<div class="self-end">
				<Button variant="primary" size="sm" disabled={isHosting} onclick={handleStartHost}>
					{isHosting ? "Abrindo..." : "Abrir Porta UPnP"}
				</Button>
			</div>
		</div>

		{#if hostResult?.success}
			<div class="bg-emerald-500/10 border border-emerald-500/20 rounded-xl p-3 flex flex-col gap-2">
				<div class="flex items-center gap-1.5 text-emerald-400 font-bold text-xs">
					<Check class="w-4 h-4" />
					Porta {hostPort} aberta com sucesso!
				</div>
				<p class="text-white/80">Envie este link para seu amigo colar no launcher dele:</p>
				<div class="flex items-center gap-2">
					<input 
						type="text" 
						readonly 
						value={`luxmc://join/${hostResult.externalIp || 'meu-ip'}:${hostPort}`}
						class="flex-1 bg-black/40 border border-white/10 rounded-lg px-2.5 py-1.5 font-mono text-[11px] text-white"
					/>
					<Button variant="secondary" size="sm" onclick={copyGeneratedLink}>
						{#if hasCopiedCode}
							<Check class="w-3.5 h-3.5 text-emerald-400" />
							Copiado!
						{:else}
							<Copy class="w-3.5 h-3.5" />
							Copiar
						{/if}
					</Button>
				</div>
			</div>
		{/if}

		<div class="flex items-center justify-end pt-2 border-t border-white/5">
			<Button variant="secondary" size="sm" onclick={() => showHostLanModal = false}>
				Fechar
			</Button>
		</div>
	</div>
</Modal>
