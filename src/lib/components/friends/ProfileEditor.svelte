<script lang="ts">
    import { onDestroy } from 'svelte';
    import type { PublicProfile, PublicProfileDraft } from '$lib/api/social';
    import { profiles } from '$lib/stores/profiles.svelte';
    import { membershipState } from '$lib/stores/membership.svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import { prepareAnimatedImage, isAnimatedImage } from '$lib/utils/animatedImage';
    import { profileCover } from '$lib/utils/profileImages';
    import AccountBadges from '$lib/components/ui/AccountBadges.svelte';
    import MinecraftAvatar from '$lib/components/ui/MinecraftAvatar.svelte';
    import PackCollections from './PackCollections.svelte';
    import ProfileImageEditor from './ProfileImageEditor.svelte';
    import ProfileStyleEditor from './ProfileStyleEditor.svelte';
    import ProfileSignature from '$lib/components/ui/ProfileSignature.svelte';
    import { profileBenefits } from '$lib/utils/profileStyle';
    import { UserRound, Palette, LayoutGrid, Image, Upload, Eye, Check, Star, X, ArrowUp, ArrowDown, ShieldCheck, LoaderCircle, Sparkles } from 'lucide-svelte';

    let { draft = $bindable(), profile, busy = false, canManageTags = false, processingImage = $bindable(false), pendingImage = $bindable(false), section = $bindable('identity'), onpreview, onmanageTags }: {
        draft: PublicProfileDraft; profile: PublicProfile; busy?: boolean; canManageTags?: boolean; processingImage?: boolean; pendingImage?: boolean; section?: 'identity' | 'appearance' | 'showcase'; onpreview: () => void; onmanageTags: () => void;
    } = $props();
    let favouriteName = $state(''), error = $state('');
    let crop = $state<{ image: ImageBitmap; kind: 'portrait' | 'banner' } | null>(null);
    let generation = 0;
    const locked = $derived(busy || processingImage);
    const plan = $derived(profile.membership ? (profile.membership.expiresAt ?? 0) > Date.now() / 1000 ? profile.membership.plan : 'free' : membershipState.current && (membershipState.current.expiresAt ?? 0) > Date.now() / 1000 ? membershipState.current.plan : 'free');
    const favourites = $derived(profileBenefits[plan].favourites);
    const maximum = $derived(plan === 'pro' ? 24 : plan === 'plus' ? 12 : 6);
    const installed = $derived(profiles.list.filter((profile, index, list) => list.findIndex(item => item.name === profile.name) === index));
    const tabs = $derived([{ id: 'identity', icon: UserRound, label: t('profileStudio.identity') }, { id: 'appearance', icon: Palette, label: t('profileStudio.appearance') }, { id: 'showcase', icon: LayoutGrid, label: t('profileStudio.showcase') }] as const);
    function tabKey(event: KeyboardEvent) {
        const index = tabs.findIndex(tab => tab.id === section);
        const target = event.key === 'ArrowRight' ? (index + 1) % tabs.length : event.key === 'ArrowLeft' ? (index + tabs.length - 1) % tabs.length : event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : -1;
        if (target < 0) return;
        event.preventDefault(); section = tabs[target].id;
        document.getElementById(`profile-tab-${section}`)?.focus();
    }
    function fallbackImage(event: Event) {
        const image = event.currentTarget as HTMLImageElement;
        if (!image.src.endsWith('/grass_block.png')) image.src = '/grass_block.png';
    }
    function closeCrop() { crop?.image.close(); crop = null; pendingImage = false; }
    onDestroy(() => { generation++; closeCrop(); });
    async function upload(event: Event, kind: 'portrait' | 'banner') {
        const input = event.currentTarget as HTMLInputElement, file = input.files?.[0];
        input.value = ''; if (!file || locked) return;
        error = '';
        if (!['image/png','image/jpeg','image/webp','image/gif'].includes(file.type) || file.size > 12 * 1024 * 1024) { error = t('publicProfile.imageLimit'); return; }
        if (file.type === 'image/gif' && plan === 'free') { error = t('publicProfile.gifRequiresPlus'); return; }
        closeCrop(); const run = ++generation; processingImage = true;
        try {
            if (file.type === 'image/gif') {
                const value = await prepareAnimatedImage(file);
                if (run === generation) draft[kind] = value;
            } else {
                let bitmap = await createImageBitmap(file);
                if (run !== generation) { bitmap.close(); return; }
                if (Math.max(bitmap.width, bitmap.height) > 4096) {
                    const original = bitmap, ratio = 4096 / Math.max(original.width, original.height);
                    try { bitmap = await createImageBitmap(original, {resizeWidth:Math.max(1,Math.round(original.width * ratio)),resizeHeight:Math.max(1,Math.round(original.height * ratio)),resizeQuality:'high'}); }
                    finally { original.close(); }
                    if (run !== generation) { bitmap.close(); return; }
                }
                crop = { image: bitmap, kind }; pendingImage = true;
            }
        } catch { if (run === generation) error = t('publicProfile.imageLimit'); }
        finally { if (run === generation) processingImage = false; }
    }
    function preset(value: 'prism' | 'aurora' | 'horizon' | 'constellation') {
        error = '';
        try { closeCrop(); draft.banner = profileCover(value); }
        catch { error = t('publicProfile.imageLimit'); }
    }
    function toggleFavourite(name: string) {
        if (locked) return;
        if (draft.packs.includes(name)) draft.packs = draft.packs.filter(pack => pack !== name);
        else if (draft.packs.length < favourites) draft.packs = [...draft.packs, name];
    }
    function addFavourite() {
        const name = favouriteName.trim();
        if (!name || name.length > 80 || draft.packs.length >= favourites || locked) return;
        if (!draft.packs.includes(name)) draft.packs = [...draft.packs, name];
        favouriteName = '';
    }
    function reorder(index: number, direction: number) {
        const next = [...draft.packs], target = index + direction;
        if (locked || target < 0 || target >= next.length) return;
        [next[index], next[target]] = [next[target], next[index]]; draft.packs = next;
    }
    function bioTemplate(key: string) {
        const template = t(`profileStudio.${key}`);
        const next = draft.description.trim() ? `${draft.description.trim()}\n\n${template}` : template;
        if (next.length <= 400) draft.description = next;
    }
</script>

<div class="profile-editor-grid">
    <div class="min-w-0 space-y-5">
        <div class="editor-tabs" role="tablist" aria-label={t('profileStudio.sections')}>
            {#each tabs as tab}<button type="button" role="tab" id="profile-tab-{tab.id}" aria-controls="profile-section-{tab.id}" aria-selected={section === tab.id} tabindex={section === tab.id ? 0 : -1} disabled={locked} onclick={() => section = tab.id} onkeydown={tabKey}><tab.icon class="h-4 w-4" /><span>{tab.label}</span></button>{/each}
        </div>
        {#if error}<p role="alert" class="rounded-xl border border-danger/25 bg-danger/10 p-3 text-xs text-danger">{error}</p>{/if}
        <div role="tabpanel" tabindex="0" id="profile-section-{section}" aria-labelledby="profile-tab-{section}" class="space-y-5">
            {#if section === 'identity'}
                <div><h3 class="text-lg font-semibold">{t('profileStudio.identityTitle')}</h3><p class="mt-1 text-xs leading-5 text-fg-muted">{t('profileStudio.identityHelp')}</p></div>
                <div class="grid gap-4 sm:grid-cols-2">
                    <label class="editor-field"><span>{t('publicProfile.displayName')}</span><input aria-label={t('publicProfile.displayName')} aria-describedby="profile-name-help" disabled={locked} bind:value={draft.displayName} maxlength="32" placeholder={profile.username} autocomplete="nickname" /><small id="profile-name-help"><span>{t('profileStudio.nameHelp')}</span><span>{draft.displayName.length}/32</span></small></label>
                    <label class="editor-field"><span>{t('publicProfile.status')}</span><input aria-label={t('publicProfile.status')} aria-describedby="profile-status-help" disabled={locked} bind:value={draft.status} maxlength="80" placeholder={t('profileStudio.statusPlaceholder')} /><small id="profile-status-help"><span>{t('profileStudio.statusHelp')}</span><span>{draft.status.length}/80</span></small></label>
                </div>
                <div class="flex flex-wrap gap-2" aria-label={t('profileStudio.statusSuggestions')}>{#each ['readyStatus','buildingStatus','adventureStatus'] as key}<button type="button" disabled={locked} class={button({variant:'ghostBrand',size:'sm'})} onclick={() => draft.status = t(`profileStudio.${key}`)}>{t(`profileStudio.${key}`)}</button>{/each}<button type="button" disabled={locked || !draft.status} class={button({variant:'ghost',size:'sm'})} onclick={() => draft.status = ''}>{t('profileStudio.clearStatus')}</button></div>
                <label class="editor-field"><span>{t('publicProfile.about')}</span><textarea aria-label={t('publicProfile.about')} aria-describedby="profile-bio-help" disabled={locked} bind:value={draft.description} maxlength="400" rows="5" placeholder={t('profileStudio.bioPlaceholder')}></textarea><small id="profile-bio-help"><span>{t('profileStudio.bioHelp')}</span><span>{draft.description.length}/400</span></small></label>
                <div class="flex flex-wrap items-center gap-2"><span class="text-xs text-fg-muted">{t('profileStudio.bioIdeas')}</span>{#each ['playStyle','inviteBio'] as key}<button type="button" disabled={locked || (draft.description.length + t(`profileStudio.${key}Template`).length + (draft.description.trim() ? 2 : 0) > 400)} class={button({variant:'secondary',size:'sm'})} onclick={() => bioTemplate(`${key}Template`)}><Sparkles class="h-3.5 w-3.5" />{t(`profileStudio.${key}`)}</button>{/each}</div>
                <div class="rounded-2xl border border-border bg-bg-subtle/45 p-4"><div class="flex items-center gap-2 text-xs font-semibold"><ShieldCheck class="h-4 w-4 text-brand-400" />{t('profileStudio.tagsTitle')}</div><div class="mt-3"><AccountBadges role={profile.role} tags={profile.tags} plan={profile.membership?.plan} size="md" /></div><p class="mt-3 text-xs leading-5 text-fg-muted">{t('profileStudio.tagsHelp')}</p>{#if canManageTags}<button type="button" disabled={locked} class={button({variant:'secondary',size:'sm',class:'mt-3'})} onclick={onmanageTags}>{t('profileStudio.manageTags')}</button>{/if}</div>
            {:else if section === 'appearance'}
                <div><h3 class="text-lg font-semibold">{t('profileStudio.appearanceTitle')}</h3><p class="mt-1 text-xs leading-5 text-fg-muted">{t('profileStudio.appearanceHelp')}</p></div>
                <ProfileStyleEditor bind:value={draft.profileStyle} {plan} disabled={locked} previewName={draft.displayName || profile.username} previewImage={draft.portrait} previewBanner={draft.banner} />
                {#if crop}<ProfileImageEditor image={crop.image} kind={crop.kind} onapply={value => { if (crop) draft[crop.kind] = value; closeCrop(); }} oncancel={closeCrop} />{/if}
                <div class="grid gap-4 sm:grid-cols-2">
                    {#each ['banner','portrait'] as kind}
                        <div class="rounded-2xl border border-border bg-bg-subtle/35 p-4">
                            <div class="flex items-center justify-between gap-2"><h4 class="text-xs font-semibold">{t(`publicProfile.${kind}`)}</h4>{#if isAnimatedImage(kind === 'banner' ? draft.banner : draft.portrait)}<span class="rounded-md border border-brand-400/25 bg-brand-500/10 px-2 py-0.5 text-[10px] text-brand-400">GIF</span>{/if}</div>
                            <div class="mt-3 grid h-28 place-items-center overflow-hidden rounded-xl border border-border bg-bg-elevated">
                                {#if kind === 'banner' && draft.banner}<img src={draft.banner} alt="" class="h-full w-full object-cover" />{:else if kind === 'portrait'}{#if draft.portrait}<img src={draft.portrait} alt="" class="h-20 w-20 rounded-2xl object-cover" />{:else}<MinecraftAvatar username={profile.username} avatarUrl={profile.avatarUrl} class="h-20 w-20" />{/if}{:else}<Image class="h-7 w-7 text-fg-subtle" />{/if}
                            </div>
                            <label class="mt-3 flex min-h-10 cursor-pointer items-center justify-center gap-2 rounded-xl border border-border bg-bg-elevated px-3 py-2 text-xs font-medium hover:border-brand-400/50 focus-within:ring-2 focus-within:ring-brand-400"><Upload class="h-3.5 w-3.5" />{t('publicProfile.chooseImage')}<input type="file" accept={plan === 'free' ? "image/png,image/jpeg,image/webp" : "image/png,image/jpeg,image/webp,image/gif"} disabled={locked} aria-label={t(kind === 'banner' ? 'profileStudio.selectBanner' : 'profileStudio.selectPortrait')} onchange={event => void upload(event, kind as 'banner' | 'portrait')} class="sr-only" /></label>
                            <p class="mt-2 text-[10px] text-fg-muted">{t('publicProfile.gifRequiresPlus')}</p>
                            <button type="button" disabled={locked || !(kind === 'banner' ? draft.banner : draft.portrait)} class={button({variant:'ghost',size:'sm',block:true,class:'mt-2'})} onclick={() => { closeCrop(); if (kind === 'banner') draft.banner=''; else draft.portrait=''; }}>{t(kind === 'banner' ? 'profileStudio.removeCover' : 'profileStudio.minecraftAvatar')}</button>
                        </div>
                    {/each}
                </div>
                <p class="text-xs leading-5 text-fg-muted">{t('profileStudio.imageHelp')}</p>
                {#if processingImage}<p role="status" class="flex gap-2 text-xs text-fg-muted"><LoaderCircle class="h-4 w-4 animate-spin" />{t('publicProfile.processingImage')}</p>{/if}
                <section class="space-y-3"><h4 class="text-xs font-semibold">{t('profileStudio.coverPresets')}</h4><div class="grid grid-cols-2 gap-3 sm:grid-cols-4">{#each ['prism','aurora','horizon','constellation'] as presetId}<button type="button" disabled={locked} class="cover-tile" aria-label={t('profileStudio.applyCover',{name:t(`profileStudio.${presetId}`)})} onclick={() => preset(presetId as 'prism' | 'aurora' | 'horizon' | 'constellation')}><span class="cover-art cover-art--{presetId}" aria-hidden="true"></span><span>{t(`profileStudio.${presetId}`)}</span></button>{/each}</div><p class="text-xs leading-5 text-fg-muted">{t('profileStudio.coverHelp')}</p></section>
            {:else}
                <div><h3 class="text-lg font-semibold">{t('profileStudio.showcaseTitle')}</h3><p class="mt-1 text-xs leading-5 text-fg-muted">{t('profileStudio.showcaseHelp')}</p></div>
                <p class="text-xs leading-5 text-fg-muted">{t('planProfile.moreFavourites',{count:favourites})}</p>
                <div class="flex items-center justify-between"><h4 class="flex items-center gap-2 text-xs font-semibold"><Star class="h-4 w-4 text-brand-400" />{t('publicProfile.packs')}</h4><span class="text-xs tabular-nums text-fg-muted">{draft.packs.length}/{favourites}</span></div>
                {#if draft.packs.length}<ol class="space-y-2">{#each draft.packs as name, index (name)}<li class="flex items-center gap-2 rounded-xl border border-border bg-bg-subtle/40 p-2.5"><span class="w-5 shrink-0 text-center text-[10px] tabular-nums text-fg-subtle">{index + 1}</span><span class="min-w-0 flex-1 truncate text-xs font-medium" title={name}>{name}</span><button type="button" disabled={locked || index === 0} aria-label={t('profileStudio.moveUp',{name})} class={button({variant:'ghost',size:'icon',class:'h-8 w-8'})} onclick={() => reorder(index,-1)}><ArrowUp class="h-3.5 w-3.5" /></button><button type="button" disabled={locked || index === draft.packs.length - 1} aria-label={t('profileStudio.moveDown',{name})} class={button({variant:'ghost',size:'icon',class:'h-8 w-8'})} onclick={() => reorder(index,1)}><ArrowDown class="h-3.5 w-3.5" /></button><button type="button" disabled={locked} aria-label={`${t('common.remove')}: ${name}`} class={button({variant:'ghostDanger',size:'icon',class:'h-8 w-8'})} onclick={() => toggleFavourite(name)}><X class="h-3.5 w-3.5" /></button></li>{/each}</ol>{:else}<p class="rounded-xl border border-dashed border-border p-4 text-xs leading-5 text-fg-muted">{t('profileStudio.favouritesEmpty')}</p>{/if}
                <form class="flex items-end gap-2" onsubmit={event => {event.preventDefault();addFavourite();}}><label class="editor-field min-w-0 flex-1"><span>{t('publicProfile.addFavourite')}</span><input disabled={locked || draft.packs.length >= favourites} bind:value={favouriteName} maxlength="80" placeholder={t('profileStudio.favouritePlaceholder')} /></label><button type="submit" disabled={locked || !favouriteName.trim() || draft.packs.length >= favourites || draft.packs.includes(favouriteName.trim())} class={button({variant:'secondary'})}>{t('common.add')}</button></form>
                {#if installed.length}<section class="space-y-3"><h4 class="text-xs font-semibold">{t('profileStudio.fromLibrary')}</h4><div class="grid gap-2 sm:grid-cols-2">{#each installed as instance (instance.id)}<button type="button" class="library-pick" disabled={locked || (!draft.packs.includes(instance.name) && draft.packs.length >= favourites)} aria-pressed={draft.packs.includes(instance.name)} onclick={() => toggleFavourite(instance.name)}><img src={instance.icon || '/grass_block.png'} alt="" class="h-8 w-8 shrink-0 rounded-lg object-cover" onerror={fallbackImage} /><span class="min-w-0 flex-1 text-left"><span class="block truncate text-xs font-medium">{instance.name}</span><span class="mt-1 block text-[10px] text-fg-muted">{instance.mcVersion}</span></span>{#if draft.packs.includes(instance.name)}<Check class="h-4 w-4 shrink-0 text-brand-400" />{/if}</button>{/each}</div></section>{/if}
                <PackCollections bind:value={draft.collections} editing disabled={locked} {maximum} />
            {/if}
        </div>
    </div>
    <aside class="profile-live-preview" aria-label={t('profileStudio.livePreview')}>
        <div class="mb-3 flex items-center gap-2 text-[10px] font-semibold uppercase tracking-widest text-fg-muted"><Eye class="h-3.5 w-3.5" />{t('profileStudio.livePreview')}</div>
        <ProfileSignature signature={draft.profileStyle}><article class="overflow-hidden rounded-2xl border border-border bg-bg-subtle/35">
            <div data-profile-banner class="relative h-28 overflow-hidden bg-gradient-to-br from-brand-500/25 via-bg-subtle to-bg-elevated">{#if draft.banner}<img src={draft.banner} alt="" class="h-full w-full object-cover" />{/if}<div class="absolute inset-0 bg-gradient-to-t from-bg-elevated to-transparent"></div></div>
            <div data-profile-preview class="relative space-y-3 px-4 pb-5"><div data-profile-avatar class="-mt-8 h-16 w-16 rounded-2xl border-4 border-bg-elevated bg-bg-subtle">{#if draft.portrait}<img src={draft.portrait} alt="" class="h-full w-full rounded-xl object-cover" />{:else}<MinecraftAvatar username={profile.username} avatarUrl={profile.avatarUrl} class="h-full w-full" />{/if}</div><div><h4 data-profile-name class="break-words text-base font-semibold">{draft.displayName.trim() || profile.username}</h4><p class="mt-1 truncate text-[11px] text-fg-muted">@{profile.username}</p></div><AccountBadges role={profile.role} tags={profile.tags} plan={profile.membership?.plan} />{#if draft.status.trim()}<p data-profile-status class="rounded-lg border border-brand-400/20 bg-brand-500/10 p-2.5 text-xs leading-5">{draft.status}</p>{/if}{#if draft.description.trim()}<p class="line-clamp-6 whitespace-pre-wrap break-words text-xs leading-5 text-fg-muted">{draft.description}</p>{/if}{#if draft.packs.length}<div class="border-t border-border pt-3"><p class="mb-2 text-[10px] font-semibold uppercase tracking-wide text-fg-muted">{t('publicProfile.packs')}</p><div class="flex flex-wrap gap-1.5">{#each draft.packs.slice(0,3) as name}<span class="max-w-full truncate rounded-lg border border-border bg-bg-elevated px-2 py-1 text-[10px]">{name}</span>{/each}{#if draft.packs.length > 3}<span class="text-[10px] text-fg-muted">+{draft.packs.length - 3}</span>{/if}</div></div>{/if}{#if draft.collections.length}<p class="text-[11px] text-fg-muted">{t('profileStudio.collectionCount',{count:draft.collections.length})}</p>{/if}</div>
        </article></ProfileSignature>
        <p class="my-3 text-xs leading-5 text-fg-muted">{t('profileStudio.previewHelp')}</p>
        <button type="button" class={button({variant:'secondary',size:'sm',block:true})} disabled={locked || pendingImage} onclick={onpreview}><Eye class="h-3.5 w-3.5" />{t('profileStudio.fullPreview')}</button>
    </aside>
</div>

<style>
    .profile-editor-grid { display: grid; grid-template-columns: minmax(0, 1fr) 280px; gap: 28px; align-items: start; }
    .editor-tabs { display: flex; gap: 4px; padding: 5px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-subtle) / .45); }
    .editor-tabs button { display: flex; flex: 1; align-items: center; justify-content: center; gap: 7px; min-width: 0; padding: 11px 8px; border: 1px solid transparent; border-radius: 10px; font-size: 12px; color: rgb(var(--fg-muted)); transition: background 160ms ease, color 160ms ease; }
    .editor-tabs button[aria-selected='true'] { border-color: rgb(var(--brand-400) / .25); color: rgb(var(--fg)); background: rgb(var(--brand-500) / .1); box-shadow: inset 0 1px 0 rgb(var(--surface-shine) / .07); }
    .editor-field { display: block; font-size: 12px; font-weight: 550; }
    .editor-field input, .editor-field textarea { display: block; width: 100%; margin-top: 8px; padding: 12px; border: 1px solid rgb(var(--border)); border-radius: 12px; background: rgb(var(--bg-subtle) / .45); color: rgb(var(--fg)); font-size: 13px; font-weight: 400; }
    .editor-field textarea { resize: vertical; min-height: 112px; }
    .editor-field small { display: flex; justify-content: space-between; gap: 8px; margin-top: 7px; font-size: 10px; line-height: 1.5; font-weight: 400; color: rgb(var(--fg-muted)); }
    .editor-field small span:last-child { flex-shrink: 0; font-variant-numeric: tabular-nums; }
    .profile-live-preview { position: sticky; top: 0; min-width: 0; }
    .cover-tile { overflow: hidden; border: 1px solid rgb(var(--border)); border-radius: 12px; background: rgb(var(--bg-subtle) / .45); transition: border-color 160ms ease, translate 180ms ease; }
    .cover-tile > span:last-child { display: block; padding: 9px 5px; font-size: 11px; }
    .cover-tile:hover:not(:disabled) { translate: 0 -2px; border-color: rgb(var(--brand-400) / .5); }
    .cover-art { display: block; height: 60px; background-color: rgb(var(--bg-overlay)); }
    .cover-art--prism { background-image: repeating-linear-gradient(40deg, rgb(var(--brand-400) / .3) 0 18px, rgb(var(--info) / .12) 18px 42px); }
    .cover-art--aurora { background-image: radial-gradient(ellipse at 20% 15%, rgb(var(--brand-400) / .6), transparent 70%), radial-gradient(ellipse at 85% 85%, rgb(var(--success) / .5), transparent 70%); }
    .cover-art--horizon { background-image: linear-gradient(180deg, rgb(var(--info) / .6), transparent 70%), linear-gradient(150deg, transparent 50%, rgb(var(--bg-subtle)) 50%); }
    .cover-art--constellation { background-image: radial-gradient(circle, rgb(var(--brand-300) / .8) 1px, transparent 2px); background-size: 23px 17px; }
    .library-pick { display: flex; align-items: center; gap: 10px; min-width: 0; padding: 10px; border: 1px solid rgb(var(--border)); border-radius: 12px; background: rgb(var(--bg-subtle) / .35); }
    .library-pick[aria-pressed='true'] { border-color: rgb(var(--brand-400) / .35); background: rgb(var(--brand-500) / .08); }
    @media (max-width: 850px) { .profile-editor-grid { grid-template-columns: minmax(0, 1fr); } .profile-live-preview { position: static; } }
    @media (max-width: 500px) { .editor-tabs button { flex-direction: column; gap: 5px; font-size: 11px; } }
    :global(html.no-ui-motion) .cover-tile, :global(html.no-anim) .cover-tile { transition: none; }
    :global(html.no-ui-motion) .cover-tile:hover, :global(html.no-anim) .cover-tile:hover { translate: none; }
    @media (prefers-reduced-motion: reduce) { :global(html:not(.force-ui-motion)) .cover-tile { transition: none; translate: none; } }
</style>
