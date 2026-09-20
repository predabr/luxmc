<script lang="ts">
	import { parseJoinAddress } from "$lib/utils/directJoin";
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { 
		Share2, 
		Copy, 
		Check, 
		Wifi, 
		X, 
		Globe, 
		Play, 
		Radio,
		RefreshCw,
		Router,
		Server,
		ShieldCheck
	} from "lucide-svelte";
	import { 
		p2pGetHostLink, 
		p2pScanLanWorlds, 
		upnpOpenPort, 
		type HostLinkInfo, 
		type DiscoveredLanWorld,
		type UpnpPortMappingResult 
	} from "$lib/api/p2p";
	import { toast } from "$lib/stores/toasts.svelte";
	import Button from "./Button.svelte";

	type Props = {
		open: boolean;
		onClose: () => void;
		onConnect?: (ip: string, port: number) => void;
	};

	let { open, onClose, onConnect }: Props = $props();

	let portInput = $state(25565);
	let hostInfo = $state<HostLinkInfo | null>(null);
	let copiedKey = $state<string | null>(null);
	let activeTab = $state<"host" | "join">("host");
	let joinInput = $state("");

	let isScanning = $state(false);
	let discoveredWorlds = $state<DiscoveredLanWorld[]>([]);
	let isUpnpLoading = $state(false);
	let upnpResult = $state<UpnpPortMappingResult | null>(null);

	async function refreshHostLink() {
		try {
			hostInfo = await p2pGetHostLink(portInput);
		} catch (e) {
			console.warn("Failed to get host link:", e);
		}
	}

	async function scanLanWorlds() {
		isScanning = true;
		try {
			discoveredWorlds = await p2pScanLanWorlds();
		} catch (e) {
			console.warn("Failed to scan LAN worlds:", e);
		} finally {
			isScanning = false;
		}
	}

	async function handleUpnp() {
		if (isUpnpLoading) return;
		isUpnpLoading = true;
		try {
			const res = await upnpOpenPort(portInput);
			upnpResult = res;
			if (res.success) {
				toast(`Porta ${portInput} aberta no roteador via UPnP!`, "success");
				await refreshHostLink();
			} else {
				toast(res.message || "Não foi possível abrir a porta via UPnP", "error");
			}
		} catch (e) {
			toast(String(e), "error");
		} finally {
			isUpnpLoading = false;
		}
	}

	onMount(() => {
		refreshHostLink();
		scanLanWorlds();
	});

	$effect(() => {
		if (open) {
			refreshHostLink();
			scanLanWorlds();
		}
	});

	function copyText(text: string, key: string) {
		navigator.clipboard.writeText(text).then(() => {
			copiedKey = key;
			toast("Copiado para a área de transferência!", "success");
			setTimeout(() => {
				if (copiedKey === key) copiedKey = null;
			}, 2000);
		});
	}

	function handleJoin() {
		try {
			const { host, port } = parseJoinAddress(joinInput);
			onConnect?.(host, port);
			onClose();
		} catch (error) {
			toast(String(error), "error");
		}
	}

	function joinDiscovered(world: DiscoveredLanWorld) {
		onConnect?.(world.host, world.port);
		onClose();
	}

	function useDiscoveredPort(port: number) {
		portInput = port;
		refreshHostLink();
		toast(`Porta ${port} selecionada para hospedar!`, "info");
	}
</script>

{#if open}
	<div 
		class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-md"
		transition:fade={{ duration: 180 }}
	>
		<div 
			class="relative w-full max-w-lg bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-2xl overflow-hidden flex flex-col gap-5 max-h-[90vh] overflow-y-auto custom-scrollbar"
			transition:scale={{ start: 0.95, duration: 200 }}
		>
			<div class="absolute -top-24 -left-20 w-48 h-48 bg-emerald-500/15 rounded-full blur-3xl pointer-events-none"></div>

			<div class="flex items-start justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-emerald-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
						<Radio class="w-5 h-5 animate-pulse" />
					</div>
					<div>
						<h3 class="text-base font-bold text-fg">Luxmc P2P & Host Direto</h3>
						<p class="text-xs text-fg/50">Jogue com amigos em LAN, VPN ou Internet com UPnP</p>
					</div>
				</div>
				<button 
					type="button" 
					class="p-2 rounded-xl text-fg/40 hover:text-fg hover:bg-fg/5 transition-colors cursor-pointer"
					onclick={onClose}
					aria-label="Fechar"
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			<div class="flex bg-bg-subtle p-1 rounded-2xl border border-fg/5">
				<button 
					type="button" 
					class="flex-1 py-1.5 text-xs font-bold rounded-xl transition-all cursor-pointer {activeTab === 'host' ? 'bg-bg-overlay text-fg shadow-sm' : 'text-fg/40 hover:text-fg'}"
					onclick={() => activeTab = 'host'}
				>
					Hospedar Mundo
				</button>
				<button 
					type="button" 
					class="flex-1 py-1.5 text-xs font-bold rounded-xl transition-all cursor-pointer {activeTab === 'join' ? 'bg-bg-overlay text-fg shadow-sm' : 'text-fg/40 hover:text-fg'}"
					onclick={() => activeTab = 'join'}
				>
					Entrar via Link / Rede
				</button>
			</div>

			{#if activeTab === "host"}
				<div class="flex flex-col gap-4">
					{#if discoveredWorlds.length > 0}
						<div class="p-3 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 flex flex-col gap-2">
							<div class="flex items-center justify-between">
								<span class="text-xs font-bold text-emerald-400 flex items-center gap-1.5">
									<Wifi class="w-3.5 h-3.5" /> Mundos LAN Detectados no Minecraft
								</span>
								<button 
									type="button" 
									onclick={scanLanWorlds} 
									disabled={isScanning}
									class="text-[10px] text-emerald-400 hover:underline flex items-center gap-1 cursor-pointer"
								>
									<RefreshCw class="w-3 h-3 {isScanning ? 'animate-spin' : ''}" /> Atualizar
								</button>
							</div>
							{#each discoveredWorlds as world}
								<div class="flex items-center justify-between bg-bg/50 px-3 py-2 rounded-xl border border-emerald-500/10 text-xs">
									<div class="truncate mr-2">
										<span class="font-bold text-fg">{world.motd || "Mundo LAN"}</span>
										<span class="text-fg/40 text-[10px] ml-1.5 font-mono">Porta: {world.port}</span>
									</div>
									<Button size="sm" variant="ghost" class="text-emerald-400 text-xs shrink-0" onclick={() => useDiscoveredPort(world.port)}>
										Usar Porta
									</Button>
								</div>
							{/each}
						</div>
					{/if}

					<div class="flex items-center justify-between gap-3 bg-bg-subtle p-3 rounded-2xl border border-fg/5">
						<div class="flex flex-col">
							<label for="p2p-port-input" class="text-xs font-semibold text-fg">Porta do Mundo LAN</label>
							<span class="text-[11px] text-fg/40">Porta exibida no chat ao abrir para LAN</span>
						</div>
						<div class="flex items-center gap-2">
							<input 
								id="p2p-port-input"
								type="number" 
								bind:value={portInput}
								oninput={refreshHostLink}
								class="w-24 bg-bg border border-fg/10 rounded-xl px-2.5 py-1 text-xs font-mono text-fg text-center focus:border-brand-500 outline-none"
							/>
							<Button 
								variant="ghost" 
								size="sm" 
								class="text-xs text-brand-400 shrink-0 border border-brand-500/20"
								onclick={handleUpnp}
								disabled={isUpnpLoading}
							>
								{#if isUpnpLoading}
									<RefreshCw class="w-3 h-3 animate-spin mr-1" />
								{:else}
									<Router class="w-3 h-3 mr-1" />
								{/if}
								UPnP
							</Button>
						</div>
					</div>

					{#if upnpResult}
						<div class="p-2.5 rounded-xl text-xs flex items-center gap-2 {upnpResult.success ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'}">
							<ShieldCheck class="w-4 h-4 shrink-0" />
							<span>{upnpResult.message}</span>
						</div>
					{/if}

					{#if hostInfo}
						<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-4 flex flex-col gap-3">
							<div class="flex items-center justify-between">
								<div class="text-[11px] font-bold text-fg/50 uppercase tracking-wider flex items-center gap-1.5">
									<Wifi class="w-3.5 h-3.5 text-emerald-400" /> IP Local / VPN (Mesmo Wi-Fi, ZeroTier, Radmin)
								</div>
								<button 
									type="button" 
									class="text-xs font-bold text-emerald-400 hover:underline flex items-center gap-1 cursor-pointer"
									onclick={() => copyText(hostInfo?.directAddress || "", "direct")}
								>
									{#if copiedKey === "direct"}<Check class="w-3.5 h-3.5" />{:else}<Copy class="w-3.5 h-3.5" />{/if}
									Copiar IP
								</button>
							</div>
							<div class="font-mono text-xs font-bold text-fg bg-bg-overlay/40 px-3 py-2 rounded-xl border border-fg/5 truncate">
								{hostInfo.directAddress}
							</div>

							{#if hostInfo.publicAddress}
								<div class="flex items-center justify-between pt-2 border-t border-fg/5">
									<div class="text-[11px] font-bold text-fg/50 uppercase tracking-wider flex items-center gap-1.5">
										<Globe class="w-3.5 h-3.5 text-blue-400" /> IP Público (Internet com Porta Aberta / UPnP)
									</div>
									<button 
										type="button" 
										class="text-xs font-bold text-blue-400 hover:underline flex items-center gap-1 cursor-pointer"
										onclick={() => copyText(hostInfo?.publicAddress || "", "public")}
									>
										{#if copiedKey === "public"}<Check class="w-3.5 h-3.5" />{:else}<Copy class="w-3.5 h-3.5" />{/if}
										Copiar
									</button>
								</div>
								<div class="font-mono text-xs font-bold text-fg bg-bg-overlay/40 px-3 py-2 rounded-xl border border-fg/5 truncate">
									{hostInfo.publicAddress}
								</div>
							{/if}

							<div class="flex items-center justify-between pt-2 border-t border-fg/5">
								<div class="text-[11px] font-bold text-fg/50 uppercase tracking-wider flex items-center gap-1.5">
									<Share2 class="w-3.5 h-3.5 text-brand-400" /> Link Direto One-Click Luxmc
								</div>
								<button 
									type="button" 
									class="text-xs font-bold text-brand-400 hover:underline flex items-center gap-1 cursor-pointer"
									onclick={() => copyText(hostInfo?.shareLink || "", "link")}
								>
									{#if copiedKey === "link"}<Check class="w-3.5 h-3.5" />{:else}<Share2 class="w-3.5 h-3.5" />{/if}
									Copiar Link
								</button>
							</div>
							<div class="font-mono text-xs text-fg/70 bg-bg-overlay/40 px-3 py-2 rounded-xl border border-fg/5 truncate">
								{hostInfo.shareLink}
							</div>
						</div>
					{/if}

					<div class="text-[11px] text-fg/40 leading-relaxed">
						Abra seu mundo no Minecraft (<kbd class="px-1 py-0.5 rounded bg-bg text-[10px] text-fg/60">ESC</kbd> &gt; <kbd class="px-1 py-0.5 rounded bg-bg text-[10px] text-fg/60">Abrir para LAN</kbd>), use o botão UPnP para liberar portas automaticamente no roteador ou use VPNs como ZeroTier / Radmin / Tailscale com o IP local.
					</div>
				</div>
			{:else}
				<div class="flex flex-col gap-4">
					{#if discoveredWorlds.length > 0}
						<div class="flex flex-col gap-2">
							<div class="flex items-center justify-between">
								<span class="text-xs font-bold text-fg/70 flex items-center gap-1.5">
									<Wifi class="w-3.5 h-3.5 text-emerald-400" /> Mundos Encontrados na sua Rede Local
								</span>
								<button 
									type="button" 
									onclick={scanLanWorlds} 
									disabled={isScanning}
									class="text-[10px] text-brand-400 hover:underline flex items-center gap-1 cursor-pointer"
								>
									<RefreshCw class="w-3 h-3 {isScanning ? 'animate-spin' : ''}" /> Escanear
								</button>
							</div>
							<div class="flex flex-col gap-1.5">
								{#each discoveredWorlds as world}
									<div class="flex items-center justify-between bg-bg-subtle border border-fg/10 p-3 rounded-2xl hover:border-emerald-500/30 transition-colors">
										<div class="flex items-center gap-2.5 truncate mr-2">
											<Server class="w-4 h-4 text-emerald-400 shrink-0" />
											<div class="truncate">
												<div class="text-xs font-bold text-fg truncate">{world.motd || "Mundo Minecraft"}</div>
												<div class="text-[10px] font-mono text-fg/40">{world.host}:{world.port}</div>
											</div>
										</div>
										<Button size="sm" variant="primary" class="shrink-0 text-xs" onclick={() => joinDiscovered(world)}>
											<Play class="w-3 h-3 fill-current mr-1" /> Entrar
										</Button>
									</div>
								{/each}
							</div>
						</div>
					{/if}

					<div class="flex flex-col gap-1.5">
						<label for="p2p-join-input" class="text-xs font-semibold text-fg/70">Cole o link direto (luxmc://join/...) ou IP:Porta:</label>
						<input 
							id="p2p-join-input"
							type="text" 
							placeholder="192.168.1.100:25565 ou luxmc://join/..."
							bind:value={joinInput}
							class="w-full bg-bg-subtle border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg placeholder-fg/30 focus:border-emerald-500 outline-none"
						/>
					</div>

					<Button variant="primary" size="md" class="w-full justify-center" onclick={handleJoin}>
						<Play class="w-4 h-4 fill-current mr-1.5" /> Conectar Agora
					</Button>
				</div>
			{/if}
		</div>
	</div>
{/if}
