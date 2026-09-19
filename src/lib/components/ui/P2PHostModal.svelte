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
		Radio
	} from "lucide-svelte";
	import { p2pGetHostLink, type HostLinkInfo } from "$lib/api/p2p";
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
	let copied = $state(false);
	let activeTab = $state<"host" | "join">("host");
	let joinInput = $state("");

	async function refreshHostLink() {
		try {
			hostInfo = await p2pGetHostLink(portInput);
		} catch (e) {
			console.warn("Failed to get host link:", e);
		}
	}

	onMount(() => {
		refreshHostLink();
	});

	$effect(() => {
		const _p = portInput;
		refreshHostLink();
	});

	function copyLink() {
		if (!hostInfo) return;
		navigator.clipboard.writeText(hostInfo.shareLink).then(() => {
			copied = true;
			toast("Link P2P de conexão copiado!", "success");
			setTimeout(() => copied = false, 2500);
		});
	}

	function copyDirectAddress() {
		if (!hostInfo) return;
		navigator.clipboard.writeText(hostInfo.directAddress).then(() => {
			copied = true;
			toast("Endereço IP:Porta copiado!", "success");
			setTimeout(() => copied = false, 2500);
		});
	}

    function handleJoin() {
        try {
            const { host, port } = parseJoinAddress(joinInput);
            onConnect?.(host, port);
            onClose();
        } catch (error) { toast(String(error), "error"); }
    }

</script>

{#if open}
	<div 
		class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/75 backdrop-blur-md"
		transition:fade={{ duration: 180 }}
	>
		<div 
			class="relative w-full max-w-md bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-2xl overflow-hidden flex flex-col gap-5"
			transition:scale={{ start: 0.95, duration: 200 }}
		>
			<!-- Top Aura -->
			<div class="absolute -top-24 -left-20 w-48 h-48 bg-emerald-500/15 rounded-full blur-3xl pointer-events-none"></div>

			<!-- Header -->
			<div class="flex items-start justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-emerald-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
						<Radio class="w-5 h-5 animate-pulse" />
					</div>
					<div>
						<h3 class="text-base font-bold text-fg">Luxmc Direct P2P Link</h3>
						<p class="text-xs text-fg/50">Compartilhe e jogue em mundos LAN sem complicação</p>
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

			<!-- Tab switch -->
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
					Entrar via Link / Código
				</button>
			</div>

			{#if activeTab === "host"}
				<div class="flex flex-col gap-4">
					<div class="flex items-center gap-2">
						<label for="p2p-port-input" class="text-xs font-semibold text-fg/60 shrink-0">Porta LAN aberta no Minecraft:</label>
						<input 
							id="p2p-port-input"
							type="number" 
							bind:value={portInput}
							oninput={refreshHostLink}
							class="w-24 bg-bg border border-fg/10 rounded-xl px-2.5 py-1 text-xs font-mono text-fg text-center focus:border-brand-500 outline-none"
						/>
					</div>

					{#if hostInfo}
						<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-4 flex flex-col gap-3">
							<div class="flex items-center justify-between">
								<div class="text-[11px] font-bold text-fg/50 uppercase tracking-wider">Endereço Direto</div>
								<button 
									type="button" 
									class="text-xs font-bold text-emerald-400 hover:underline flex items-center gap-1 cursor-pointer"
									onclick={copyDirectAddress}
								>
									{#if copied}<Check class="w-3.5 h-3.5" />{:else}<Copy class="w-3.5 h-3.5" />{/if}
									Copiar IP
								</button>
							</div>
							<div class="font-mono text-sm font-bold text-fg bg-bg-overlay/40 px-3 py-2 rounded-xl border border-fg/5 truncate">
								{hostInfo.directAddress}
							</div>

							<div class="flex items-center justify-between pt-2 border-t border-fg/5">
								<div class="text-[11px] font-bold text-fg/50 uppercase tracking-wider">Link One-Click Luxmc</div>
								<button 
									type="button" 
									class="text-xs font-bold text-brand-400 hover:underline flex items-center gap-1 cursor-pointer"
									onclick={copyLink}
								>
									{#if copied}<Check class="w-3.5 h-3.5" />{:else}<Share2 class="w-3.5 h-3.5" />{/if}
									Copiar Link
								</button>
							</div>
							<div class="font-mono text-xs text-fg/70 bg-bg-overlay/40 px-3 py-2 rounded-xl border border-fg/5 truncate">
								{hostInfo.shareLink}
							</div>
						</div>
					{/if}

					<div class="text-[11px] text-fg/40 leading-relaxed">
						Abra seu mundo para LAN no Minecraft (ESC &gt; Abrir para LAN), insira a porta gerada acima e envie o link para seus amigos na mesma rede ou via VPN (ZeroTier / Radmin / Tailscale).
					</div>
				</div>
			{:else}
				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-1.5">
						<label for="p2p-join-input" class="text-xs font-semibold text-fg/70">Cole o link (luxmc://join/...) ou IP:Porta:</label>
						<input 
							id="p2p-join-input"
							type="text" 
							placeholder="192.168.1.100:25565 ou luxmc://join/..."
							bind:value={joinInput}
							class="w-full bg-bg-subtle border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg placeholder-fg/30 focus:border-emerald-500 outline-none"
						/>
					</div>

					<Button variant="primary" size="md" class="w-full justify-center" onclick={handleJoin}>
						<Play class="w-4 h-4 fill-current" /> Conectar Agora
					</Button>
				</div>
			{/if}
		</div>
	</div>
{/if}
