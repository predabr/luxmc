<script lang="ts">
	import { onMount } from "svelte";
	import { LogOut, LogIn, User, Settings, CircleCheck, CircleAlert, CircleX, Users, Plus, Check } from "lucide-svelte";
	import Button from "./Button.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { authAccounts } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

	let showMenu = $state(false);
	let savedAccounts = $state<Array<{
		id: string;
		username: string;
		uuid: string;
		accessToken: string | null;
		expiresAt: string | null;
	}>>([]);

	let skinUrl = $derived(
		account.value?.username 
			? `https://mc-heads.net/avatar/${account.value.username}/64` 
			: "/grass_block.png"
	);

	type AccountStatus = "online" | "expiring" | "expired" | "offline";

	function computeStatus(): AccountStatus {
		const a = account.value;
		if (!a) return "offline";
		if (a.minecraftToken && a.minecraftToken.startsWith("luxmc-")) return "offline";
		if (!a.expiresAt || a.expiresAt === 0) return "online";
		const remaining = a.expiresAt - Date.now();
		if (remaining <= 0) return "expired";
		if (remaining < 1000 * 60 * 60 * 24) return "expiring";
		return "online";
	}

	const accountStatus = $derived(computeStatus());

	const statusConfig = $derived.by(() => {
		switch (accountStatus) {
			case "online":
				return { color: "rgb(34, 197, 94)", label: t("accountIndicator.online"), Icon: CircleCheck };
			case "expiring":
				return { color: "rgb(234, 179, 8)", label: t("accountIndicator.expiring"), Icon: CircleAlert };
			case "expired":
				return { color: "rgb(239, 68, 68)", label: t("accountIndicator.expired"), Icon: CircleX };
			case "offline":
				return { color: "rgb(95, 105, 130)", label: t("accountIndicator.offline"), Icon: User };
		}
	});

	async function loadAccounts() {
		try {
			savedAccounts = await authAccounts().catch(() => []);
		} catch {
			savedAccounts = [];
		}
	}

	onMount(() => {
		void loadAccounts();
	});

	function handleLogout() {
		account.clear();
		showMenu = false;
		toast("Conta desconectada.", "info");
	}

	function toggleMenu() {
		showMenu = !showMenu;
		if (showMenu) {
			void loadAccounts();
		}
	}

	function handleSkinError(e: Event) {
		const img = e.target as HTMLImageElement;
		img.src = "/grass_block.png";
	}

	function handleSwitchAccount(acc: { id: string; username: string; uuid: string; accessToken: string | null; expiresAt: string | null }) {
		const newAcc = {
			id: acc.id,
			username: acc.username,
			uuid: acc.uuid,
			minecraftToken: acc.accessToken || "",
			expiresAt: acc.expiresAt ? new Date(acc.expiresAt).getTime() : 0
		};
		localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
		account.account = newAcc;
		showMenu = false;
		toast(`Alternado para a conta ${acc.username}!`, "success");
	}
</script>

<div class="relative">
	{#if account.value}
		{@const status = statusConfig}
		<button
			onclick={toggleMenu}
			class="flex items-center gap-2 h-9 px-2.5 rounded-full bg-[#18191c] border border-white/10 hover:border-brand-500/50 hover:bg-[#202127] transition-all cursor-pointer shadow-sm"
			title={account.value.username}
		>
			<span class="relative inline-flex">
				<img
					src={skinUrl}
					alt={account.value.username}
					class="w-6 h-6 rounded-md border border-white/10 object-cover bg-black/40"
					onerror={handleSkinError}
				/>
				<span
					class="absolute -bottom-0.5 -right-0.5 inline-block h-2 w-2 rounded-full ring-2 ring-[#18191c]"
					style="background: {status.color};"
					aria-label={status.label}
				></span>
			</span>
			<span class="text-xs font-bold text-white truncate max-w-[120px]">{account.value.username}</span>
		</button>

		{#if showMenu}
			<div class="absolute top-11 right-0 w-64 rounded-2xl bg-[#141518] border border-white/10 shadow-2xl z-50 animate-fade-in overflow-hidden">
				<div class="p-3.5 border-b border-white/5 bg-[#18191c]/80">
					<div class="flex items-center justify-between">
						<span class="text-[10px] text-white/40 uppercase tracking-wider font-extrabold">{t("accountIndicator.account")}</span>
						<span class="text-[9px] font-black uppercase px-1.5 py-0.5 rounded-full bg-white/5 text-white/60">
							{account.value.minecraftToken && !account.value.minecraftToken.startsWith('luxmc-') && account.value.minecraftToken.length > 50 ? 'Microsoft' : 'Offline'}
						</span>
					</div>
					<div class="flex items-center gap-2.5 mt-2">
						<img 
							src={skinUrl} 
							alt={account.value.username} 
							class="w-8 h-8 rounded-lg border border-white/10 object-cover bg-black/30"
							onerror={handleSkinError}
						/>
						<div class="min-w-0">
							<p class="text-xs text-white font-black truncate">{account.value.username}</p>
							<div class="flex items-center gap-1.5 text-[10px] mt-0.5" style="color: {status.color};">
								<status.Icon size={12} />
								<span>{status.label}</span>
							</div>
						</div>
					</div>
				</div>

				<!-- Saved accounts list -->
				{#if savedAccounts.length > 1}
					<div class="p-2 border-b border-white/5 max-h-36 overflow-y-auto custom-scrollbar">
						<div class="text-[9px] font-extrabold text-white/40 uppercase px-2 py-1">Trocar Conta</div>
						{#each savedAccounts as acc}
							{@const isActive = acc.username.toLowerCase() === account.value.username.toLowerCase()}
							<button 
								type="button"
								class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-xl text-xs transition-all cursor-pointer {isActive ? 'bg-brand-500/10 text-brand-500 font-bold border border-brand-500/20' : 'text-white/60 hover:text-white hover:bg-white/5'}"
								onclick={() => handleSwitchAccount(acc)}
							>
								<div class="flex items-center gap-2 min-w-0">
									<img 
										src={`https://mc-heads.net/avatar/${acc.username}/32`} 
										alt={acc.username} 
										class="w-5 h-5 rounded object-cover"
										onerror={handleSkinError}
									/>
									<span class="truncate">{acc.username}</span>
								</div>
								{#if isActive}
									<Check class="w-3.5 h-3.5 text-brand-500" />
								{/if}
							</button>
						{/each}
					</div>
				{/if}

				<div class="p-2 space-y-1">
					<a
						href="/"
						class="flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-bold text-white/70 hover:text-white hover:bg-white/5 transition-colors"
						onclick={() => (showMenu = false)}
					>
						<Plus size={14} class="text-brand-500" />
						Adicionar Outra Conta
					</a>
					<a
						href="/settings"
						class="flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-bold text-white/70 hover:text-white hover:bg-white/5 transition-colors"
						onclick={() => (showMenu = false)}
					>
						<Settings size={14} />
						{t("accountIndicator.settings")}
					</a>
					<button
						onclick={handleLogout}
						class="w-full flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-bold text-red-400 hover:bg-red-500/10 transition-colors cursor-pointer"
					>
						<LogOut size={14} />
						{t("accountIndicator.logout")}
					</button>
				</div>
			</div>
		{/if}
	{:else}
		<a 
			href="/"
			class="flex items-center gap-1.5 h-9 px-3 rounded-full bg-brand-500 hover:bg-brand-400 text-black text-xs font-black transition-all shadow-sm active:scale-95"
		>
			<LogIn size={14} />
			<span>Entrar</span>
		</a>
	{/if}
</div>
