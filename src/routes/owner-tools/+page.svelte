<script lang="ts">
    import { friendsState } from '$lib/stores/friends.svelte';
    import { profiles } from '$lib/stores/profiles.svelte';
    import { appState } from '$lib/stores/app.svelte';
    import { ownerTools } from '$lib/api/owner';
    import { button } from '$lib/components/ui/button';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { ShieldCheck, Activity, Wrench, LoaderCircle } from 'lucide-svelte';
    let selected = $state(profiles.activeId || profiles.list[0]?.id || '');
    let busy = $state(false);
    let error = $state('');
    let result = $state<Record<string, unknown> | null>(null);
    async function run(operation: 'diagnose' | 'repair') {
        if (busy || !selected || friendsState.ownProfile?.role !== 'owner') return;
        busy = true; error = ''; result = null;
        try { result = await ownerTools(friendsState.accountId, selected, operation); }
        catch (cause) { error = String(cause); }
        finally { busy = false; }
    }
</script>
<div class="mx-auto w-full max-w-4xl space-y-6 p-6">
    <header class="flex items-center gap-4"><div class="rounded-2xl border border-brand-400/25 bg-brand-400/10 p-4"><ShieldCheck class="h-7 w-7 text-brand-400" /></div><div><p class="page-eyebrow">{t('ownerTools.badge')}</p><h1 class="page-title">{t('ownerTools.title')}</h1></div></header>
    {#if friendsState.ownProfile?.role !== 'owner'}<p class="rounded-2xl border border-border bg-bg-elevated p-6 text-sm text-fg-muted">{t('ownerTools.restricted')}</p>
    {:else}
        <section class="space-y-5 rounded-3xl border border-border bg-bg-elevated p-6">
            <p class="text-sm leading-relaxed text-fg-muted">{t('ownerTools.description')}</p>
            <label class="block text-xs font-semibold">{t('ownerTools.instance')}<select bind:value={selected} disabled={busy} class="mt-2 w-full rounded-xl border border-border bg-bg-subtle p-3">{#each profiles.list as profile}<option value={profile.id}>{profile.name}</option>{/each}</select></label>
            <div class="flex flex-wrap gap-3"><button type="button" class={button({variant:'primary'})} disabled={busy || !selected} onclick={() => run('diagnose')}><Activity class="h-4 w-4" />{t('ownerTools.diagnose')}</button><button type="button" class={button({variant:'secondary'})} disabled={busy || !selected || appState.isGameRunning} onclick={() => run('repair')}><Wrench class="h-4 w-4" />{t('ownerTools.repair')}</button><a href="/logs-history" class={button({variant:'secondary'})}>{t('ownerTools.history')}</a></div>
            {#if busy}<p role="status" class="flex items-center gap-2 text-sm text-fg-muted"><LoaderCircle class="h-4 w-4 animate-spin" />{t('common.loading')}</p>{/if}
            {#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}
            {#if result}<pre class="max-h-96 overflow-auto whitespace-pre-wrap break-words rounded-2xl border border-border bg-bg-subtle p-4 text-xs leading-relaxed text-fg">{JSON.stringify(result,null,2)}</pre>{/if}
        </section>
    {/if}
</div>
