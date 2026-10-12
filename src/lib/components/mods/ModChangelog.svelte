<script lang="ts">
    import type { ModVersion } from '$lib/api/types';
    import { modsVersionHistory } from '$lib/api/mods';
    import { versionChangelog } from '$lib/utils/changelogHistory';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import { LoaderCircle, History, RefreshCw } from 'lucide-svelte';
    import { untrack } from 'svelte';

    let { projectId, source, versions, renderBody }: { projectId: string; source: string; versions: ModVersion[]; renderBody: (body: string) => string } = $props();
    let history = $state<ModVersion[]>([]);
    let entries = $state<Record<string, { body: string; error: string; loading: boolean }>>({});
    let visibleCount = $state(6);
    let selected = $state('');
    let nextOffset = $state<number | null>(null);
    let loadingPage = $state(false);
    let pageError = $state('');
    let generation = 0;
    const shown = $derived(selected ? history.filter(version => version.id === selected) : history.slice(0, visibleCount));
    const hasMore = $derived(!selected && (history.length > visibleCount || nextOffset !== null));

    $effect(() => {
        const id = projectId;
        const provider = source;
        const initial = versions;
        generation += 1;
        history = [...new Map(initial.map(version => [version.id, version])).values()];
        entries = {};
        visibleCount = 6;
        selected = '';
        pageError = '';
        loadingPage = false;
        nextOffset = provider === 'curseforge' ? 0 : null;
        if (provider === 'curseforge') untrack(() => { void loadPage(id, provider, generation, 0); });
    });

    $effect(() => {
        const batch = shown;
        const id = projectId;
        const provider = source;
        const token = generation;
        untrack(() => { for (const version of batch) if (!entries[version.id]) void loadEntry(version.id, id, provider, token); });
    });

    async function loadEntry(versionId: string, id = projectId, provider = source, token = generation) {
        entries[versionId] = { body: '', error: '', loading: true };
        try {
            const body = await versionChangelog(id, provider, versionId);
            if (generation === token) entries[versionId] = { body, error: '', loading: false };
        } catch (error) {
            if (generation === token) entries[versionId] = { body: '', error: String(error), loading: false };
        }
    }

    async function loadPage(id = projectId, provider = source, token = generation, offset = nextOffset) {
        if (loadingPage || offset === null) return;
        loadingPage = true;
        pageError = '';
        try {
            const page = await modsVersionHistory(id, provider, offset);
            if (generation !== token) return;
            const merged = offset === 0 ? [...page.versions, ...history] : [...history, ...page.versions];
            history = [...new Map(merged.map(version => [version.id, version])).values()];
            nextOffset = page.nextOffset !== null && page.nextOffset > offset ? page.nextOffset : null;
        } catch (error) {
            if (generation === token) pageError = String(error);
        } finally {
            if (generation === token) loadingPage = false;
        }
    }

    async function more() {
        if (visibleCount >= history.length) await loadPage();
        visibleCount += 6;
    }
</script>

<section class="space-y-5 rounded-3xl border border-border bg-bg-elevated p-6" aria-label={t('packFollowup.history')}>
    <div class="flex flex-wrap items-start justify-between gap-4"><div><h2 class="flex items-center gap-2 text-base font-semibold text-fg"><History class="h-5 w-5 text-brand-400" />{t('packFollowup.history')}</h2><p class="mt-2 text-xs leading-relaxed text-fg-muted">{t('packFollowup.historyHelp')}</p></div><label class="text-xs font-semibold text-fg">{t('catalog.versionChanges')}<select bind:value={selected} class="mt-2 block max-w-72 rounded-xl border border-border bg-bg-subtle p-3 text-sm" disabled={!history.length}><option value="">{t('packFollowup.allVersions')}</option>{#each history as version (version.id)}<option value={version.id}>{version.name || version.versionNumber || version.id}</option>{/each}</select></label></div>
    {#if !shown.length && !loadingPage}<p class="text-sm text-fg-muted">{t('catalog.noVersions')}</p>{/if}
    <ol class="space-y-4">
        {#each shown as version (version.id)}
            {@const entry = entries[version.id]}
            <li class="overflow-hidden rounded-2xl border border-border bg-bg-subtle p-5" data-changelog-version={version.id}>
                <h3 class="mb-4 text-sm font-semibold text-fg">{version.name || version.versionNumber || version.id}</h3>
                {#if !entry || entry.loading}<p role="status" class="flex items-center gap-2 text-xs text-fg-muted"><LoaderCircle class="h-4 w-4 animate-spin" />{t('common.loading')}</p>{:else if entry.error}<p role="alert" class="text-xs text-danger">{entry.error}</p><button type="button" class={button({variant:'ghost',size:'sm',class:'mt-2'})} onclick={() => loadEntry(version.id)}><RefreshCw class="h-3 w-3" />{t('ui.b9e10688be012d8b')}</button>{:else if entry.body}<div class="prose max-w-none break-words text-sm text-fg [&_a]:text-brand-400 [&_img]:max-w-full [&_pre]:overflow-x-auto">{@html renderBody(entry.body)}</div>{:else}<p class="text-xs text-fg-muted">{t('catalog.noChangelog')}</p>{/if}
            </li>
        {/each}
    </ol>
    {#if pageError}<p role="alert" class="text-xs text-danger">{pageError}</p><button type="button" class={button({variant:'secondary',size:'sm'})} onclick={() => loadPage()}>{t('ui.b9e10688be012d8b')}</button>{/if}
    {#if loadingPage}<p role="status" class="flex items-center gap-2 text-xs text-fg-muted"><LoaderCircle class="h-4 w-4 animate-spin" />{t('common.loading')}</p>{/if}
    {#if hasMore}<button type="button" class={button({variant:'secondary',block:true})} disabled={loadingPage} onclick={more}>{t('packFollowup.moreHistory')}</button>{/if}
</section>
