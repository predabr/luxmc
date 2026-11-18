<script lang="ts">
    import MembershipCard from "./MembershipCard.svelte";
    import AccountBadges from "$lib/components/ui/AccountBadges.svelte";
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
	}

	async function handleLogout() {
        if (account.value?.id.startsWith("luxmc:")) {
            try { await luxAccountLogout(account.value.id); } catch (cause) { toast(String(cause), "error"); return; }
        }
		account.clear();
		onClose();
		goto("/");
	}

	function handleNavigateSkins() {
		onClose();
		goto("/skins");
	}

</script>

<Modal isOpen={true} {onClose} title={uiText("profileCard.title")} maxWidth="max-w-lg">
    <div class="space-y-4">
        <section class="overflow-hidden rounded-2xl border border-border bg-bg-subtle/40">
            <div class="relative h-24">
                {#if friendsState.ownProfile?.banner}
                    <img src={friendsState.ownProfile.banner} alt="" class="h-full w-full object-cover" />
                {:else}
                    <div class="profile-modal-cover h-full w-full" aria-hidden="true"></div>
                {/if}
                <div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-bg-elevated/20 to-transparent"></div>
                <button type="button" disabled={!friendsState.me} class="absolute right-3 top-3 inline-flex items-center gap-1.5 rounded-full border border-fg/10 bg-bg-overlay/70 px-3 py-1.5 text-[11px] font-semibold text-fg backdrop-blur transition-colors hover:border-brand-400/45 hover:text-brand-300 disabled:opacity-50" onclick={() => { onClose(); publicProfile.edit(); }}><Pencil class="h-3.5 w-3.5" />{uiText('publicProfile.edit')}</button>
            </div>
            <div class="relative flex items-end gap-4 px-4 pb-4">
                <div class="-mt-9 h-[74px] w-[74px] shrink-0 overflow-hidden rounded-2xl border-[3px] border-bg-elevated bg-bg-subtle shadow-soft">
                    <img src={friendsState.ownProfile?.portrait || activeSkinStore.current.avatarUrl || account.value?.avatarUrl || "/grass_block.png"} alt={uiText("ui.ca8e826d9c2ec401")} class="h-full w-full object-cover {friendsState.ownProfile?.portrait ? '' : '[image-rendering:pixelated]'}" onerror={e => { const image = e.currentTarget as HTMLImageElement; if (!image.src.endsWith('/grass_block.png')) image.src = '/grass_block.png'; }} />
                </div>
                <div class="min-w-0 flex-1 pb-1">
                    <h3 class="truncate text-lg font-bold leading-tight tracking-tight text-fg">{friendsState.ownProfile?.displayName || currentUsername}</h3>
                    <p class="mt-0.5 truncate text-[11px] text-fg-muted">@{currentUsername}</p>
                </div>
            </div>
            <div class="space-y-2 px-4 pb-4">
                <div class="flex flex-wrap items-center gap-2"><AccountBadges role={friendsState.ownProfile?.role} plan={friendsState.ownProfile?.membership?.plan} tags={friendsState.ownProfile?.tags} size="sm" /><span class="ml-auto inline-flex items-center gap-1.5 rounded-full border border-border bg-bg/50 px-2.5 py-1 text-[10px] font-medium text-fg-muted">{#if isMicrosoft}<MicrosoftLogo size={12} />{uiText("ui.f7d65270bfce73a6")}{:else if isLuxmc}<ShieldCheck class="h-3 w-3 text-brand-400" />{uiText("profileCard.luxmcAccount")}{:else}<Gamepad2 class="h-3 w-3" />{uiText("profileCard.offlineAccount")}{/if}</span></div>
                {#if friendsState.ownProfile?.status}<p class="rounded-lg border border-brand-400/20 bg-brand-500/[0.08] px-3 py-2 text-xs leading-5 text-fg">{friendsState.ownProfile.status}</p>{/if}
            </div>
        </section>

        <div class="grid grid-cols-3 divide-x divide-border rounded-2xl border border-border bg-bg/35 py-3">
            <div class="px-3"><p class="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wide text-fg-subtle"><Clock class="h-3.5 w-3.5 text-brand-400" />{uiText("ui.342c23f0d2a98044")}</p><p class="mt-1 text-base font-bold text-fg">{gamingStats.formattedTotalTime}</p></div>
            <div class="px-3"><p class="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wide text-fg-subtle"><Gamepad2 class="h-3.5 w-3.5 text-success" />{uiText("profileCard.today")}</p><p class="mt-1 text-base font-bold text-fg">{gamingStats.formattedTodayTime}</p></div>
            <div class="px-3"><p class="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wide text-fg-subtle"><Layers class="h-3.5 w-3.5 text-brand-400" />{uiText("instances.title")}</p><p class="mt-1 text-base font-bold text-fg">{profiles.list.length}</p></div>
        </div>

        {#if isLuxmc}
            <div class="rounded-xl border border-border bg-bg/35 px-3 py-2 text-[11px] text-fg-muted" role="status">
                <p>{cloudAccount.busy ? uiText("profileCard.syncing") : cloudAccount.ready ? uiText("profileCard.synced") : uiText("profileCard.syncPending")}</p>
                {#if cloudAccount.error}<p class="mt-1.5 text-danger">{cloudAccount.error}</p><button type="button" class={launcherButton({variant: "secondary", size: "sm", class: "mt-2"})} onclick={() => cloudAccount.reload()}>{uiText("ui.35754d098bca5970")}</button>{/if}
            </div>
        {/if}

        <MembershipCard />

        <div class="space-y-2">
            <div class="grid grid-cols-2 gap-2"><button type="button" class={launcherButton({variant: "primary"})} onclick={handleNavigateSkins}><Shirt class="h-4 w-4" />{uiText("ui.3eea64827771de1e")}</button><button type="button" class={launcherButton({variant: "secondary"})} onclick={async () => { if (account.value?.uuid) { await navigator.clipboard.writeText(account.value.uuid);  } }}><Copy class="h-4 w-4" />{uiText("profileCard.copyId")}</button></div>
            {#if friendsState.ownProfile?.role === 'owner'}<a href="/owner-tools" class={launcherButton({variant: "secondary", block: true})} onclick={onClose}><ShieldCheck class="h-4 w-4" />{uiText('ownerTools.title')}</a>{/if}
        </div>

        {#if !isMicrosoft && !isLuxmc}
            {#if isEditingNick}
                <div class="flex flex-wrap gap-2"><input aria-label={uiText("ui.b035e0876e9c5cc3")} class="min-w-0 flex-1 rounded-xl border border-border bg-bg/40 px-3 py-2 text-fg" bind:value={editedNick} maxlength="16" onkeydown={e => { if (e.key === 'Enter') handleSaveNick(); }} /><button type="button" class={launcherButton({variant: "primary", size: "sm"})} onclick={handleSaveNick}>{uiText("common.save")}</button><button type="button" class={launcherButton({variant: "secondary", size: "icon"})} aria-label={uiText("common.cancel")} onclick={() => isEditingNick = false}><X class="h-4 w-4" /></button></div>
            {:else}<button type="button" class={launcherButton({variant: "ghost", size: "sm"})} onclick={() => {editedNick = currentUsername; isEditingNick = true;}}><Pencil class="h-4 w-4" />{uiText("ui.40407a7585eaae09")}</button>{/if}
        {/if}

        <p class="rounded-xl border border-border bg-bg/35 px-3 py-2 text-[11px] leading-relaxed text-fg-muted">{isMicrosoft ? uiText("profileCard.microsoftSkins") : isLuxmc ? uiText("profileCard.luxmcSkins") : uiText("profileCard.offlineSkins")}</p>
        <div class="border-t border-border pt-3"><button type="button" class={launcherButton({variant: "danger", size: "sm", class: "w-full"})} onclick={handleLogout}><LogOut class="h-4 w-4" />{uiText("ui.07b09d8a74c4c8f4")}</button></div>
    </div>
</Modal>

<style>
    .profile-modal-cover { background-image: radial-gradient(ellipse at 18% 20%, rgb(var(--brand-400) / .55), transparent 62%), radial-gradient(ellipse at 82% 0%, rgb(var(--info) / .45), transparent 58%), linear-gradient(120deg, rgb(var(--bg-subtle)), rgb(var(--bg-overlay))); }
</style>
