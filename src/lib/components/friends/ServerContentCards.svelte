<script lang="ts">
    import { Package, Image, FileArchive, Power, Trash2, Check, X, ChevronLeft, ChevronRight } from 'lucide-svelte';
    import { serverManage, type ServerContent } from '$lib/api/localServers';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from './serverButton';
    let { id, kind, items, running, busy, pro, onchange }: {id:string;kind:string;items:ServerContent[];running:boolean;busy:boolean;pro:boolean;onchange:(action:()=>Promise<unknown>)=>void}=$props();
    type Detail={name:string;title?:string;icon?:string};
    let metadata=$state<Record<string,Detail>>({});let query=$state('');let page=$state(0);let confirm=$state('');
    const filtered=$derived(items.filter(item=>`${item.name} ${metadata[item.name]?.title||''}`.toLowerCase().includes(query.toLowerCase())));
    const pages=$derived(Math.max(1,Math.ceil(filtered.length/24)));
    const visible=$derived(filtered.slice(Math.min(page,pages-1)*24,(Math.min(page,pages-1)+1)*24));
    async function details(names:string[]){if(!names.length)return;try{const values=await serverManage<Detail[]>({action:'content_details',id,kind,names});metadata={...metadata,...Object.fromEntries(values.map(value=>[value.name,value]))};}catch{}}
    const detailNames=$derived(JSON.stringify(visible.filter(item=>!item.staged).map(item=>item.name)));
    $effect(()=>{void details(JSON.parse(detailNames));});
    function remove(item:ServerContent){confirm='';onchange(async()=>{await serverManage({action:'remove_content',id,kind:item.kind,name:item.name});if(!running)await serverManage({action:'apply_content',id});});}
</script>

<div class="space-y-4" data-server-content-cards>
    <div class="flex flex-wrap items-center justify-between gap-3"><h4 class="text-sm font-semibold">{t('serverPanel.serverFiles')} <span class="ml-2 text-fg-muted">{items.length}</span></h4><input class="min-w-0 rounded-xl border border-border bg-bg-subtle px-4 py-3 text-sm" aria-label={t('serverContent.filter')} placeholder={t('serverContent.filter')} bind:value={query} oninput={()=>page=0} /></div>
    {#if !filtered.length}<p class="rounded-2xl border border-dashed border-border p-8 text-sm text-fg-muted">{t('serverPanel.noContents')}</p>{/if}
    <div class="grid gap-3 xl:grid-cols-2">{#each visible as item (`${item.name}:${item.staged}`)}{@const disabled=item.name.endsWith('.disabled')}{@const detail=metadata[item.name]}
        <article class="content-card group rounded-2xl border border-border bg-bg-subtle p-4" class:opacity-60={disabled}>
            <div class="flex items-start gap-4"><div class="flex h-16 w-16 shrink-0 items-center justify-center overflow-hidden rounded-2xl border border-border bg-bg-elevated">{#if detail?.icon}<img src={detail.icon} alt="" loading="lazy" decoding="async" class="h-full w-full object-contain" />{:else if kind==='resourcepack'}<Image class="h-7 w-7 text-brand-400" />{:else if kind==='datapack'}<FileArchive class="h-7 w-7 text-brand-400" />{:else}<Package class="h-7 w-7 text-brand-400" />{/if}</div><div class="min-w-0 flex-1"><h4 class="break-words text-sm font-semibold leading-6">{detail?.title||item.name.replace(/\.(jar|zip)(\.disabled)?$/i,'')}</h4><p class="mt-1 break-all text-xs leading-5 text-fg-muted">{item.name}</p><p class="mt-2 text-xs tabular-nums text-fg-muted">{(item.bytes/1048576).toFixed(1)} MiB · <span class={item.staged?'text-warning':disabled?'text-fg-muted':'text-success'}>{t(item.staged?item.remove?'serverPanel.pendingRemove':'serverPanel.pending':disabled?'serverContent.disabled':'serverContent.enabled')}</span></p></div></div>
            {#if !item.staged}<div class="mt-4 flex flex-wrap items-center justify-end gap-2 border-t border-border pt-3">{#if kind!=='resourcepack'}<button type="button" class={button({variant:disabled?'primary':'secondary',size:'sm'})} disabled={busy||running||!pro} onclick={()=>onchange(()=>serverManage({action:'toggle_content',id,kind:item.kind,name:item.name,enabled:disabled}))}><Power class="h-4 w-4" />{t(disabled?'serverContent.enable':'serverContent.disable')}</button>{/if}<button type="button" class={button({variant:'ghostDanger',size:'sm'})} disabled={busy||!pro} onclick={()=>confirm=item.name}><Trash2 class="h-4 w-4" />{t('serverContent.delete')}</button></div>{/if}
            {#if confirm===item.name}<div class="mt-3 rounded-xl border border-danger/25 bg-danger/5 p-3"><p class="text-xs leading-6">{t(running?'serverContent.deletePending':'serverContent.deleteConfirm')}</p><div class="mt-2 flex gap-2"><button type="button" class={button({variant:'danger',size:'sm'})} disabled={busy} onclick={()=>remove(item)}><Check class="h-4 w-4" />{t('serverContent.delete')}</button><button type="button" aria-label={t('localServer.cancel')} class={button({variant:'ghost',size:'sm'})} onclick={()=>confirm=''}><X class="h-4 w-4" /></button></div></div>{/if}
        </article>
    {/each}</div>
    {#if pages>1}<div class="flex items-center justify-center gap-4"><button type="button" aria-label={t('serverContent.previous')} class={button({size:'icon'})} disabled={page===0} onclick={()=>page--}><ChevronLeft class="h-4 w-4" /></button><span class="text-xs tabular-nums text-fg-muted">{Math.min(page,pages-1)+1} / {pages}</span><button type="button" aria-label={t('serverContent.next')} class={button({size:'icon'})} disabled={page>=pages-1} onclick={()=>page++}><ChevronRight class="h-4 w-4" /></button></div>{/if}
</div>

<style>
    .content-card{animation:content-enter .2s ease-out;transition:transform .18s ease,border-color .18s ease,box-shadow .18s ease}.content-card:hover{transform:translateY(-2px);border-color:rgb(var(--brand-400)/.4);box-shadow:0 10px 24px rgb(var(--bg-overlay)/.15)}@keyframes content-enter{from{opacity:0;transform:translateY(5px)}to{opacity:1;transform:translateY(0)}}@media(prefers-reduced-motion:reduce){.content-card{animation:none;transition:none}.content-card:hover{transform:none}}
</style>
