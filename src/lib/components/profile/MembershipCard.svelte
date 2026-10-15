<script lang="ts">
    import { translateUi as uiText } from '$lib/i18n/useTranslation.svelte';
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
    {#if membershipState.paid && membershipState.current?.expiresAt}<p class="text-xs text-fg-muted">Ativo até {new Date(membershipState.current.expiresAt * 1000).toLocaleDateString()} · {membershipState.current.collections} coleções · {membershipState.current.presets} perfis de aparência</p>{:else}<p class="text-xs text-fg-muted">Seu launcher completo. Amplie seu perfil com Plus ou Pro.</p>{/if}
    <div class="grid grid-cols-2 gap-2 text-xs"><div class="rounded-xl border border-border bg-bg-elevated/50 p-3"><strong class="block text-base text-brand-400">{benefits.favourites}</strong><span class="text-fg-muted">favoritos no perfil</span></div><div class="rounded-xl border border-border bg-bg-elevated/50 p-3"><strong class="block text-base text-brand-400">{benefits.accents.length}</strong><span class="text-fg-muted">paletas do perfil</span></div></div>
    <p class="text-xs leading-5 text-fg-muted">{plan === 'pro' ? 'Sua assinatura completa: Halo, Contorno duplo, Órbita, Brilho da capa, Pulso do avatar e composição centralizada.' : plan === 'plus' ? 'Seis paletas para o perfil, molduras Halo e Contorno duplo. Seu estilo aparece para seus amigos.' : uiText('refinement.freePlan')}</p>
    {:else}
        <p class="text-xs leading-5 text-fg-muted" role="status">{uiText(membershipState.error ? 'refinement.planRetryHint' : 'refinement.planLoadingHint')}</p>
    {/if}
    <button type="button" class={button({variant:'secondary',block:true})} disabled={opening} onclick={() => portal()}><ExternalLink class="h-4 w-4" />{opening ? 'Abrindo…' : 'Meu plano e pedidos'}</button>
    {#if canManage}<button type="button" class={button({variant:'primary',block:true})} disabled={opening} onclick={() => portal(true)}>Central do dono · aprovar planos</button>{/if}
    {#if membershipState.error}<p class="text-xs text-danger" role="status">{membershipState.error}</p><button type="button" class={button({variant:'ghost',size:'sm'})} onclick={() => membershipState.refresh()}>Atualizar meu plano</button>{/if}
    {#if membershipState.paid}
        <p class="text-xs text-fg-muted">Perfis sincronizados de aparência · {membershipState.presets.length}/{membershipState.current?.presets}</p>
        <div class="flex gap-2"><input aria-label="Nome do perfil de aparência" bind:value={name} maxlength="40" placeholder="Meu perfil de aparência" class="min-w-0 flex-1 rounded-xl border border-border bg-bg-elevated px-3 py-2 text-xs" /><button type="button" aria-label="Salvar perfil de aparência" class={button({variant:'secondary',size:'icon'})} disabled={membershipState.busy || !name.trim() || membershipState.presets.length >= (membershipState.current?.presets || 0)} onclick={() => run(async () => { await membershipState.savePreset(name); name = ''; })}><Plus class="h-4 w-4" /></button></div>
        {#each membershipState.presets as preset (preset.id)}<div class="flex items-center gap-2 rounded-xl border border-border p-2"><button type="button" class="min-w-0 flex-1 truncate px-2 text-left text-xs hover:text-brand-400" disabled={membershipState.busy} onclick={() => run(() => membershipState.applyPreset(preset.id))}>{preset.name}</button><button type="button" class={button({variant:'ghostDanger',size:'icon'})} aria-label={`Excluir ${preset.name}`} disabled={membershipState.busy} onclick={() => run(() => membershipState.deletePreset(preset.id))}><Trash2 class="h-3.5 w-3.5" /></button></div>{/each}
    {/if}
</section>
