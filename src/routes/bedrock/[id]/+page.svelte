<script lang="ts">
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import { open } from '@tauri-apps/plugin-dialog';
    import { bedrock } from '$lib/stores/bedrock.svelte';
    import { bedrockInstanceContent, bedrockImportContent, bedrockDeleteContent, bedrockOpenFolder, bedrockSelectWorldAccount, bedrockOpen, type BedrockInstanceContent, type BedrockContentEntry } from '$lib/api/bedrock';
    import { pageMotion } from '$lib/actions/pageMotion';
    import { button } from '$lib/components/ui/button';
    import Modal from '$lib/components/ui/Modal.svelte';
    import { ArrowLeft, Blocks, Play, Package, FolderOpen, Plus, Trash2, LoaderCircle, RefreshCw, ShieldCheck, Globe, Layers } from 'lucide-svelte';
    const id = $derived(page.params.id ?? '');
    const instance = $derived(bedrock.value?.instances.find(instance => instance.id === id));
    const installation = $derived(bedrock.value?.installations.find(installation => installation.id === instance?.installationId && installation.profileId === instance?.profileId));
    let content = $state<BedrockInstanceContent | null>(null);
    let tab = $state('overview');
    let busy = $state(false);
    let error = $state('');
    let notice = $state('');
    let removing = $state<BedrockContentEntry | null>(null);
    const entries = $derived(content?.entries.filter(entry => entry.kind === tab) ?? []);
    const tabs = [{ id: 'overview', title: 'Visão geral', icon: Blocks }, { id: 'resources', title: 'Pacotes de recursos', icon: Package }, { id: 'behaviors', title: 'Add-ons', icon: Layers }, { id: 'worlds', title: 'Mundos', icon: Globe }];
    async function load() { content = await bedrockInstanceContent(id); }
    async function run(action: () => Promise<unknown>) { if (busy) return; busy = true; error = ''; try { await action(); } catch (failure) { error = String(failure); } finally { busy = false; } }
    async function importContent() {
        const kind = tab;
        const file = await open({ title: kind === 'worlds' ? 'Importar mundo Bedrock' : 'Importar pacote Bedrock', multiple: false, filters: [{ name: 'Conteúdo Bedrock', extensions: kind === 'worlds' ? ['mcworld', 'zip'] : ['mcpack', 'zip'] }] });
        if (typeof file === 'string') await run(async () => { await bedrockImportContent(id, kind, file); await load(); });
    }
    onMount(() => { void run(async () => { await bedrock.refresh(); if (instance?.profileId === 'managed') await load(); }); });
</script>

<div class="mx-auto max-w-6xl space-y-6 p-6 sm:p-8">
    <a href="/instances" class={button({variant:'ghost',size:'sm'})}><ArrowLeft class="h-4 w-4" />Instâncias</a>
    {#if instance}
        <header data-motion-surface class="surface-glass relative overflow-hidden rounded-3xl border border-border p-6 sm:p-8">
            <div class="pointer-events-none absolute right-0 top-0 h-48 w-48 rounded-full bg-brand-500/10 blur-3xl"></div>
            <div class="relative flex flex-wrap items-center justify-between gap-5"><div class="flex items-center gap-4"><div class="grid h-16 w-16 place-items-center rounded-2xl border border-brand-400/20 bg-brand-500/10"><Blocks class="h-8 w-8 text-brand-400" /></div><div><p class="text-[10px] font-semibold uppercase tracking-widest text-brand-400">Minecraft Bedrock</p><h1 class="mt-1 text-2xl font-bold text-fg">{instance.name}</h1><p class="mt-2 text-sm text-fg-muted">Versão {installation?.version || 'indisponível'}</p></div></div><button class={button({variant:'play',size:'lg'})} disabled={busy || !installation} aria-busy={busy} onclick={() => run(async () => { const result = await bedrockOpen(id); notice = result.notice ?? ''; if (instance.profileId === 'managed') await load(); })}>{#if busy}<LoaderCircle class="h-5 w-5 animate-spin" />{:else}<Play class="h-5 w-5" />{/if}Jogar</button></div>
        </header>
        <nav class="flex flex-wrap gap-2 border-b border-border pb-3" aria-label="Conteúdo da instância Bedrock">{#each tabs as option}<button class="launcher-tab flex items-center gap-2 text-sm {tab === option.id ? 'bg-brand-500/10 text-brand-400' : 'text-fg-muted'}" aria-pressed={tab === option.id} onclick={() => tab = option.id}><option.icon class="h-4 w-4" />{option.title}{#if option.id !== 'overview'}<span class="text-xs text-fg-muted">{content?.entries.filter(entry => entry.kind === option.id).length ?? 0}</span>{/if}</button>{/each}</nav>
        {#if error}<p role="alert" class="rounded-xl border border-danger/30 bg-danger/5 p-4 text-sm text-danger">{error}</p>{/if}
        {#if notice}<p role="status" class="rounded-xl border border-warning/30 bg-warning/5 p-4 text-sm text-fg">{notice}</p>{/if}
        <section use:pageMotion={tab} class="surface-glass min-h-64 space-y-5 rounded-3xl border border-border p-6">
            {#if tab === 'overview'}
                <h2 class="text-lg font-semibold text-fg">Sua instalação</h2>
                <div class="grid gap-4 sm:grid-cols-3">{#each [{title:'Recursos',value:content?.entries.filter(entry => entry.kind === 'resources').length ?? 0},{title:'Add-ons',value:content?.entries.filter(entry => entry.kind === 'behaviors').length ?? 0},{title:'Mundos',value:content?.entries.filter(entry => entry.kind === 'worlds').length ?? 0}] as stat}<div class="rounded-2xl border border-border bg-bg-subtle p-5"><p class="text-xs text-fg-muted">{stat.title}</p><p class="mt-2 text-2xl font-semibold text-fg">{stat.value}</p></div>{/each}</div>
                {#if content}<p class="flex items-start gap-2 text-sm text-fg-muted"><ShieldCheck class="mt-0.5 h-4 w-4 shrink-0 text-brand-400" />{content.isolated ? 'Esta instância usa seus próprios arquivos de conteúdo e mundos.' : 'Sua pasta está pronta. A ligação com os dados do jogo é conferida ao clicar em Jogar.'}</p><code class="block break-all rounded-xl border border-border bg-bg-subtle p-3 text-xs text-fg-muted">{content.directory}</code><button class={button({variant:'secondary'})} disabled={busy} onclick={() => run(() => bedrockOpenFolder(id))}><FolderOpen class="h-4 w-4" />Abrir pasta da instância</button>{#if content.worldAccounts.length > 1}<label class="block space-y-2 text-sm text-fg">Conta dos mundos no Windows<select class="w-full max-w-lg rounded-xl border border-border bg-bg-elevated p-3" value={content.worldAccount ?? ''} disabled={busy} onchange={event => { const account = event.currentTarget.value; if (account) void run(async () => { await bedrockSelectWorldAccount(id, account); await load(); }); }}><option value="" disabled>Escolher conta do Bedrock</option>{#each content.worldAccounts as account}<option value={account}>{account}</option>{/each}</select></label>{/if}{:else if instance.profileId !== 'managed'}<p class="text-sm leading-relaxed text-fg-muted">Esta instalação foi importada de outro provedor. Instale uma versão pelo Luxmc para usar pastas e conteúdos gerenciados aqui.</p>{/if}
                <p class="text-xs leading-relaxed text-fg-subtle">O Luxmc baixa e gerencia o pacote da versão escolhida. O Windows executa o Bedrock e confere a licença Microsoft. Os add-ons precisam ser ativados nas configurações do mundo dentro do jogo.</p>
            {:else}
                <div class="flex flex-wrap items-center justify-between gap-3"><h2 class="text-lg font-semibold text-fg">{tabs.find(option => option.id === tab)?.title}</h2><div class="flex gap-2"><button class={button({variant:'secondary',size:'icon'})} aria-label="Atualizar conteúdos" disabled={busy || !content} onclick={() => run(load)}><RefreshCw class="h-4 w-4 {busy ? 'animate-spin' : ''}" /></button><button class={button({variant:'primary'})} disabled={busy || !content} onclick={importContent}><Plus class="h-4 w-4" />Importar {tab === 'worlds' ? '.mcworld' : '.mcpack'}</button></div></div>
                {#if !entries.length}<div class="grid min-h-48 place-items-center rounded-2xl border border-dashed border-border p-6 text-center"><div><Package class="mx-auto h-8 w-8 text-fg-subtle" /><p class="mt-3 text-sm text-fg-muted">Nenhum conteúdo nesta categoria.</p><p class="mt-2 text-xs text-fg-subtle">Importe um arquivo para adicioná-lo à pasta desta instância.</p></div></div>{:else}<div class="grid gap-3 sm:grid-cols-2">{#each entries as entry (entry.name)}<article class="flex items-center gap-3 rounded-2xl border border-border bg-bg-subtle p-4">{#if entry.icon}<img src={entry.icon} alt="" class="h-12 w-12 rounded-xl object-cover" />{:else}<Package class="h-10 w-10 shrink-0 text-brand-400" />{/if}<div class="min-w-0 flex-1"><h3 class="truncate text-sm font-semibold text-fg">{entry.title}</h3><p class="mt-1 text-xs text-fg-muted">Arquivo da instância</p></div><button class={button({variant:'ghostDanger',size:'icon'})} disabled={busy} aria-label={`Excluir ${entry.title}`} onclick={() => removing = entry}><Trash2 class="h-4 w-4" /></button></article>{/each}</div>{/if}
            {/if}
        </section>
    {:else if busy}<p role="status" class="flex items-center gap-3 text-fg-muted"><LoaderCircle class="h-5 w-5 animate-spin" />Carregando instância…</p>{:else}<p role="alert" class="text-fg-muted">Instância Bedrock não encontrada.</p>{/if}
</div>
<Modal isOpen={!!removing} title="Excluir conteúdo Bedrock" showClose={!busy} onClose={() => { if (!busy) removing = null; }}><div class="space-y-5"><p class="text-sm text-fg">Excluir definitivamente <strong>{removing?.title}</strong> desta instância?</p>{#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}<div class="flex justify-end gap-3"><button class={button({variant:'secondary'})} disabled={busy} onclick={() => removing = null}>Cancelar</button><button class={button({variant:'danger'})} disabled={busy} aria-busy={busy} onclick={() => { const entry = removing; if (entry) void run(async () => { await bedrockDeleteContent(id, entry.kind, entry.name); await load(); removing = null; }); }}>{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}Excluir definitivamente</button></div></div></Modal>
