<script lang="ts">
    import type { PackCollection, PackEntry } from '$lib/api/social';
    import { instancePackReference } from '$lib/api/studio';
    import { modsProjectDetails } from '$lib/api/mods';
    import { modpackInstallation } from '$lib/stores/modpackInstallation.svelte';
    import { profiles } from '$lib/stores/profiles.svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import GlassSelect from '$lib/components/ui/GlassSelect.svelte';
    import { Library, Plus, Download, X, Link2 } from 'lucide-svelte';
    let { value=$bindable([]), editing=false, disabled=false }: {value?:PackCollection[];editing?:boolean;disabled?:boolean}=$props();
    let title=$state(''); let selected=$state(''); let busy=$state(false); let error=$state('');
    async function add(collection:PackCollection) {
        if (!selected || busy) return; busy=true;error='';
        try {
            const entry=await instancePackReference(selected);
            if (!entry) throw new Error(t('studio.linkedOnly'));
            if (collection.entries.some(item=>item.source===entry.source && item.projectId===entry.projectId && item.versionId===entry.versionId)) return;
            value=value.map(item=>item.id===collection.id ? {...item,entries:[...item.entries,entry]}:item);
        } catch(cause) {error=String(cause);} finally {busy=false;}
    }
    async function install(entry:PackEntry) {
        if (busy) return; busy=true;error='';
        try {
            const project=await modsProjectDetails(entry.projectId,entry.source);
            await modpackInstallation.start({sourceId:entry.projectId,source:entry.source,title:entry.name,slug:project.slug,description:project.description,iconUrl:project.iconUrl,bannerUrl:null,downloads:project.downloads,author:project.author?.name || null,versions:project.gameVersions,categories:project.categories},entry.name,4096,'',entry.versionId);
        } catch(cause) {error=String(cause);} finally {busy=false;}
    }
</script>
<section class="space-y-3 rounded-2xl border border-border bg-bg-subtle/60 p-4">
    <h3 class="flex items-center gap-2 text-sm font-semibold"><Library class="h-4 w-4 text-brand-400" />{t('studio.collections')}</h3>
    <p class="text-xs leading-relaxed text-fg-muted">{t('studio.collectionsHelp')}</p>
    {#if error}<p role="alert" class="text-xs text-danger">{error}</p>{/if}
    {#if editing}
        <div class="flex gap-2"><input aria-label={t('studio.collectionName')} bind:value={title} maxlength="80" disabled={disabled || busy} class="min-w-0 flex-1 rounded-xl border border-border bg-bg-elevated px-3 py-2 text-sm" placeholder={t('studio.collectionName')} /><button type="button" class={button({variant:'secondary',size:'sm'})} aria-label={t('common.add')} disabled={disabled || busy || !title.trim() || value.length>=6} onclick={()=>{value=[...value,{id:crypto.randomUUID(),title:title.trim(),description:'',entries:[]}];title='';}}><Plus class="h-4 w-4" /></button></div>
        <GlassSelect bind:value={selected} options={profiles.list.map(profile=>({value:profile.id,label:profile.name}))} label={t('studio.chooseLinked')} disabled={disabled || busy} />
    {/if}
    {#each value as collection (collection.id)}
        <article class="space-y-3 rounded-2xl border border-border bg-bg-elevated p-4">
            <div class="flex items-center justify-between gap-3"><h4 class="min-w-0 truncate text-sm font-semibold">{collection.title}</h4>{#if editing}<button type="button" class={button({variant:'ghostDanger',size:'icon'})} aria-label={t('common.remove')+': '+collection.title} disabled={disabled || busy} onclick={()=>value=value.filter(item=>item.id!==collection.id)}><X class="h-4 w-4" /></button>{/if}</div>
            {#if editing}<label class="block text-xs text-fg-muted">{t('publicProfile.about')}<textarea maxlength="240" rows="2" disabled={disabled || busy} value={collection.description} oninput={event=>value=value.map(item=>item.id===collection.id?{...item,description:event.currentTarget.value}:item)} class="mt-2 w-full rounded-xl border border-border bg-bg-subtle p-3 text-sm text-fg"></textarea></label>{:else if collection.description}<p class="whitespace-pre-wrap text-xs leading-relaxed text-fg-muted">{collection.description}</p>{/if}
            {#each collection.entries as entry}
                <div class="flex items-center gap-3 rounded-xl border border-border p-3"><Link2 class="h-4 w-4 shrink-0 text-brand-400" /><div class="min-w-0 flex-1"><p class="truncate text-sm font-medium">{entry.name}</p><p class="mt-1 break-all text-[10px] text-fg-muted">{entry.source} · {entry.projectId} · {entry.versionId}</p></div>{#if editing}<button type="button" disabled={disabled || busy} aria-label={t('common.remove')+': '+entry.name} class={button({variant:'ghostDanger',size:'icon'})} onclick={()=>value=value.map(item=>item.id===collection.id?{...item,entries:item.entries.filter(candidate=>candidate!==entry)}:item)}><X class="h-4 w-4" /></button>{:else}<button type="button" disabled={busy} aria-label={t('studio.installExact')+': '+entry.name} class={button({variant:'secondary',size:'icon'})} onclick={()=>void install(entry)}><Download class="h-4 w-4" /></button>{/if}</div>
            {/each}
            {#if editing}<button type="button" disabled={disabled || busy || !selected || collection.entries.length>=8} class={button({variant:'secondary',size:'sm'})} onclick={()=>void add(collection)}><Plus class="h-4 w-4" />{t('studio.addLinked')}</button>{/if}
        </article>
    {/each}
</section>
