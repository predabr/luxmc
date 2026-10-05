<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
    import { luxAccountLogout } from "$lib/api/luxAccount";
    import { cloudAccount } from "$lib/stores/cloudAccount.svelte";
	import { fade, scale } from "svelte/transition";
	import { 
		User, 
		LogOut, 
		Check, 
		X, 
		ShieldCheck, 
		Sparkles, 
		Gamepad2, 
		Shirt, 
		Clock, 
		Layers, 
		Zap,
		Pencil,
		AlertCircle
	} from "lucide-svelte";
	import { account, saveCurrentAccount } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import MicrosoftLogo from "$lib/components/ui/MicrosoftLogo.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { goto } from "$app/navigation";

	let { onClose }: { onClose: () => void } = $props();

	let currentUsername = $derived(account.value?.username || "Gamer");
	let isEditingNick = $state(false);
	let editedNick = $state(account.value?.username || "");
	let isMicrosoft = $derived(Boolean(account.value?.minecraftToken && account.value.minecraftToken.length > 30));

	function handleSaveNick() {
        if (account.value?.id.startsWith("luxmc:")) { toast(uiText("ui.a33b204315ed7848"), "info"); return; }
		const trimmed = editedNick.trim();
		if (!trimmed) {
			toast(uiText("ui.30ebf8e2f3c1346e"), "error");
			return;
		}
		if (trimmed.length < 3 || trimmed.length > 16) {
			toast("O nickname deve ter entre 3 e 16 caracteres.", "error");
			return;
		}
		if (account.value) {
			const updated = {
				...account.value,
				username: trimmed
			};
			account.value = updated;
			void saveCurrentAccount(updated);
		}
		isEditingNick = false;
		toast(`Nickname alterado para ${trimmed}!`, "success");
	}

	async function handleLogout() {
        if (account.value?.id.startsWith("luxmc:")) {
            try { await luxAccountLogout(account.value.id); } catch (cause) { toast(String(cause), "error"); return; }
        }
		account.clear();
		onClose();
		toast(uiText("ui.a15ae849056f492b"), "info");
		goto("/");
	}

	function handleNavigateSkins() {
		onClose();
		goto("/skins");
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			onClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div 
	class="fixed inset-0 z-[10000] flex items-center justify-center bg-bg-overlay/75 backdrop-blur-md p-4 select-none"
	in:fade={{ easing: quintOut, duration: 260 }}
	out:fade={{ easing: quintOut, duration: 220 }}
	onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
	role="dialog"
	aria-modal="true"
	tabindex="-1"
>
	<!-- Modal Card -->
	<div 
		class="w-full max-w-md min-w-[320px] sm:min-w-[440px] rounded-3xl bg-bg-elevated border border-fg/10 shadow-2xl overflow-hidden flex flex-col"
		in:scale={{ easing: backOut, start: 0.95, duration: 220 }}
		out:scale={{ easing: backOut, start: 0.95, duration: 220 }}
	>
        {#if account.value?.id.startsWith("luxmc:")}
            <div class="mx-5 my-3 rounded-xl border border-brand-500/20 bg-brand-500/5 p-3 text-xs">
                <p class="font-semibold text-brand-400">{uiText("ui.40c6a069e0f5c85e")} {cloudAccount.busy ? "Sincronizando…" : cloudAccount.ready ? uiText("ui.5964e609915ee6be") : uiText("ui.cc190ab9f5ec058f")}</p>
                {#if cloudAccount.error}<p role="alert" class="mt-2 text-danger">{cloudAccount.error}</p><button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "mt-2" })} onclick={() => cloudAccount.reload()}>{uiText("ui.35754d098bca5970")}</button>{/if}
            </div>
        {/if}
		<!-- Header Banner -->
		<div class="h-28 w-full relative bg-gradient-to-r from-amber-500/20 via-purple-500/20 to-blue-500/20 p-5 flex items-start justify-between">
			<div class="flex items-center gap-2">
				<span class="bg-bg-overlay/60 backdrop-blur-md border border-fg/10 text-fg/80 text-[10px] font-black uppercase px-3 py-1 rounded-full flex items-center gap-1.5">
					<Zap class="w-3 h-3 text-brand-500" /> {uiText("ui.bfe682a7eb84b3ca")}
				</span>
			</div>
			
			<button 
				type="button"
				class={launcherButton({ variant: "secondary", size: "icon", class: "flex items-center justify-center" })}
				onclick={onClose}
				title={uiText("statusBanner.dismiss")}
			>
				<X class="w-4 h-4" />
			</button>
		</div>

		<!-- Avatar & User Info Row -->
		<div class="px-6 pb-6 pt-0 relative flex flex-col">
			<!-- Overlapping Avatar -->
			<div class="-mt-12 mb-4 flex items-end justify-between">
				<div class="relative">
					<div class="h-20 w-20 rounded-full overflow-hidden bg-bg-subtle border-4 border-border shadow-2xl flex items-center justify-center">
						<img loading="lazy" decoding="async" 
							src={activeSkinStore.current.avatarUrl || account.value?.avatarUrl || (account.value ? "https://mc-heads.net/avatar/" + (account.value.username || account.value.uuid) + "/100" : "https://mc-heads.net/avatar/MHF_Steve/100")} 
							alt={uiText("ui.ca8e826d9c2ec401")} 
							class="w-full h-full object-cover"
							onerror={(e) => {
								const img = e.currentTarget as HTMLImageElement;
								const fallback = account.value?.username ? `https://mc-heads.net/avatar/${account.value.username}/100` : "/grass_block.png";
								if (img.src !== fallback) img.src = fallback;
								else img.src = "/grass_block.png";
							}}
						/>
					</div>
					<div class="absolute -bottom-1 -right-1 h-5 w-5 bg-emerald-500 rounded-full border-2 border-border shadow-md flex items-center justify-center">
						<div class="h-2 w-2 rounded-full bg-fg animate-pulse"></div>
					</div>
				</div>

				<button 
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
					onclick={handleNavigateSkins}
				>
					<Shirt class="w-3.5 h-3.5 text-brand-500" />
					{uiText("ui.3eea64827771de1e")}
				</button>
			</div>

			<!-- Nickname & Account Type -->
			<div class="space-y-1">
				{#if isEditingNick}
					<div class="flex items-center gap-2 mt-1">
						<input 
							type="text" 
							bind:value={editedNick} 
							class="flex-1 bg-bg-subtle border border-brand-500/50 rounded-full px-4 py-2 text-sm font-bold text-fg outline-none focus:ring-1 focus:ring-brand-500"
							placeholder={uiText("ui.b035e0876e9c5cc3")}
							maxlength="16"
							onkeydown={(e) => { if (e.key === "Enter") handleSaveNick(); }}
						/>
						<button 
							type="button"
							class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-1" })}
							onclick={handleSaveNick}
						>
							<Check class="w-3.5 h-3.5 stroke-[3]" />
							{uiText("common.save")}
						</button>
						<button 
							type="button"
							class={launcherButton({ variant: "secondary", size: "icon", class: "flex items-center justify-center" })}
							onclick={() => { isEditingNick = false; editedNick = currentUsername; }}
						>
							<X class="w-3.5 h-3.5" />
						</button>
					</div>
				{:else}
					<div class="flex items-center gap-2">
						<h2 class="text-xl font-black text-fg tracking-tight">{currentUsername}</h2>
						<button 
							type="button" 
							class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
							title={uiText("ui.40407a7585eaae09")}
							onclick={() => { editedNick = currentUsername; isEditingNick = true; }}
						>
							<Pencil class="w-3.5 h-3.5" />
						</button>
					</div>
				{/if}

				<div class="flex items-center gap-2 pt-1">
					{#if isMicrosoft}
						<span class="bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 text-[10px] font-bold px-2.5 py-0.5 rounded-full flex items-center gap-1.5">
							<MicrosoftLogo size={12} /> {uiText("ui.f7d65270bfce73a6")}
						</span>
					{:else}
						<span class="bg-amber-500/15 text-amber-300 border border-amber-500/30 text-[10px] font-bold px-2.5 py-0.5 rounded-full flex items-center gap-1">
							<Gamepad2 class="w-3 h-3" /> {uiText("ui.88a6b56dec5ef600")}
						</span>
					{/if}
					<span class="text-fg/30 text-[10px] font-mono truncate max-w-[170px]" title={account.value?.uuid}>
						{uiText("ui.83f8f6fd45122155")} {account.value?.uuid?.slice(0, 10)}...
					</span>
				</div>
			</div>

			<!-- Player Stats Cards -->
			<div class="grid grid-cols-2 gap-2.5 mt-5">
				<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex flex-col justify-between">
					<span class="text-[10px] font-bold text-fg/40 uppercase flex items-center gap-1">
						<Clock class="w-3 h-3 text-brand-500" /> {uiText("ui.342c23f0d2a98044")}
					</span>
					<div class="text-base font-black text-fg mt-1">{gamingStats.formattedTotalTime}</div>
					<span class="text-[9px] text-fg/30 mt-0.5">{uiText("ui.abbb702f56a38a8d")}</span>
				</div>

				<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex flex-col justify-between">
					<span class="text-[10px] font-bold text-fg/40 uppercase flex items-center gap-1">
						<Layers class="w-3 h-3 text-emerald-400" /> {uiText("ui.2bc187012c72f5ee")}
					</span>
					<div class="text-base font-black text-fg mt-1">{gamingStats.formattedTodayTime}</div>
					<span class="text-[9px] text-fg/30 mt-0.5">{uiText("ui.ac30130b17c38f8f")}</span>
				</div>
			</div>

			<!-- Actions Section -->
			<div class="mt-6 pt-5 border-t border-fg/5 flex flex-col gap-2">
				<button 
					type="button"
					class={launcherButton({ variant: "danger", size: "sm", class: "w-full flex items-center justify-center gap-2" })}
					onclick={handleLogout}
				>
					<LogOut class="w-4 h-4" />
					{uiText("ui.07b09d8a74c4c8f4")}
				</button>
			</div>
		</div>
	</div>
</div>
