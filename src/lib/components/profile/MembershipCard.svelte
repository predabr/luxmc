<script lang="ts">
    import { translateUi as uiText, currentUiLocale } from '$lib/i18n/useTranslation.svelte';
    import { account } from '$lib/stores/account.svelte';
    import { membershipState } from '$lib/stores/membership.svelte';
    import { friendsState } from '$lib/stores/friends.svelte';
    import { profileBenefits } from '$lib/utils/profileStyle';
    import { openMembershipPortal } from '$lib/api/membership';
    import { button } from '$lib/components/ui/button';
    import { toast } from '$lib/stores/toasts.svelte';
    import { Sparkles, ExternalLink, Plus, Trash2 } from 'lucide-svelte';
    let opening = $state(false);
    let name = $state('');
    const plan = $derived(membershipState.ready ? membershipState.current?.plan : undefined);
    const benefits = $derived(plan ? profileBenefits[plan] : undefined);
    const canManage = $derived(Boolean(account.value && membershipState.ownerFor(account.value.id) && friendsState.accountId === account.value.id && friendsState.me?.id === membershipState.socialId && friendsState.me?.role === 'owner'));
    async function run(action: () => Promise<unknown>) { try { await action(); } catch (cause) { toast(String(cause), 'error'); } }
    async function portal(owner = false) { if (!account.value || opening || owner && !canManage) return; opening = true; try { await run(() => openMembershipPortal(account.value!.id, owner)); } finally { opening = false; } }
</script>
<section class="space-y-3 rounded-2xl border border-brand-400/25 bg-brand-500/5 p-4" aria-busy={!plan && membershipState.busy}>
    <div class="flex items-center gap-2"><Sparkles class="h-4 w-4 text-brand-400" /><h3 class="font-semibold">{plan ? `Luxmc ${plan === 'pro' ? 'Pro' : plan === 'plus' ? 'Plus' : 'Free'}` : uiText(membershipState.error ? 'refinement.planUnavailable' : 'refinement.planLoading')}</h3></div>
    {#if plan && benefits}
    {#if membershipState.paid && membershipState.current?.expiresAt}<p class="text-xs text-fg-muted">{uiText('membershipPanel.activeUntil', {date: new Date(membershipState.current.expiresAt * 1000).toLocaleDateString(currentUiLocale()), collections: membershipState.current.collections, presets: membershipState.current.presets})}</p>{:else}<p class="text-xs text-fg-muted">{uiText('membershipPanel.free')}</p>{/if}
    <div class="grid grid-cols-2 gap-2 text-xs"><div class="rounded-xl border border-border bg-bg-elevated/50 p-3"><strong class="block text-base text-brand-400">{benefits.favourites}</strong><span class="text-fg-muted">{uiText('membershipPanel.favourites')}</span></div><div class="rounded-xl border border-border bg-bg-elevated/50 p-3"><strong class="block text-base text-brand-400">{benefits.accents.length}</strong><span class="text-fg-muted">{uiText('membershipPanel.palettes')}</span></div></div>
    <p class="text-xs leading-5 text-fg-muted">{plan === 'pro' ? uiText('proCards.benefits') : plan === 'plus' ? uiText('membershipPanel.plus') : uiText('refinement.freePlan')}</p>
    {:else}
        <p class="text-xs leading-5 text-fg-muted" role="status">{uiText(membershipState.error ? 'refinement.planRetryHint' : 'refinement.planLoadingHint')}</p>
    {/if}
    <button type="button" class={button({variant:'secondary',block:true})} disabled={opening} onclick={() => portal()}><ExternalLink class="h-4 w-4" />{uiText(opening ? 'membershipPanel.opening' : 'membershipPanel.portal')}</button>
    {#if canManage}<button type="button" class={button({variant:'primary',block:true})} disabled={opening} onclick={() => portal(true)}>{uiText('membershipPanel.owner')}</button>{/if}
    {#if membershipState.error}<p class="text-xs text-danger" role="status">{membershipState.error}</p><button type="button" class={button({variant:'ghost',size:'sm'})} onclick={() => membershipState.refresh()}>{uiText('membershipPanel.refresh')}</button>{/if}
    {#if membershipState.paid}
        <p class="text-xs text-fg-muted">{uiText('membershipPanel.synced')} · {membershipState.presets.length}/{membershipState.current?.presets}</p>
        <div class="flex gap-2"><input aria-label={uiText('membershipPanel.presetName')} bind:value={name} maxlength="40" placeholder={uiText('membershipPanel.presetPlaceholder')} class="min-w-0 flex-1 rounded-xl border border-border bg-bg-elevated px-3 py-2 text-xs" /><button type="button" aria-label={uiText('membershipPanel.savePreset')} class={button({variant:'secondary',size:'icon'})} disabled={membershipState.busy || !name.trim() || membershipState.presets.length >= (membershipState.current?.presets || 0)} onclick={() => run(async () => { await membershipState.savePreset(name); name = ''; })}><Plus class="h-4 w-4" /></button></div>
        {#each membershipState.presets as preset (preset.id)}<div class="flex items-center gap-2 rounded-xl border border-border p-2"><button type="button" class="min-w-0 flex-1 truncate px-2 text-left text-xs hover:text-brand-400" disabled={membershipState.busy} onclick={() => run(() => membershipState.applyPreset(preset.id))}>{preset.name}</button><button type="button" class={button({variant:'ghostDanger',size:'icon'})} aria-label={uiText('membershipPanel.delete', {name:preset.name})} disabled={membershipState.busy} onclick={() => run(() => membershipState.deletePreset(preset.id))}><Trash2 class="h-3.5 w-3.5" /></button></div>{/each}
    {/if}
</section>
