<script lang="ts">
	import { LogOut, LogIn, User, Settings, CircleCheck, CircleAlert, CircleX } from "lucide-svelte";
	import Button from "./Button.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	let showMenu = $state(false);
	let skinUrl = $derived(account.value?.id ? `https://crafatar.com/avatars/${account.value.id}?size=64&overlay` : "");

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

	function handleLogout() {
		account.clear();
		showMenu = false;
	}

	function toggleMenu() {
		showMenu = !showMenu;
	}

	function handleSkinError(e: Event) {
		const img = e.target as HTMLImageElement;
		img.src = "data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 24 24%22%3E%3Crect fill=%22%2314b8a6%22 width=%2224%22 height=%2224%22/%3E%3C/svg%3E";
	}
</script>

<div class="relative">
	{#if account.value}
		{@const status = statusConfig}
		<button
			onclick={toggleMenu}
			class="flex items-center gap-2 h-9 px-2 rounded-lg bg-bg-elevated border border-border hover:border-brand-400 hover:bg-bg-subtle transition-all duration-200"
			title={account.value.username}
		>
			<span class="relative inline-flex">
				<img
					src={skinUrl}
					alt={account.value.username}
					class="w-6 h-6 rounded border border-border-strong object-cover"
					onerror={handleSkinError}
				/>
				<span
					class="absolute -bottom-0.5 -right-0.5 inline-block h-2.5 w-2.5 rounded-full border-2"
					style="background: {status.color}; border-color: rgb(var(--bg-elevated));"
					aria-label={status.label}
				></span>
			</span>
			<span class="text-sm text-fg truncate max-w-[120px]">{account.value.username}</span>
		</button>

		{#if showMenu}
			<div class="absolute top-10 right-0 mt-1 w-56 rounded-lg bg-bg-elevated border border-border shadow-lg z-50 animate-fade-in">
				<div class="p-3 border-b border-border">
					<p class="text-xs text-fg-muted uppercase tracking-wide font-semibold">{t("accountIndicator.account")}</p>
					<p class="text-sm text-fg font-medium mt-1">{account.value.username}</p>
					<div class="mt-2 flex items-center gap-2 text-xs" style="color: {status.color};">
						<status.Icon size={14} />
						<span>{status.label}</span>
					</div>
				</div>
				<div class="p-2 space-y-1">
					<a
						href="/settings"
						class="flex items-center gap-2 px-3 py-2 rounded-md text-sm text-fg hover:bg-bg-subtle transition-colors"
						onclick={() => (showMenu = false)}
					>
						<Settings size={16} />
						{t("accountIndicator.settings")}
					</a>
					<button
						onclick={handleLogout}
						class="w-full flex items-center gap-2 px-3 py-2 rounded-md text-sm text-red-400 hover:bg-red-500/10 transition-colors"
					>
						<LogOut size={16} />
						{t("accountIndicator.logout")}
					</button>
				</div>
			</div>
		{/if}
	{:else}
		<div class="flex items-center gap-2">
			<User size={18} class="text-fg-muted" />
			<span class="text-sm text-fg-muted">{t("accountIndicator.notLoggedIn")}</span>
		</div>
	{/if}
</div>

<style>
	:global(.animate-fade-in) {
		animation: fade-in 0.15s ease-out;
	}

	@keyframes fade-in {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
