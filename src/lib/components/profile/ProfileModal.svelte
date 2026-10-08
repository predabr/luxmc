<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import { luxAccountLogout } from "$lib/api/luxAccount";
    import { cloudAccount } from "$lib/stores/cloudAccount.svelte";
	import { 
		Copy,
		LogOut, 
		X, 
		ShieldCheck, 
		Gamepad2, 
		Shirt, 
		Clock, 
		Layers, 
		Pencil
	} from "lucide-svelte";
	import { account, saveCurrentAccount } from "$lib/stores/account.svelte";
	import { friendsState } from "$lib/stores/friends.svelte";
    import { publicProfile } from "$lib/stores/publicProfile.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import MicrosoftLogo from "$lib/components/ui/MicrosoftLogo.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { goto } from "$app/navigation";
    import Modal from "$lib/components/ui/Modal.svelte";
    import { profiles } from "$lib/stores/profiles.svelte";
    import { authRenameOffline } from "$lib/api/auth";

	let { onClose }: { onClose: () => void } = $props();

	let currentUsername = $derived(account.value?.username || "Gamer");
	let isEditingNick = $state(false);
	let editedNick = $state(account.value?.username || "");
	let isLuxmc = $derived(Boolean(account.value?.id.startsWith("luxmc:")));
    let isMicrosoft = $derived(!isLuxmc && Boolean(account.value?.minecraftToken && account.value.minecraftToken.length > 30));

	async function handleSaveNick() {
        if (account.value?.id.startsWith("luxmc:")) { toast(uiText("ui.a33b204315ed7848"), "info"); return; }
		const trimmed = editedNick.trim();
		if (!trimmed) {
			toast(uiText("ui.30ebf8e2f3c1346e"), "error");
			return;
		}
		if (!/^[A-Za-z0-9_]{3,16}$/.test(trimmed)) {
			toast(uiText("profileCard.nicknameRule"), "error");
			return;
		}
		if (account.value) {
            try { await authRenameOffline(account.value.id, trimmed); }
            catch (error) { toast(String(error), "error"); return; }
			const updated = {
				...account.value,
				username: trimmed
			};
			account.value = updated;
			await saveCurrentAccount(updated);
		}
		isEditingNick = false;
		toast(uiText("profileCard.nicknameSaved"), "success");
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

</script>

<Modal isOpen={true} {onClose} title={uiText("profileCard.title")} maxWidth="max-w-lg">
    <div class="space-y-6">
        <div class="flex items-center gap-4 rounded-2xl border border-brand-500/20 bg-gradient-to-br from-brand-500/10 via-bg/30 to-transparent p-5">
            <div class="h-20 w-20 shrink-0 overflow-hidden rounded-2xl border border-border bg-bg-subtle shadow-soft">
                <img src={friendsState.ownProfile?.portrait || activeSkinStore.current.avatarUrl || account.value?.avatarUrl || "/grass_block.png"} alt={uiText("ui.ca8e826d9c2ec401")} class="h-full w-full object-cover {friendsState.ownProfile?.portrait ? '' : '[image-rendering:pixelated]'}" onerror={e => { const image = e.currentTarget as HTMLImageElement; if (!image.src.endsWith('/grass_block.png')) image.src = '/grass_block.png'; }} />
            </div>
            <div class="min-w-0 flex-1">
                <p class="mb-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-fg-subtle">{uiText("profileCard.identity")}</p>
                <h2 class="truncate text-2xl font-bold tracking-tight text-fg">{friendsState.ownProfile?.displayName || currentUsername}</h2>
                {#if friendsState.ownProfile?.role === "owner"}<span class="mt-2 inline-flex rounded-lg border border-brand-400/30 bg-brand-400/10 px-2 py-1 text-xs font-semibold text-brand-400">{uiText("ownerTools.badge")}</span>{/if}
                <p class="mt-2 inline-flex items-center gap-2 text-xs font-medium text-brand-400">
                    {#if isMicrosoft}<MicrosoftLogo size={14} />{uiText("ui.f7d65270bfce73a6")}
                    {:else if isLuxmc}<ShieldCheck class="h-4 w-4" />{uiText("profileCard.luxmcAccount")}
                    {:else}<Gamepad2 class="h-4 w-4" />{uiText("profileCard.offlineAccount")}{/if}
                </p>
            </div>
        </div>
        {#if isLuxmc}
            <div class="rounded-xl border border-border bg-bg/35 p-4 text-sm text-fg-muted" role="status">
                <p>{cloudAccount.busy ? uiText("profileCard.syncing") : cloudAccount.ready ? uiText("profileCard.synced") : uiText("profileCard.syncPending")}</p>
                {#if cloudAccount.error}<p class="mt-2 text-danger">{cloudAccount.error}</p><button type="button" class={launcherButton({variant: "secondary", size: "sm", class: "mt-3"})} onclick={() => cloudAccount.reload()}>{uiText("ui.35754d098bca5970")}</button>{/if}
            </div>
        {/if}
        <div class="grid grid-cols-3 gap-3">
            <div class="rounded-2xl border border-border bg-bg/35 p-4"><Clock class="mb-3 h-5 w-5 text-brand-400" /><p class="text-[10px] font-semibold uppercase tracking-wide text-fg-subtle">{uiText("ui.342c23f0d2a98044")}</p><p class="mt-1 text-lg font-bold text-fg">{gamingStats.formattedTotalTime}</p></div>
            <div class="rounded-2xl border border-border bg-bg/35 p-4"><Gamepad2 class="mb-3 h-5 w-5 text-success" /><p class="text-[10px] font-semibold uppercase tracking-wide text-fg-subtle">{uiText("profileCard.today")}</p><p class="mt-1 text-lg font-bold text-fg">{gamingStats.formattedTodayTime}</p></div>
            <div class="rounded-2xl border border-border bg-bg/35 p-4"><Layers class="mb-3 h-5 w-5 text-brand-400" /><p class="text-[10px] font-semibold uppercase tracking-wide text-fg-subtle">{uiText("instances.title")}</p><p class="mt-1 text-lg font-bold text-fg">{profiles.list.length}</p></div>
        </div>
        <p class="rounded-xl border border-brand-500/15 bg-brand-500/5 p-4 text-xs leading-relaxed text-fg-muted">{isMicrosoft ? uiText("profileCard.microsoftSkins") : isLuxmc ? uiText("profileCard.luxmcSkins") : uiText("profileCard.offlineSkins")}</p>
        {#if !isMicrosoft && !isLuxmc}
            {#if isEditingNick}
                <div class="flex flex-wrap gap-2"><input aria-label={uiText("ui.b035e0876e9c5cc3")} class="min-w-0 flex-1 rounded-xl border border-border bg-bg/40 px-3 py-2 text-fg" bind:value={editedNick} maxlength="16" onkeydown={e => { if (e.key === 'Enter') handleSaveNick(); }} /><button type="button" class={launcherButton({variant: "primary", size: "sm"})} onclick={handleSaveNick}>{uiText("common.save")}</button><button type="button" class={launcherButton({variant: "secondary", size: "icon"})} aria-label={uiText("common.cancel")} onclick={() => isEditingNick = false}><X class="h-4 w-4" /></button></div>
            {:else}<button type="button" class={launcherButton({variant: "ghost", size: "sm"})} onclick={() => {editedNick = currentUsername; isEditingNick = true;}}><Pencil class="h-4 w-4" />{uiText("ui.40407a7585eaae09")}</button>{/if}
        {/if}
        <div class="space-y-3"><button type="button" disabled={!friendsState.me} class={launcherButton({variant:'secondary',block:true})} onclick={() => { onClose(); publicProfile.edit(); }}><Pencil class="h-4 w-4" />{uiText('publicProfile.edit')}</button>{#if friendsState.ownProfile?.role === 'owner'}<a href="/owner-tools" class={launcherButton({variant:'secondary',block:true})} onclick={onClose}><ShieldCheck class="h-4 w-4" />{uiText('ownerTools.title')}</a>{/if}</div>
        <div class="grid grid-cols-2 gap-3"><button type="button" class={launcherButton({variant: "primary", size: "lg"})} onclick={handleNavigateSkins}><Shirt class="h-4 w-4" />{uiText("ui.3eea64827771de1e")}</button><button type="button" class={launcherButton({variant: "secondary", size: "lg"})} onclick={async () => { if (account.value?.uuid) { await navigator.clipboard.writeText(account.value.uuid); toast(uiText("profileCard.idCopied"), "success"); } }}><Copy class="h-4 w-4" />{uiText("profileCard.copyId")}</button></div>
        <div class="border-t border-border pt-4"><button type="button" class={launcherButton({variant: "danger", size: "sm", class: "w-full"})} onclick={handleLogout}><LogOut class="h-4 w-4" />{uiText("ui.07b09d8a74c4c8f4")}</button></div>
    </div>
</Modal>
