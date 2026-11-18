<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { focusTrap } from "$lib/utils/focusTrap";
    import { backOut, quintOut } from "svelte/easing";
    import LuxAccountForm from "./LuxAccountForm.svelte";
	import { fade, scale } from "svelte/transition";
	import { 
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
	import { authLogin, type AuthAccount } from "$lib/api";
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

	let tab = $state<"luxmc" | "microsoft">("luxmc");
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
			onAccountAdded?.(acc as AuthAccount);
			isLoggingIn = false;
			handleClose();
		} catch (e) {
			const detail = String(e).replace(/^(?:Error:\s*)?not found:\s*/i, "");
			errorMsg = uiText("ui.62406a19962ea989") + detail;
			toast(errorMsg, "error");
		} finally {
			isLoggingIn = false;
		}
	}

	function handleClose() {
		if (isLoggingIn) return;
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
        role="presentation"
        onkeydown={(event) => { if (event.key === 'Escape') handleClose(); }}
	>
		<div 
			use:focusTrap role="dialog" aria-modal="true" aria-labelledby="account-login-title" tabindex="-1" class="auth-surface w-full max-w-[600px] max-h-[90dvh] overflow-y-auto rounded-3xl bg-bg-elevated border border-fg/10 flex flex-col"
			in:scale={{ easing: backOut, start: 0.95, duration: 240 }}
			out:scale={{ easing: backOut, start: 0.95, duration: 180 }}
		>
			<!-- Header -->
			<div class="p-6 border-b border-border bg-bg-elevated flex items-center justify-between">
				<div class="flex items-center gap-2.5">
					<div class="w-8 h-8 rounded-xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center text-brand-500">
						<Gamepad2 class="w-4 h-4" />
					</div>
					<div>
						<h3 id="account-login-title" class="text-xl font-semibold tracking-tight text-fg">{uiText("ui.d1f39e30f9b200a4")}</h3>
						<p class="mt-1 text-xs leading-relaxed text-fg-muted">{uiText("ui.a62914dff8852842")}</p>
					</div>
				</div>
				<button 
					type="button" 
					class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center justify-center" })}
					onclick={handleClose}
					disabled={isLoggingIn}
					aria-label={uiText("common.close")}
				>
					<X class="h-4 w-4" />
				</button>
			</div>

			<!-- Tab Switcher (Luxmc vs Microsoft) -->
			<div class="p-6">
                <div class="grid grid-cols-2 gap-2" aria-label={uiText('loginDesign.method')}>
                    {#each ['luxmc', 'microsoft'] as method}
                        <button type="button" aria-label={method === 'microsoft' ? 'Microsoft' : 'Luxmc'} aria-pressed={tab === method} disabled={isLoggingIn} onclick={() => { tab = method as typeof tab; errorMsg = null; }} class="flex min-w-0 flex-col items-start gap-3 rounded-2xl border p-3 text-left transition-colors {tab === method ? 'border-brand-400/50 bg-brand-500/10' : 'border-border bg-bg-subtle hover:border-fg/25'}">
                            {#if method === 'microsoft'}<MicrosoftLogo size={19} />{:else}<img src="/logo.png" alt="" class="h-6 w-6 object-contain" />{/if}
                            <span class="text-xs font-semibold text-fg">{method === 'microsoft' ? 'Microsoft' : 'Luxmc'}</span><span class="text-[10px] leading-relaxed text-fg-muted">{uiText(`loginDesign.${method}`)}</span>
                        </button>
                    {/each}
                </div>

				{#if tab === 'luxmc'}
                    <LuxAccountForm bind:busy={isLoggingIn} onAuthenticated={(value) => { onAccountAdded?.(value); handleClose(); }} />
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
						{#if isLoggingIn}<p role="status" class="text-xs text-fg/60 leading-relaxed">{uiText("ui.7b258b8a8dbb4fc2")}</p>{/if}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
