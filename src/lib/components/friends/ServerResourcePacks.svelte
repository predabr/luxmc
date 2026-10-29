<script lang="ts">
    import { serverMessage } from '$lib/utils/serverMessages';
    import { onMount } from 'svelte';
    import { open } from '@tauri-apps/plugin-dialog';
    import { Upload, Image, Check, Download, Search, LoaderCircle, ShieldCheck, Trash2 } from 'lucide-svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from './serverButton';
    import { serverManage, serverSettings, type ServerContent, type ServerSettings } from '$lib/api/localServers';
    import { modsSearch } from '$lib/api/mods';
    import type { ModSearchResultItem } from '$lib/api/types';
    let { id, mcVersion, running, pro, onchanged }: {id:string;mcVersion:string;running:boolean;pro:boolean;onchanged?:()=>void}=$props();
    let files=$state<ServerContent[]>([]);let settings=$state<ServerSettings|null>(null);let required=$state(false);let busy=$state(false);let error=$state('');let info=$state('');let query=$state('');let results=$state<ModSearchResultItem[]>([]);let searching=$state(false);let disposed=false;let icons=$state<Record<string,string>>({});let deleteFile=$state('');
    async function load() {const [items,value]=await Promise.all([serverManage<ServerContent[]>({action:'contents',id}),serverSettings(id)]);if(!disposed){files=items.filter(item=>item.kind==='resourcepack');settings=value;required=value.requireResourcePack;}}
    onMount(()=>{void load().catch(cause=>{error=serverMessage(cause);});return()=>{disposed=true;};});
    async function act(action:()=>Promise<unknown>){if(busy)return;busy=true;error='';info='';try{await action();}catch(cause){if(!disposed)error=serverMessage(cause);}finally{if(!disposed){try{await load();onchanged?.();}catch{}busy=false;}}}
    async function imported(action:()=>Promise<unknown>,file?:string){
        await act(async()=>{
            const before=await serverManage<ServerContent[]>({action:'contents',id});await action();
            const current=await serverManage<ServerContent[]>({action:'contents',id});
            if(!running && !before.some(item=>item.staged) && !current.some(item=>item.staged && item.kind!=='resourcepack')){
                await serverManage({action:'apply_content',id});
                const items=await serverManage<ServerContent[]>({action:'contents',id});
                const added=items.filter(item=>item.kind==='resourcepack' && !item.staged && (file?item.name===file:!before.some(previous=>previous.name===item.name && previous.kind===item.kind)));
                if(added.length===1)await serverManage({action:'use_resource_pack',id,file:added[0].name,required});
                info=t('packs.imported');
            }else info=t('packs.pending');
        });
    }
    async function choose(){try{const path=await open({multiple:false,filters:[{name:'ZIP',extensions:['zip']}]});if(typeof path==='string')await imported(()=>serverManage({action:'import_content',id,kind:'resourcepack',path}),path.split(/[\\/]/).pop());}catch(cause){error=serverMessage(cause);}}
    const iconNames=$derived(JSON.stringify(files.filter(file=>!file.staged).slice(0,24).map(file=>file.name)));
    $effect(()=>{const names=JSON.parse(iconNames);if(names.length)void serverManage<{name:string;icon?:string}[]>({action:'content_details',id,kind:'resourcepack',names}).then(values=>{if(!disposed)icons=Object.fromEntries(values.filter(value=>value.icon).map(value=>[value.name,value.icon!]));}).catch(()=>{});});
    async function removeFile(file:string){deleteFile='';await act(async()=>{if(settings?.resourcePackFile===file)await serverManage({action:'use_resource_pack',id,file:'',required:false});await serverManage({action:'remove_content',id,kind:'resourcepack',name:file});if(!running)await serverManage({action:'apply_content',id});});}
    async function search(){if(searching)return;searching=true;error='';try{const next=await modsSearch(query,mcVersion,12,0,'resourcepack','downloads',undefined,undefined,'modrinth');if(!disposed)results=next;}catch(cause){if(!disposed)error=serverMessage(cause);}finally{if(!disposed)searching=false;}}
</script>

<div class="space-y-5" data-server-resource-packs>
    <div class="rounded-2xl border border-brand-400/25 bg-brand-500/5 p-5"><div class="flex items-start gap-4"><div class="rounded-xl bg-brand-500/10 p-3 text-brand-400"><Image class="h-6 w-6" /></div><div class="min-w-0"><h4 class="font-semibold">{t('packs.title')}</h4><p class="mt-2 text-sm leading-7 text-fg-muted">{t('packs.how')}</p><p class="mt-2 text-xs leading-6 text-fg-muted">{t('packs.limit')}</p></div></div><div class="mt-4 flex flex-wrap items-center gap-4"><button type="button" class={button({variant:'primary'})} disabled={busy || !pro} onclick={choose}><Upload class="h-4 w-4" />{t('packs.import')}</button><label class="flex items-center gap-2 text-sm"><input type="checkbox" bind:checked={required} disabled={busy || running || !pro} />{t('serverPanel.requirePack')}</label>{#if settings?.resourcePackFile}<button type="button" class={button({variant:'secondary',size:'sm'})} disabled={busy || running || !pro} onclick={()=>act(()=>serverManage({action:'use_resource_pack',id,file:settings!.resourcePackFile,required}))}><ShieldCheck class="h-4 w-4" />{t('packs.saveRequirement')}</button>{/if}</div></div>
    {#if error}<p role="alert" class="rounded-xl border border-danger/30 p-4 text-sm leading-6 text-danger">{error}</p>{/if}
    {#if info}<p role="status" class="text-sm leading-6 text-success">{info}</p>{/if}
    {#if busy}<p role="status" class="flex items-center gap-2 text-sm text-fg-muted"><LoaderCircle class="h-4 w-4 animate-spin" />{t('localServer.loading')}</p>{/if}
    {#if files.length}<div><h4 class="font-semibold">{t('serverContent.textures')}</h4><p class="mt-2 text-sm leading-6 text-fg-muted">{t('serverContent.chooseTexture')}</p></div>{/if}
    {#each files as file (`${file.name}:${file.staged}`)}<article class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border bg-bg-subtle p-4"><div class="flex min-w-0 flex-1 items-center gap-4"><div class="flex h-14 w-14 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border bg-bg-elevated">{#if icons[file.name]}<img src={icons[file.name]} alt="" class="h-full w-full object-contain" />{:else}<Image class="h-6 w-6 text-brand-400" />{/if}</div><div class="min-w-0"><h4 class="break-all text-sm font-semibold">{file.name}</h4><p class="mt-1 text-xs text-fg-muted">{(file.bytes/1048576).toFixed(1)} MiB · {t(file.staged?'serverPanel.pending':settings?.resourcePackFile===file.name?'packs.active':'serverPanel.installed')}</p></div></div>{#if !file.staged && !file.remove}<button type="button" class={button({variant:settings?.resourcePackFile===file.name?'secondary':'primary',size:'sm'})} disabled={busy || running || !pro} onclick={()=>act(()=>serverManage({action:'use_resource_pack',id,file:file.name,required}))}><Check class="h-4 w-4" />{t('packs.activate')}</button><button type="button" class={button({variant:'ghostDanger',size:'sm'})} disabled={busy || running || !pro} onclick={()=>deleteFile=file.name}><Trash2 class="h-4 w-4" />{t('serverContent.delete')}</button>{/if}{#if deleteFile===file.name}<div class="w-full rounded-xl border border-danger/25 p-3"><p class="text-xs leading-6">{t('serverContent.deleteConfirm')}</p><div class="mt-3 flex gap-2"><button type="button" class={button({variant:'danger',size:'sm'})} onclick={()=>removeFile(file.name)}>{t('serverContent.delete')}</button><button type="button" class={button({variant:'ghost',size:'sm'})} onclick={()=>deleteFile=''}>{t('localServer.cancel')}</button></div></div>{/if}</article>{/each}
    {#if settings?.resourcePackFile}<button type="button" class={button({variant:'ghost',size:'sm'})} disabled={busy || running || !pro} onclick={()=>act(()=>serverManage({action:'use_resource_pack',id,file:'',required:false}))}>{t('packs.disable')}</button>{/if}
    <form class="flex gap-2" onsubmit={event=>{event.preventDefault();void search();}}><input class="min-w-0 flex-1 rounded-xl border border-border bg-bg-subtle px-4 py-3 text-sm" aria-label={t('packs.search')} placeholder={t('packs.search')} bind:value={query} /><button class={button({variant:'secondary'})} disabled={searching}><Search class="h-4 w-4" />{t('serverPanel.search')}</button></form>
    <div class="grid gap-3 lg:grid-cols-2">{#each results as item (`${item.source}:${item.sourceId}`)}<article class="flex gap-3 rounded-xl border border-border p-4">{#if item.iconUrl}<img class="h-12 w-12 shrink-0 rounded-xl object-contain" src={item.iconUrl} alt="" loading="lazy" />{/if}<div class="min-w-0 flex-1"><h4 class="truncate text-sm font-semibold">{item.title}</h4><p class="mt-1 line-clamp-2 text-xs leading-5 text-fg-muted">{item.description}</p><button type="button" class={button({variant:'secondary',size:'sm',class:'mt-3'})} disabled={busy || !pro} onclick={()=>imported(()=>serverManage({action:'download_content',id,project:item.sourceId||item.slug,source:item.source,kind:'resourcepack'}))}><Download class="h-4 w-4" />{t('serverPanel.download')}</button></div></article>{/each}</div>
</div>
