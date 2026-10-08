<script lang="ts">
    import { publicProfile } from '$lib/stores/publicProfile.svelte';
    import { friendsState } from '$lib/stores/friends.svelte';
    import { socialPublicProfile, socialSaveProfile, type PublicProfile, type PackCollection } from '$lib/api/social';
    import { profiles } from '$lib/stores/profiles.svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import { focusTrap } from '$lib/utils/focusTrap';
    import { X, Pencil, Save, Image, LoaderCircle, Star, Upload } from 'lucide-svelte';
    import { goto } from '$app/navigation';
    import MinecraftAvatar from '$lib/components/ui/MinecraftAvatar.svelte';
    import PackCollections from './PackCollections.svelte';
    let collections = $state<PackCollection[]>([]);
    let profile = $state<PublicProfile | null>(null);
    let editing = $state(false);
    let busy = $state(false);
    let error = $state('');
    let description = $state('');
    let displayName = $state('');
    let status = $state('');
    let banner = $state('');
    let portrait = $state('');
    let packs = $state<string[]>([]);
    let favouriteName = $state('');
    let generation = 0;
    $effect(() => {
        const open = publicProfile.open;
        const id = publicProfile.target?.id;
        const own = publicProfile.own;
        const account = friendsState.accountId;
        const run = ++generation;
        if (!open || !account) { profile = null; busy = false; return; }
        profile = null; displayName = ''; status = ''; description = ''; banner = ''; portrait = ''; packs = []; collections=[]; error = ''; busy = true; editing = own;
        void socialPublicProfile(account, id).then(value => {
            if (run !== generation) return;
            profile = value; displayName = value.displayName; status = value.status; description = value.description; banner = value.banner; portrait = value.portrait; packs = value.packs; collections=value.collections;
        }).catch(cause => { if (run === generation) error = String(cause); }).finally(() => { if (run === generation) busy = false; });
    });
    async function image(event: Event, kind: 'banner' | 'portrait') {
        const file = (event.target as HTMLInputElement).files?.[0];
        if (!file) return;
        error = '';
        if (!['image/png','image/jpeg','image/webp','image/gif'].includes(file.type) || file.size > 12 * 1024 * 1024 || (file.type === 'image/gif' && file.size > 700000)) { error = t('publicProfile.imageLimit'); return; }
        const run = generation;
        busy = true;
        try {
        let value = await new Promise<string>((resolve,reject) => { const reader = new FileReader(); reader.onload = () => resolve(String(reader.result)); reader.onerror = reject; reader.readAsDataURL(file); });
        if (file.type !== 'image/gif') {
            try {
                const bitmap = await createImageBitmap(file);
                const limit = kind === 'banner' ? 1280 : 320;
                const ratio = Math.min(1, limit / Math.max(bitmap.width,bitmap.height));
                const canvas = document.createElement('canvas');
                canvas.width = Math.max(1,Math.round(bitmap.width*ratio)); canvas.height = Math.max(1,Math.round(bitmap.height*ratio));
                canvas.getContext('2d')!.drawImage(bitmap,0,0,canvas.width,canvas.height); bitmap.close();
                for (const quality of [.85,.7,.55,.4]) { value = canvas.toDataURL('image/webp',quality); if (value.length < 250000) break; }
                if (value.length >= 250000) { error = t('publicProfile.imageLimit'); return; }
            } catch { error = t('publicProfile.imageLimit'); return; }
        }
        if (run !== generation) return;
        if (kind === 'banner') banner = value; else portrait = value;
        } catch { if (run === generation) error = t('publicProfile.imageLimit'); } finally { if (run === generation) busy = false; }
    }
    async function save() {
        if (busy || !profile) return;
        busy = true; error = '';
        const run = generation;
        const saved = {displayName,status,description,banner,portrait,packs:[...packs],collections:$state.snapshot(collections)};
        try { await socialSaveProfile(friendsState.accountId, saved); if (run !== generation) return; profile = {...profile,...saved}; editing = false; }
        catch(cause) { if (run === generation) error = String(cause); }
        finally { if (run === generation) busy = false; }
    }
</script>
{#if publicProfile.open}
    <div class="fixed inset-0 z-[180] grid place-items-center bg-bg-overlay/70 p-4 backdrop-blur-sm" role="presentation" onclick={event => { if (event.target === event.currentTarget && !busy) publicProfile.close(); }} onkeydown={event => { if (event.key === 'Escape' && !busy) publicProfile.close(); }}>
        <div use:focusTrap role="dialog" aria-modal="true" aria-labelledby="public-profile-title" tabindex="-1" class="public-player-profile flex max-h-[90dvh] w-full max-w-xl flex-col overflow-hidden rounded-3xl border border-border bg-bg-elevated text-fg shadow-elevated">
            <div class="relative h-44 shrink-0 bg-gradient-to-br from-brand-500/25 via-bg-subtle to-bg-elevated">{#if banner}<img src={banner} alt="" class="h-full w-full object-cover" />{/if}<div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-transparent to-transparent"></div><span class="absolute left-6 top-5 rounded-full border border-border bg-bg-elevated/80 px-3 py-1.5 text-[10px] font-semibold uppercase tracking-widest">{t('publicProfile.title')}</span><button type="button" disabled={busy} onclick={() => publicProfile.close()} class={button({variant:'secondary',size:'icon',class:'absolute right-4 top-4'})} aria-label={t('common.close')}><X class="h-4 w-4" /></button></div>
            <div class="min-h-0 flex-1 space-y-5 overflow-y-auto px-6 pb-6">
                <div class="relative flex items-end gap-4"><div class="h-24 w-24 shrink-0 overflow-hidden rounded-3xl border-4 border-bg-elevated bg-bg-subtle shadow-elevated">{#if portrait}<img src={portrait} alt="" class="h-full w-full object-cover" />{:else}<MinecraftAvatar username={profile?.username || publicProfile.target?.username || 'Steve'} avatarUrl={profile?.avatarUrl} class="h-full w-full" />{/if}</div><div class="min-w-0 pb-2"><h2 id="public-profile-title" class="truncate text-2xl font-bold tracking-tight">{displayName || profile?.username || publicProfile.target?.username || friendsState.me?.username}</h2><p class="mt-1 truncate text-xs text-fg-muted">@{profile?.username || publicProfile.target?.username || friendsState.me?.username}</p></div></div>
                {#if error}<p role="alert" class="rounded-xl border border-danger/20 bg-danger/10 p-3 text-sm text-danger">{error}</p>{/if}
                {#if busy && !profile}<p role="status" class="flex gap-2 text-sm text-fg-muted"><LoaderCircle class="h-4 w-4 animate-spin" />{t('common.loading')}</p>{/if}
                {#if profile}
                    {#if editing}
                        <div class="grid gap-3 sm:grid-cols-2">
                            <label class="text-xs font-semibold">{t('publicProfile.displayName')}<input disabled={busy} bind:value={displayName} maxlength="32" placeholder={profile.username} class="mt-2 w-full rounded-xl border border-border bg-bg-subtle p-3 text-sm font-normal" /></label>
                            <label class="text-xs font-semibold">{t('publicProfile.status')}<input disabled={busy} bind:value={status} maxlength="80" class="mt-2 w-full rounded-xl border border-border bg-bg-subtle p-3 text-sm font-normal" /></label>
                        </div>
                        <p class="text-xs leading-relaxed text-fg-muted">{t('publicProfile.visibility')}</p>
                        <label class="block text-xs font-semibold">{t('publicProfile.about')}<textarea disabled={busy} bind:value={description} maxlength="400" rows="3" class="mt-2 w-full resize-y rounded-xl border border-border bg-bg-subtle p-3 text-sm font-normal focus-visible:ring-2 focus-visible:ring-brand-400"></textarea></label>
                        <div class="grid grid-cols-2 gap-3">{#each ['banner','portrait'] as kind}<label class="cursor-pointer rounded-2xl border border-border bg-bg-subtle p-4 text-xs font-semibold transition-colors hover:border-brand-400/50 focus-within:ring-2 focus-within:ring-brand-400"><span class="flex items-center gap-2"><Image class="h-4 w-4" />{t(`publicProfile.${kind}`)}</span><input type="file" accept="image/png,image/jpeg,image/webp,image/gif" disabled={busy} onchange={event => void image(event,kind as 'banner'|'portrait')} class="sr-only" /><span class="mt-3 flex items-center gap-2 text-xs font-normal text-brand-400"><Upload class="h-3.5 w-3.5" />{t('publicProfile.chooseImage')}{#if (kind === 'banner' ? banner : portrait).startsWith('data:image/gif;')}<span class="rounded-md border border-brand-400/30 px-1.5 py-0.5 text-[9px] font-bold">GIF</span>{/if}</span><button type="button" class={button({variant:'ghostDanger',size:'sm',class:'mt-2'})} onclick={() => { if (kind === 'banner') banner=''; else portrait=''; }}>{t('common.remove')}</button></label>{/each}</div>
                        <p class="text-[11px] leading-relaxed text-fg-muted">{t('publicProfile.imageLimit')}</p>
                        <div class="flex items-end gap-2"><label class="min-w-0 flex-1 text-xs font-semibold">{t('publicProfile.addFavourite')}<input disabled={busy} bind:value={favouriteName} maxlength="80" class="mt-2 w-full rounded-xl border border-border bg-bg-subtle px-3 py-2 text-sm font-normal" /></label><button type="button" class={button({variant:'secondary',size:'sm'})} disabled={busy || !favouriteName.trim() || packs.length >= 8} onclick={() => { packs = [...new Set([...packs,favouriteName.trim()])]; favouriteName=''; }}>{t('common.add')}</button></div>
                        <fieldset class="space-y-2"><legend class="mb-2 text-xs font-semibold">{t('publicProfile.packs')}</legend>{#each [...new Set([...profiles.list.map(profile => profile.name), ...packs])] as name}<label class="flex items-center gap-2 text-sm"><input type="checkbox" checked={packs.includes(name)} disabled={busy || (!packs.includes(name) && packs.length >= 8)} onchange={() => packs = packs.includes(name) ? packs.filter(pack => pack !== name) : [...packs,name]} />{name}</label>{/each}</fieldset>
                        <PackCollections bind:value={collections} editing disabled={busy} />
                        <button type="button" disabled={busy} class={button({variant:'primary',block:true})} onclick={() => void save()}>{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Save class="h-4 w-4" />{/if}{t('common.save')}</button>
                    {:else}
                        {#if profile.status}<p class="rounded-2xl border border-brand-400/20 bg-brand-400/10 px-4 py-3 text-sm">{profile.status}</p>{/if}
                        {#if profile.description}<section class="rounded-2xl border border-border bg-bg-subtle/70 p-4"><h3 class="text-xs font-semibold uppercase tracking-wider text-fg-muted">{t('publicProfile.about')}</h3><p class="mt-2 whitespace-pre-wrap break-words text-sm leading-relaxed">{profile.description}</p></section>{/if}
                        {#if profile.packs.length}<section class="rounded-2xl border border-border bg-bg-subtle/70 p-4"><h3 class="text-xs font-semibold uppercase tracking-wider text-fg-muted">{t('publicProfile.packs')}</h3><div class="mt-3 flex flex-wrap gap-2">{#each profile.packs as pack}<button type="button" title={t("workshop.recommend")} class={button({variant:"secondary",size:"sm"})} onclick={() => { publicProfile.close(); void goto(`/mods?type=modpack&search=${encodeURIComponent(pack)}`); }}><Star class="h-3.5 w-3.5 text-brand-400" />{pack}</button>{/each}</div></section>{/if}
                        {#if profile.collections.length}<PackCollections value={profile.collections} />{/if}
                        {#if publicProfile.own}<button type="button" class={button({variant:'secondary',block:true})} onclick={() => editing=true}><Pencil class="h-4 w-4" />{t('publicProfile.edit')}</button>{/if}
                    {/if}
                {/if}
            </div>
        </div>
    </div>
{/if}
