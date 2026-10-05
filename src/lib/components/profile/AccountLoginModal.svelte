<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
    import LuxAccountForm from "./LuxAccountForm.svelte";
	import { fade, scale } from "svelte/transition";
	import { 
		User, 
		Check, 
		X, 
		Sparkles, 
		LogIn, 
		Gamepad2,
		ShieldCheck,
		AlertCircle
	} from "lucide-svelte";
	import { account, saveCurrentAccount } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import MicrosoftLogo from "$lib/components/ui/MicrosoftLogo.svelte";
	import { authLogin, authOfflineLogin, type AuthAccount } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";

	let { 
		isOpen = $bindable(false), 
		onClose,
		onAccountAdded
	}: { 
		isOpen: boolean; 
		onClose?: () => void;
		onAccountAdded?: (acc: AuthAccount) => void;
	} = $props();

	let tab = $state<"luxmc" | "offline" | "microsoft">("luxmc");
	let offlineUsername = $state("");
	let isLoggingIn = $state(false);
	let errorMsg = $state<string | null>(null);

	function updateActiveAppearance(acc: { uuid: string; username: string; skinUrl?: string; skinVariant?: string; capeUrl?: string }) {
		const skinUrl = acc.skinUrl || `https://minotar.net/skin/${acc.username}`;
		activeSkinStore.setSkin({
			id: acc.uuid,
			name: acc.username,
			url: `https://mc-heads.net/body/${acc.username}/300`,
			skinUrl,
			avatarUrl: `https://mc-heads.net/avatar/${acc.username}/100`,
			type: acc.skinVariant?.toLowerCase() === "slim" ? "alex" : "steve",
			hasCape: Boolean(acc.capeUrl),
			capeType: acc.capeUrl ? "custom" : "none",
			customCapeUrl: acc.capeUrl || ""
		});
	}

	async function handleOfflineLogin() {
		if (isLoggingIn) return;
		const trimmed = offlineUsername.trim();
		if (!trimmed) {
			errorMsg = "Por favor, digite um nickname.";
			return;
		}
		if (trimmed.length < 3 || trimmed.length > 16) {
			errorMsg = "O nickname deve ter entre 3 e 16 caracteres.";
			return;
		}
		if (!/^[a-zA-Z0-9_]+$/.test(trimmed)) {
			errorMsg = uiText("ui.ed9583ced94dc1a4");
			return;
		}

		isLoggingIn = true;
		errorMsg = null;
		try {
			const acc = await authOfflineLogin(trimmed);
			account.value = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken,
				expiresAt: acc.expiresAt ? (acc.expiresAt < 1e11 ? acc.expiresAt * 1000 : acc.expiresAt) : 0,
				skinUrl: acc.skinUrl ?? null,
				skinVariant: acc.skinVariant ?? "classic",
				capeUrl: acc.capeUrl ?? null
			};
			void saveCurrentAccount(account.value);
			updateActiveAppearance(acc);
			toast(uiText("ui.953b1b10d0d549e9", {arg0: (trimmed)}), "success");
			onAccountAdded?.(acc);
			isLoggingIn = false;
			handleClose();
		} catch (e) {
			errorMsg = uiText("ui.9538dd9126533a1c") + String(e);
			toast(errorMsg, "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleMicrosoftLogin() {
		if (isLoggingIn) return;
		isLoggingIn = true;
		errorMsg = null;
		try {
			const acc = await authLogin();
			account.value = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken,
				expiresAt: acc.expiresAt ? (acc.expiresAt < 1e11 ? acc.expiresAt * 1000 : acc.expiresAt) : 0,
				skinUrl: acc.skinUrl ?? null,
				skinVariant: acc.skinVariant ?? "classic",
				capeUrl: acc.capeUrl ?? null
			};
			void saveCurrentAccount(account.value);
			updateActiveAppearance(acc);
			toast(uiText("ui.fbb71389641a9e1d", {arg0: (acc.username)}), "success");
			onAccountAdded?.(acc as AuthAccount);
			isLoggingIn = false;
			handleClose();
		} catch (e) {
			errorMsg = uiText("ui.62406a19962ea989") + String(e);
			toast(errorMsg, "error");
		} finally {
			isLoggingIn = false;
		}
	}

	function handleClose() {
		if (isLoggingIn) return;
		offlineUsername = "";
		errorMsg = null;
		isLoggingIn = false;
		isOpen = false;
		onClose?.();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") handleClose();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div 
		class="fixed inset-0 z-[10001] flex items-center justify-center bg-bg-overlay/75 backdrop-blur-md p-4 select-none"
		in:fade={{ easing: quintOut, duration: 220 }}
		out:fade={{ easing: quintOut, duration: 180 }}
		onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div 
			class="auth-surface w-full max-w-[480px] max-h-[90dvh] overflow-y-auto rounded-3xl bg-bg-elevated border border-fg/10 flex flex-col"
			in:scale={{ easing: backOut, start: 0.95, duration: 240 }}
			out:scale={{ easing: backOut, start: 0.95, duration: 180 }}
		>
			<!-- Header -->
			<div class="p-5 border-b border-fg/5 bg-bg-elevated flex items-center justify-between">
				<div class="flex items-center gap-2.5">
					<div class="w-8 h-8 rounded-xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center text-brand-500">
						<Gamepad2 class="w-4 h-4" />
					</div>
					<div>
						<h3 class="text-sm font-black text-fg">{uiText("ui.d1f39e30f9b200a4")}</h3>
						<p class="text-[10px] text-fg/40">{uiText("ui.a62914dff8852842")}</p>
					</div>
				</div>
				<button 
					type="button" 
					class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center justify-center" })}
					onclick={handleClose}
					disabled={isLoggingIn}
					aria-label={uiText("common.close")}
				>
					✕
				</button>
			</div>

			<!-- Tab Switcher (Offline vs Microsoft) -->
			<div class="p-4">
				<div class="grid grid-cols-3 gap-2 p-1 bg-bg-elevated rounded-2xl border border-fg/5">
                    <button type="button" class="launcher-tab {tab === 'luxmc' ? 'bg-brand-500/15 text-fg' : 'text-fg-muted'}" aria-pressed={tab === "luxmc"} disabled={isLoggingIn} onclick={() => tab = "luxmc"}>Luxmc</button>
					<button 
						type="button"
						class="launcher-tab flex items-center justify-center gap-2 {tab === 'offline' ? 'bg-brand-500/15 text-fg' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
						aria-pressed={tab === "offline"}
						disabled={isLoggingIn}
						onclick={() => { tab = 'offline'; errorMsg = null; }}
					>
						<User class="w-3.5 h-3.5" />
						{uiText("ui.2793472a35db2b80")}
					</button>
					<button 
						type="button"
						class="launcher-tab flex items-center justify-center gap-2 {tab === 'microsoft' ? 'bg-brand-500/15 text-fg' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
						aria-pressed={tab === "microsoft"}
						disabled={isLoggingIn}
						onclick={() => { tab = 'microsoft'; errorMsg = null; }}
					>
						<MicrosoftLogo size={14} />
						Microsoft
					</button>
				</div>

				<!-- Offline Tab Body -->
				{#if tab === 'luxmc'}
                    <LuxAccountForm bind:busy={isLoggingIn} onAuthenticated={(value) => { onAccountAdded?.(value); handleClose(); }} />
                {:else if tab === 'offline'}
					<form onsubmit={(e) => { e.preventDefault(); handleOfflineLogin(); }} class="space-y-4 pt-4">
						<div class="space-y-1.5">
							<label for="offline-nick" class="text-xs font-bold text-fg/70 block">{uiText("ui.3984c02f20c59f47")}</label>
							<div class="relative">
								<input 
									id="offline-nick"
									type="text"
									bind:value={offlineUsername}
									placeholder={uiText("ui.7cecfb21a92e00f4")}
									maxlength="16"
									class="w-full h-11 px-4 rounded-xl bg-bg-subtle border border-fg/10 text-xs font-bold text-fg outline-none focus:border-brand-500 transition-[color,background-color,border-color,box-shadow,transform,opacity] placeholder:text-fg/30"
								/>
							</div>
							<p class="text-[10px] text-fg/40">{uiText("ui.60b07bd459a3c12a")}</p>
						</div>

						{#if errorMsg}
							<div class="p-3 bg-red-500/10 border border-red-500/25 rounded-xl text-xs text-red-400 font-bold flex items-center gap-2">
								<AlertCircle class="w-4 h-4 shrink-0" />
								<span>{errorMsg}</span>
							</div>
						{/if}

						<button 
							type="submit"
							disabled={isLoggingIn || !offlineUsername.trim()}
							class={launcherButton({ variant: "primary", size: "sm", class: "w-full disabled:opacity-50 flex items-center justify-center gap-2" })}
						>
							{#if isLoggingIn}
								<div class="w-4 h-4 rounded-full border-2 border-bg-overlay border-t-transparent animate-spin"></div>
								{uiText("ui.a89d514549530e6e")}
							{:else}
								<Check class="w-4 h-4 stroke-[3]" />
								{uiText("ui.6c88e3c5a5292efb")}
							{/if}
						</button>
					</form>
				{:else}
					<!-- Microsoft Tab Body -->
					<div class="space-y-4 pt-4 text-center">
						<div class="p-4 bg-emerald-500/10 border border-emerald-500/20 rounded-2xl space-y-1 text-left">
							<span class="text-xs font-bold text-emerald-400 block flex items-center gap-1.5">
								<ShieldCheck class="w-4 h-4" /> {uiText("ui.43663057e1279e6e")}
							</span>
							<p class="text-[10px] text-fg/60 leading-relaxed">
								{uiText("ui.1ed3a3f3494e993b")}
							</p>
						</div>

						{#if errorMsg}
							<div class="p-3 bg-red-500/10 border border-red-500/25 rounded-xl text-xs text-red-400 font-bold flex items-center gap-2 text-left">
								<AlertCircle class="w-4 h-4 shrink-0" />
								<span>{errorMsg}</span>
							</div>
						{/if}

						<button 
							type="button"
							disabled={isLoggingIn}
							onclick={handleMicrosoftLogin}
							aria-busy={isLoggingIn}
							class={launcherButton({ variant: "microsoft", size: "hero", class: "w-full" })}
						>
							{#if isLoggingIn}
								<div class="w-4 h-4 rounded-full border-2 border-bg-overlay border-t-transparent animate-spin"></div>
								{uiText("ui.acaa8fe1f2207472")}
							{:else}
								<MicrosoftLogo size={20} />
								{uiText("ui.e9a973d6fbcd7875")}
							{/if}
						</button>
						{#if isLoggingIn}<p role="status" class="text-xs text-fg/60 leading-relaxed">{uiText("design.browserLoginHint")}</p>{/if}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
