<script lang="ts">
    import { translateUi } from '$lib/i18n/useTranslation.svelte';
    import ProfileBadge from './ProfileBadge.svelte';
    let { role = 'member', tags = [], plan = 'free', size = 'sm' }: { role?: 'owner' | 'member'; tags?: string[]; plan?: 'free'|'plus'|'pro'; size?: 'sm' | 'md' } = $props();
    const visibleTags = $derived([...new Set(tags.map(tag => tag.trim()).filter(Boolean))]);
</script>

{#if role === 'owner' || visibleTags.length || plan !== 'free'}
    <span class="account-badges inline-flex max-w-full min-w-0 flex-wrap items-center gap-1.5 align-middle" aria-label="Tags do perfil">
        {#if role === 'owner'}<ProfileBadge label={translateUi('ownerTools.badge')} kind="owner" {size} />{/if}
        {#if plan !== 'free'}<ProfileBadge label={`Luxmc ${plan === 'pro' ? 'Pro' : 'Plus'}`} kind={plan} {size} order={role === 'owner' ? 1 : 0} />{/if}
        {#each visibleTags as tag, index (tag)}<ProfileBadge label={tag} {size} order={index + Number(role === 'owner') + Number(plan !== 'free')} />{/each}
    </span>
{/if}
