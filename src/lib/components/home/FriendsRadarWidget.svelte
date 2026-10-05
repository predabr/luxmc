<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import { parseJoinAddress } from "$lib/utils/directJoin";
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

	let friends = $state<FriendStatus[]>([]);

	let showDirectJoinModal = $state(false);
	let showHostLanModal = $state(false);
	let directJoinCode = $state("");
	let hostPort = $state(25565);
	let hostResult = $state<UpnpPortMappingResult | null>(null);
	let isHosting = $state(false);
	let hasCopiedCode = $state(false);

	async function launchWithTarget(profile: (typeof profiles.list)[0], serverIp?: string, serverPort?: number) {
		try {
			let accountId = account.value?.uuid;
			if (!accountId) {
				const dev = await authDevLogin().catch(() => null);
				accountId = dev?.uuid || "dev-offline-player";
			}
			return await launchGame({
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
		} catch (e) {
			toast(uiText("ui.ea9a9c9188e1f662") + String(e), "error");
			throw e;
		}
	}

	async function handleQuickJoin(friend: FriendStatus) {
		const targetProfile = profiles.active || profiles.list[0];
		if (!targetProfile) {
			toast(uiText("ui.6c4f3148d4036873"), "error");
			return;
		}

		if (friend.serverAddress) {
			toast(uiText("ui.060a298c5e2119c2", {arg0: (friend.serverAddress)}), "info");
			try {
				await launchWithTarget(targetProfile, friend.serverAddress, 25565);
			} catch (e) {
				toast(uiText("ui.eaf63d5aed7e674e", {arg0: (e)}), "error");
			}
		} else if (friend.p2pCode) {
			toast(uiText("ui.cfe07735c879d2ac", {arg0: (friend.name), arg1: (friend.p2pCode)}), "info");
			try {
				const target = parseJoinAddress(friend.p2pCode);
                await launchWithTarget(targetProfile, target.host, target.port);
			} catch (e) {
				toast(uiText("ui.eaf63d5aed7e674e", {arg0: (e)}), "error");
			}
		}
	}

    async function handleDirectJoinSubmit() {
        try {
            const { host, port } = parseJoinAddress(directJoinCode);
            const targetProfile = profiles.active || profiles.list[0];
            if (!targetProfile) throw new Error(uiText("ui.533d82c485c89647"));
            await launchWithTarget(targetProfile, host, port);
            showDirectJoinModal = false;
        } catch (error) { toast(String(error), "error"); }
    }

	async function handleStartHost() {
		isHosting = true;
		try {
			hostResult = await upnpOpenPort(hostPort);
			if (hostResult.success) {
				toast(uiText("ui.8960ce11b29dc29d"), "success");
			} else {
				toast(hostResult.message, "error");
			}
		} catch (e) {
			toast(uiText("ui.c221f5fe32e4605f", {arg0: (e)}), "error");
		} finally {
			isHosting = false;
		}
	}

	function copyGeneratedLink() {
		if (!hostResult?.externalIp) return;
		const link = `luxmc://join/${hostResult.externalIp}:${hostPort}`;
		navigator.clipboard.writeText(link);
		hasCopiedCode = true;
		toast(uiText("ui.7b7b3db4059029b7"), "success");
		setTimeout(() => hasCopiedCode = false, 2500);
	}
</script>

<section class="flex flex-col gap-3">
	<!-- Header do Radar -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<div class="w-2.5 h-2.5 rounded-full bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.7)]"></div>
			<h2 class="text-xs font-bold text-fg uppercase tracking-wider flex items-center gap-1.5">
				<Radio class="w-3.5 h-3.5 text-brand-400" />
				{uiText("ui.f37d700f4d9eca09")}
			</h2>
			<span class="text-[10px] bg-fg/5 text-fg/50 px-2 py-0.5 rounded-full font-mono font-semibold border border-fg/5">
				{friends.filter(f => f.status !== 'idle').length} {uiText("ui.f6fc84c9f21c2490")}
			</span>
		</div>

		<div class="flex items-center gap-2">
			<button
				type="button"
				class={launcherButton({ variant: "ghostBrand", size: "sm", class: "flex items-center gap-1" })}
				onclick={() => showDirectJoinModal = true}
			>
				<ExternalLink class="w-3 h-3" />
				{uiText("ui.ce76754cde6723ce")}
			</button>

			<button
				type="button"
				class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1" })}
				onclick={() => showHostLanModal = true}
			>
				<Share2 class="w-3 h-3 text-emerald-400" />
				{uiText("ui.2bbdf2aae9d64f7a")}
			</button>
		</div>
	</div>

	<!-- Cards dos Amigos -->
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
		{#if friends.length === 0}
			<div class="col-span-full rounded-2xl bg-bg-elevated border border-fg/5 p-6 flex flex-col items-center justify-center text-center">
				<div class="w-10 h-10 rounded-xl bg-brand-400/10 border border-brand-400/20 flex items-center justify-center mb-3">
					<Users class="w-5 h-5 text-brand-400" />
				</div>
				<p class="text-xs font-semibold text-fg/60 mb-1">{uiText("ui.a6612dce72867a0d")}</p>
				<p class="text-[10px] text-fg/35 mb-3">{uiText("ui.2a71a371672ac10a")}</p>
				<button
					type="button"
					onclick={() => goto("/friends")}
					class={launcherButton({ variant: "ghostBrand", size: "sm", class: "" })}
				>
					{uiText("ui.8343992ed5c76b01")}
				</button>
			</div>
		{:else}
			{#each friends as friend (friend.id)}
				<div class="rounded-2xl bg-bg-elevated border border-fg/5 hover:border-brand-400/40 p-3.5 flex flex-col justify-between gap-3 transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:bg-bg-subtle shadow-sm group">
					<div class="flex items-start justify-between gap-2">
						<div class="flex items-center gap-2.5 min-w-0">
							<div class="relative w-10 h-10 rounded-xl overflow-hidden bg-bg-overlay/40 border border-fg/10 shrink-0">
								<img loading="lazy" decoding="async" src={friend.skinUrl} alt={friend.name} class="w-full h-full object-cover" />
								<span class="absolute bottom-0.5 right-0.5 w-2.5 h-2.5 rounded-full border-2 border-border {friend.status === 'in_game' || friend.status === 'in_server' ? 'bg-emerald-400' : 'bg-blue-400'}"></span>
							</div>
							<div class="min-w-0">
								<div class="flex items-center gap-1.5">
									<h3 class="text-xs font-bold text-fg truncate">{friend.name}</h3>
								</div>
								<p class="text-[11px] font-semibold text-brand-400 truncate mt-0.5">{friend.activity}</p>
							</div>
						</div>

						<span class="text-[10px] font-mono text-emerald-400/90 bg-emerald-500/10 px-1.5 py-0.5 rounded-md border border-emerald-500/20 shrink-0 flex items-center gap-1">
							<Wifi class="w-2.5 h-2.5" />
							{friend.pingMs}ms
						</span>
					</div>

					<div class="pt-2 border-t border-fg/5 flex items-center justify-between">
						<span class="text-[10px] text-fg/40 truncate max-w-[150px]">{friend.detail}</span>

						{#if friend.serverAddress || friend.p2pCode}
							<button
								type="button"
								class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1" })}
								onclick={() => handleQuickJoin(friend)}
							>
								<Play class="w-2.5 h-2.5 fill-current" />
								{uiText("home.loginTab")}
							</button>
						{:else}
							<span class="text-[10px] font-semibold text-fg/30">{uiText("statusBanner.online")}</span>
						{/if}
					</div>
				</div>
			{/each}
		{/if}
	</div>
</section>

<!-- Modal: Entrar com Código Direct Join -->
<Modal isOpen={showDirectJoinModal} onClose={() => showDirectJoinModal = false} title={uiText("ui.b3b9347132ed1591")}>
	<div class="flex flex-col gap-4 text-xs">
		<p class="text-fg/70 leading-relaxed">
			{uiText("ui.62da8a4f5eb7248a")} <code class="text-brand-400 font-mono">192.168.1.100:25565</code>{uiText("ui.c0d80d7ff231b165")}
		</p>

		<div class="flex flex-col gap-1.5">
			<label for="direct-join-code-input" class="text-fg/60 font-semibold">{uiText("ui.5088f15c10d00ecf")}</label>
			<input 
				id="direct-join-code-input"
				type="text" 
				bind:value={directJoinCode}
				placeholder={uiText("ui.05bac35dd2233d86")}
				class="w-full bg-bg-subtle border border-fg/10 rounded-xl px-3.5 py-2 text-fg font-mono outline-none focus:border-brand-400"
			/>
		</div>

		<div class="flex items-center justify-end gap-2 pt-2 border-t border-fg/5">
			<Button variant="secondary" size="sm" onclick={() => showDirectJoinModal = false}>
				{uiText("common.cancel")}
			</Button>
			<Button variant="primary" size="sm" onclick={handleDirectJoinSubmit}>
				<Play class="w-3.5 h-3.5 fill-current" />
				{uiText("ui.15a89e5fb2ad184b")}
			</Button>
		</div>
	</div>
</Modal>

<!-- Modal: Abrir Mundo LAN via UPnP -->
<Modal isOpen={showHostLanModal} onClose={() => showHostLanModal = false} title={uiText("ui.6b112e00ff910480")}>
	<div class="flex flex-col gap-4 text-xs">
		<p class="text-fg/70 leading-relaxed">
			{uiText("ui.4651d9e01a9e6880")}
		</p>

		<div class="flex items-center gap-3">
			<div class="flex-1 flex flex-col gap-1.5">
				<label for="host-port-input" class="text-fg/60 font-semibold">{uiText("ui.9bb50d7aeb7eb8be")}</label>
				<input 
					id="host-port-input"
					type="number" 
					bind:value={hostPort}
					class="w-full bg-bg-subtle border border-fg/10 rounded-xl px-3.5 py-2 text-fg font-mono outline-none"
				/>
			</div>

			<div class="self-end">
				<Button variant="primary" size="sm" disabled={isHosting} onclick={handleStartHost}>
					{isHosting ? "Abrindo..." : uiText("ui.6bafe2ace76df9b7")}
				</Button>
			</div>
		</div>

		{#if hostResult?.success}
			<div class="bg-emerald-500/10 border border-emerald-500/20 rounded-xl p-3 flex flex-col gap-2">
				<div class="flex items-center gap-1.5 text-emerald-400 font-bold text-xs">
					<Check class="w-4 h-4" />
					{uiText("servers.port")} {hostPort} {uiText("ui.876d1041e2983499")}
				</div>
				<p class="text-fg/80">{uiText("ui.9fb1c0a3add1d247")}</p>
				<div class="flex items-center gap-2">
					<input 
						type="text" 
						readonly 
						aria-label={uiText("ui.d4cb44ccb0d25924")}
						value={`luxmc://join/${hostResult.externalIp || 'meu-ip'}:${hostPort}`}
						class="flex-1 bg-bg-overlay/40 border border-fg/10 rounded-lg px-2.5 py-1.5 font-mono text-[11px] text-fg"
					/>
					<Button variant="secondary" size="sm" onclick={copyGeneratedLink}>
						{#if hasCopiedCode}
							<Check class="w-3.5 h-3.5 text-emerald-400" />
							{uiText("ui.a8fe0fc805d5fd50")}
						{:else}
							<Copy class="w-3.5 h-3.5" />
							{uiText("common.copy")}
						{/if}
					</Button>
				</div>
			</div>
		{/if}

		<div class="flex items-center justify-end pt-2 border-t border-fg/5">
			<Button variant="secondary" size="sm" onclick={() => showHostLanModal = false}>
				{uiText("statusBanner.dismiss")}
			</Button>
		</div>
	</div>
</Modal>
