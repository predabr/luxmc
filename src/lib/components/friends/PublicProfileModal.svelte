<script lang="ts">
    import AccountBadges from '$lib/components/ui/AccountBadges.svelte';
    import MinecraftAvatar from '$lib/components/ui/MinecraftAvatar.svelte';
    import { publicProfile } from '$lib/stores/publicProfile.svelte';
    import { friendsState } from '$lib/stores/friends.svelte';
    import { socialPublicProfile, socialSaveProfile, type PublicProfile, type PublicProfileDraft } from '$lib/api/social';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import { focusTrap } from '$lib/utils/focusTrap';
    import { X, Pencil, Save, LoaderCircle, Star, Eye, Check, Undo2, AlertTriangle } from 'lucide-svelte';
    import { goto } from '$app/navigation';
    import PackCollections from './PackCollections.svelte';
    import ProfileEditor from './ProfileEditor.svelte';
    import ProfileSignature from '$lib/components/ui/ProfileSignature.svelte';
    import { defaultProfileStyle } from '$lib/utils/profileStyle';

    const emptyDraft = (): PublicProfileDraft => ({ displayName:'', status:'', description:'', banner:'', portrait:'', packs:[], collections:[], profileStyle:{...defaultProfileStyle} });
    function draftOf(value: PublicProfile): PublicProfileDraft {
        return { profileStyle:{...value.profileStyle}, displayName:value.displayName, status:value.status, description:value.description, banner:value.banner, portrait:value.portrait, packs:[...value.packs], collections:value.collections.map(collection => ({...collection, entries:collection.entries.map(entry => ({...entry}))})) };
    }
    let profile = $state<PublicProfile | null>(null);
    let draft = $state<PublicProfileDraft>(emptyDraft());
    let editing = $state(false), preview = $state(false), busy = $state(false), processingImage = $state(false), pendingImage = $state(false);
    let section = $state<'identity' | 'appearance' | 'showcase'>('identity');
    let error = $state(''), saved = $state(false), confirmClose = $state(false), destination = $state('');
    let generation = 0;
    const dirty = $derived(Boolean(profile && (draft.displayName.trim() !== profile.displayName || draft.status.trim() !== profile.status || draft.description.trim() !== profile.description || draft.banner !== profile.banner || draft.portrait !== profile.portrait || JSON.stringify(draft.packs) !== JSON.stringify(profile.packs) || JSON.stringify(draft.collections) !== JSON.stringify(profile.collections) || JSON.stringify(draft.profileStyle) !== JSON.stringify(profile.profileStyle))));
    const fullEditor = $derived(editing && !preview);
    const locked = $derived(busy || processingImage);
    $effect(() => {
        const open = publicProfile.open, id = publicProfile.target?.id, own = publicProfile.own, account = friendsState.accountId;
        const run = ++generation;
        confirmClose = false; destination = ''; processingImage = false; pendingImage = false; saved = false;
        if (!open || !account) { profile = null; draft = emptyDraft(); busy = false; return; }
        preview = false; profile = null; draft = emptyDraft(); error = ''; busy = true; editing = own; section = 'identity';
        void socialPublicProfile(account, id).then(value => {
            if (run !== generation) return;
            profile = value; draft = draftOf(value);
        }).catch(cause => { if (run === generation) error = String(cause); }).finally(() => { if (run === generation) busy = false; });
    });
    function close() {
        const next = destination;
        publicProfile.close();
        if (next) void goto(next);
    }
    function requestClose(next = '') {
        if (busy && profile) return;
        destination = next;
        if (editing && (dirty || pendingImage || processingImage)) { confirmClose = true; return; }
        close();
    }
    function discard() {
        if (!profile || locked) return;
        draft = draftOf(profile); preview = false; editing = false; pendingImage = false; error = ''; saved = false;
    }
    async function save() {
        if (locked || pendingImage || !profile || !dirty) return;
        busy = true; error = '';
        const run = generation;
        const value = $state.snapshot(draft);
        value.displayName = value.displayName.trim(); value.status = value.status.trim(); value.description = value.description.trim();
        try {
            await socialSaveProfile(friendsState.accountId, value);
            if (run !== generation) return;
            profile = {...profile,...value}; draft = draftOf(profile); editing = false; preview = false; saved = true;
            await Promise.allSettled([friendsState.refreshProfile(), friendsState.refresh()]);
        } catch (cause) { if (run === generation) error = String(cause); }
        finally { if (run === generation) busy = false; }
    }
    function keyboard(event: KeyboardEvent) {
        if (!publicProfile.open || !editing || confirmClose) return;
        if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') { event.preventDefault(); void save(); }
    }
</script>

<svelte:window onkeydown={keyboard} />
{#if publicProfile.open}
    <div class="fixed inset-0 z-[180] grid place-items-center bg-bg-overlay/75 p-2 backdrop-blur-sm sm:p-4" role="presentation" onclick={event => {if (event.target === event.currentTarget) requestClose();}} onkeydown={event => {if (event.key === 'Escape') {event.stopPropagation();if (confirmClose) confirmClose=false;else requestClose();}}}>
        <ProfileSignature signature={draft.profileStyle} class="contents"><div use:focusTrap role="dialog" aria-modal="true" aria-labelledby="public-profile-title" tabindex="-1" class="profile-dialog public-player-profile relative flex max-h-[92dvh] w-full flex-col overflow-hidden rounded-3xl border border-border bg-bg-elevated text-fg shadow-elevated {fullEditor ? 'max-w-5xl' : 'max-w-2xl'}">
            {#if fullEditor}
                <header class="flex shrink-0 items-start justify-between gap-4 border-b border-border px-6 py-5 sm:px-7">
                    <div><p class="text-[10px] font-semibold uppercase tracking-[.2em] text-brand-400">Luxmc / {t('profileStudio.community')}</p><h2 id="public-profile-title" class="mt-2 text-2xl font-semibold tracking-tight">{t('profileStudio.title')}</h2><p class="mt-1 text-xs leading-5 text-fg-muted">{t('profileStudio.subtitle')}</p></div>
                    <button type="button" disabled={busy && !!profile} onclick={() => requestClose()} class={button({variant:'secondary',size:'icon'})} aria-label={t('common.close')}><X class="h-4 w-4" /></button>
                </header>
            {:else}
                <div data-profile-banner class="relative h-44 shrink-0 overflow-hidden bg-gradient-to-br from-brand-500/25 via-bg-subtle to-bg-elevated">{#if draft.banner}<img src={draft.banner} alt="" class="h-full w-full object-cover" />{/if}<div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-transparent to-transparent"></div><span class="absolute left-6 top-5 rounded-full border border-border bg-bg-elevated/80 px-3 py-1.5 text-[10px] font-semibold uppercase tracking-widest">{preview ? t('profileStudio.fullPreview') : t('publicProfile.title')}</span><button type="button" disabled={busy && !!profile} onclick={() => requestClose()} class={button({variant:'secondary',size:'icon',class:'absolute right-4 top-4'})} aria-label={t('common.close')}><X class="h-4 w-4" /></button></div>
            {/if}
            <div class="min-h-0 flex-1 overflow-y-auto px-6 {fullEditor ? 'py-6 sm:px-7' : 'space-y-5 pb-6'}">
                {#if error}<p role="alert" class="mb-4 rounded-xl border border-danger/20 bg-danger/10 p-3 text-sm text-danger">{error}</p>{/if}
                {#if busy && !profile}<p role="status" class="flex min-h-56 items-center justify-center gap-2 text-sm text-fg-muted"><LoaderCircle class="h-4 w-4 animate-spin" />{t('common.loading')}</p>{/if}
                {#if profile}
                    {#if fullEditor}
                        <ProfileEditor bind:draft {profile} {busy} bind:processingImage bind:pendingImage bind:section onpreview={() => preview=true} onmanageTags={() => requestClose('/owner')} />
                    {:else}
                        <div data-profile-heading class="relative flex items-end gap-4"><div data-profile-avatar class="h-24 w-24 shrink-0 rounded-3xl border-4 border-bg-elevated bg-bg-subtle shadow-elevated">{#if draft.portrait}<img src={draft.portrait} alt="" class="h-full w-full rounded-2xl object-cover" />{:else}<MinecraftAvatar username={profile.username} avatarUrl={profile.avatarUrl} class="h-full w-full" />{/if}</div><div class="min-w-0 pb-2"><h2 id="public-profile-title" data-profile-name class="break-words text-2xl font-bold tracking-tight">{draft.displayName || profile.username}</h2><p class="mt-1 truncate text-xs text-fg-muted">@{profile.username}</p></div></div>
                        <AccountBadges role={profile.role} plan={profile.membership?.plan} tags={profile.tags} size="md" />
                        {#if saved}<p role="status" class="flex items-start gap-2 rounded-xl border border-success/25 bg-success/10 p-3 text-xs leading-5"><Check class="mt-0.5 h-4 w-4 shrink-0 text-success" />{t('profileStudio.saved')}</p>{/if}
                        {#if draft.status}<p data-profile-status class="rounded-2xl border border-brand-400/20 bg-brand-400/10 px-4 py-3 text-sm">{draft.status}</p>{/if}
                        {#if draft.description}<section class="rounded-2xl border border-border bg-bg-subtle/70 p-4"><h3 class="text-xs font-semibold uppercase tracking-wider text-fg-muted">{t('publicProfile.about')}</h3><p class="mt-2 whitespace-pre-wrap break-words text-sm leading-relaxed">{draft.description}</p></section>{/if}
                        {#if draft.packs.length}<section class="rounded-2xl border border-border bg-bg-subtle/70 p-4"><h3 class="text-xs font-semibold uppercase tracking-wider text-fg-muted">{t('publicProfile.packs')}</h3><div class="mt-3 flex flex-wrap gap-2">{#each draft.packs as pack}<button type="button" title={t('workshop.recommend')} class={button({variant:'secondary',size:'sm'})} onclick={() => {if (!editing) {publicProfile.close();void goto(`/mods?type=modpack&search=${encodeURIComponent(pack)}`);}}} disabled={editing}><Star class="h-3.5 w-3.5 text-brand-400" />{pack}</button>{/each}</div></section>{/if}
                        {#if draft.collections.length}<PackCollections value={draft.collections} disabled={editing} />{/if}
                        {#if publicProfile.own && !editing}<button type="button" class={button({variant:'secondary',block:true})} disabled={busy} onclick={() => {editing=true;preview=false;saved=false;section='identity';}}><Pencil class="h-4 w-4" />{t('publicProfile.edit')}</button>{/if}
                    {/if}
                {/if}
            </div>
            {#if editing && profile}
                <footer class="flex shrink-0 flex-wrap items-center justify-between gap-3 border-t border-border bg-bg-elevated px-6 py-4 sm:px-7">
                    <div class="min-w-0 text-xs text-fg-muted"><p class="flex items-center gap-2"><span class="h-1.5 w-1.5 rounded-full {dirty || pendingImage ? 'bg-warning' : 'bg-success'}"></span>{pendingImage ? t('profileStudio.pendingCrop') : dirty ? t('profileStudio.unsaved') : t('profileStudio.upToDate')}</p><p class="mt-1 hidden text-[10px] text-fg-subtle sm:block">{t('profileStudio.shortcut')}</p></div>
                    <div class="flex flex-wrap gap-2"><button type="button" class={button({variant:'ghost',size:'sm'})} disabled={locked || (!dirty && !pendingImage)} onclick={discard}><Undo2 class="h-3.5 w-3.5" />{t('profileStudio.discard')}</button><button type="button" disabled={locked || pendingImage} class={button({variant:'secondary',size:'sm'})} onclick={() => preview = !preview}><Eye class="h-3.5 w-3.5" />{preview ? t('profileStudio.continueEditing') : t('profileStudio.preview')}</button><button type="button" disabled={locked || pendingImage || !dirty} class={button({variant:'primary',size:'sm'})} onclick={() => void save()}>{#if busy}<LoaderCircle class="h-3.5 w-3.5 animate-spin" />{:else}<Save class="h-3.5 w-3.5" />{/if}{t('profileStudio.save')}</button></div>
                </footer>
            {/if}
            {#if confirmClose}
                <div class="absolute inset-0 z-20 grid place-items-center rounded-3xl bg-bg-overlay/80 p-5 backdrop-blur-sm">
                    <div use:focusTrap role="alertdialog" aria-modal="true" aria-labelledby="profile-discard-title" aria-describedby="profile-discard-description" tabindex="-1" class="w-full max-w-sm rounded-2xl border border-border bg-bg-elevated p-6 shadow-elevated"><AlertTriangle class="h-7 w-7 text-warning" /><h3 id="profile-discard-title" class="mt-4 text-lg font-semibold">{t('profileStudio.closeTitle')}</h3><p id="profile-discard-description" class="mt-2 text-sm leading-6 text-fg-muted">{t('profileStudio.closeHelp')}</p><div class="mt-5 flex flex-wrap gap-2"><button type="button" class={button({variant:'secondary',size:'sm'})} onclick={() => confirmClose=false}>{t('profileStudio.continueEditing')}</button><button type="button" class={button({variant:'ghostDanger',size:'sm'})} onclick={close}>{t('profileStudio.discardClose')}</button></div></div>
                </div>
            {/if}
        </div></ProfileSignature>
    </div>
{/if}

<style>
    .profile-dialog { animation: profile-enter 280ms cubic-bezier(.16,1,.3,1); }
    @keyframes profile-enter { from { opacity: 0; translate: 0 14px; scale: .985; } to { opacity: 1; translate: 0 0; scale: 1; } }
    :global(html.no-ui-motion) .profile-dialog, :global(html.no-anim) .profile-dialog { animation: none; }
    @media (prefers-reduced-motion: reduce) { :global(html:not(.force-ui-motion)) .profile-dialog { animation: none; } }
</style>
