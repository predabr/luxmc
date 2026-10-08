<script lang="ts">
    import { dependencyGraph, type DependencyNode } from '$lib/api/studio';
    import { removalImpact } from '$lib/utils/dependencyImpact';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import { GitBranch, RefreshCw, ArrowRight } from 'lucide-svelte';
    let { profileId }: {profileId:string}=$props();
    let nodes=$state<DependencyNode[]>([]);let selected=$state('');let loaded=$state('');let search=$state('');let limit=$state(60);let busy=$state(false);let error=$state('');
    const current=$derived(nodes.find(node=>node.id===selected));
    const filtered=$derived(nodes.filter(node=>(node.name+' '+node.id).toLowerCase().includes(search.toLowerCase())));
    const affected=$derived(current ? removalImpact(nodes,current.file) : []);
    $effect(()=>{if(loaded!==profileId){nodes=[];selected='';}});
    async function load(){if(busy || !profileId)return;const id=profileId;busy=true;error='';try{const value=await dependencyGraph(id);if(id===profileId){nodes=value;loaded=id;selected=value[0]?.id||'';}}catch(cause){error=String(cause);}finally{busy=false;}}
</script>
<section class="space-y-4 rounded-3xl border border-border bg-bg-elevated p-5">
    <div class="flex flex-wrap items-start justify-between gap-3"><div><h2 class="flex gap-2 text-lg font-semibold"><GitBranch class="h-5 w-5 text-brand-400" />{t('studio.dependencies')}</h2><p class="mt-2 text-xs leading-relaxed text-fg-muted">{t('studio.dependenciesHelp')}</p></div><button type="button" disabled={busy || !profileId} class={button({variant:'secondary',size:'sm'})} onclick={()=>void load()}><RefreshCw class="h-4 w-4" />{busy?t('common.loading'):t('studio.scan')}</button></div>
    {#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}
    {#if loaded===profileId}
        <input aria-label={t('studio.findMod')} bind:value={search} oninput={()=>limit=60} class="w-full rounded-xl border border-border bg-bg-subtle px-3 py-2 text-sm" placeholder={t('studio.findMod')} />
        <div class="grid gap-4 md:grid-cols-2">
            <div class="max-h-96 space-y-2 overflow-y-auto rounded-2xl border border-border p-2 custom-scrollbar" role="group" aria-label={t('studio.dependencies')}>
                {#each filtered.slice(0,limit) as node}<button type="button" aria-pressed={selected===node.id} class={button({variant:selected===node.id?'ghostBrand':'ghost',block:true,class:'justify-start text-left'})} onclick={()=>selected=node.id}><span class="truncate">{node.name}</span><span class="ml-auto text-xs text-warning">{node.missing.length || ''}</span></button>{/each}
                {#if filtered.length>limit}<button type="button" class={button({variant:'secondary',block:true,size:'sm'})} onclick={()=>limit+=60}>{t('studio.more')}</button>{/if}
                {#if !filtered.length}<p class="p-3 text-xs text-fg-muted">{t('studio.noMods')}</p>{/if}
            </div>
            {#if current}<article class="min-w-0 space-y-4 rounded-2xl border border-brand-400/25 bg-brand-400/5 p-4"><div><h3 class="break-words font-semibold">{current.name}</h3><p class="mt-1 break-all text-xs text-fg-muted">{current.file} · {current.version}</p></div><div><p class="text-xs font-semibold">{t('studio.requires')}</p><div class="mt-2 flex flex-wrap gap-2">{#each current.requires as id}<button type="button" class={button({variant:current.missing.includes(id)?'ghostDanger':'secondary',size:'sm'})} onclick={()=>{const provider=nodes.find(node=>node.id===id||node.provides.includes(id));if(provider)selected=provider.id;}}><ArrowRight class="h-3 w-3" />{id}</button>{/each}</div></div><div class="rounded-xl border border-warning/25 bg-warning/5 p-3"><h4 class="text-xs font-semibold">{t('studio.removalImpact')}</h4><p class="mt-2 break-words text-xs leading-relaxed text-fg-muted">{affected.length ? affected.map(node=>node.name).join(', '):t('studio.noDependents')}</p></div><p class="text-[11px] leading-relaxed text-fg-muted">{current.metadataKnown?t('studio.graphLimit'):t('studio.unknownMetadata')}</p></article>{/if}
        </div>
    {/if}
</section>
