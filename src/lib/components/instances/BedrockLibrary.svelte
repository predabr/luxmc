<script lang="ts">
    import { onMount } from "svelte";
    import { bedrock } from "$lib/stores/bedrock.svelte";
    import { bedrockOpen, bedrockRemove, bedrockRename } from "$lib/api/bedrock";
    import { button } from "$lib/components/ui/button";
    import Button from "$lib/components/ui/Button.svelte";
    import Modal from "$lib/components/ui/Modal.svelte";
    import { Blocks, Trash2, Play, Pencil, LoaderCircle, HardDrive } from "lucide-svelte";
    let busy = $state("");
    let error = $state("");
    let removing = $state("");
    let editing = $state("");
    let name = $state("");
    let notice = $state("");
    const selected = $derived(bedrock.value?.instances.find(instance => instance.id === removing));
    onMount(() => { void bedrock.refresh(); });
    async function run(id: string, action: () => Promise<unknown>) {
        busy = id; error = "";
        try { await action(); } catch (failure) { error = String(failure); }
        finally { busy = ""; }
    }
    async function openInstance(id: string) {
        const result = await bedrockOpen(id);
        notice = result.notice ?? "";
    }
</script>

{#if bedrock.value?.instances.length}
    <section class="space-y-5" aria-label="Instâncias Bedrock">
        <div class="flex items-center gap-3"><Blocks class="h-5 w-5 text-brand-500" /><h2 class="text-lg font-semibold text-fg">Bedrock Edition</h2><span class="rounded-full border border-border bg-bg-elevated px-2 py-0.5 text-xs text-fg-muted">{bedrock.value.instances.length}</span></div>
        <div class="grid gap-5 sm:grid-cols-2 xl:grid-cols-3">
            {#each bedrock.value.instances as instance (instance.id)}
                {@const installation = bedrock.value.installations.find(item => item.profileId === instance.profileId && item.id === instance.installationId)}
                <article data-bedrock-instance={instance.id} class="relative overflow-hidden rounded-3xl border border-border bg-bg-elevated shadow-soft">
                    <div class="relative h-40 overflow-hidden"><img src="/vanilla_banner.png" alt="" class="h-full w-full object-cover" loading="lazy" /><div class="absolute inset-0 bg-gradient-to-t from-bg-elevated to-transparent"></div><span class="absolute left-4 top-4 rounded-full border border-brand-500/30 bg-bg-elevated/90 px-3 py-1 text-xs font-semibold text-brand-400">BEDROCK</span><div class="absolute bottom-3 left-5 flex h-12 w-12 items-center justify-center rounded-2xl border border-border bg-bg-elevated shadow-soft"><Blocks class="h-7 w-7 text-brand-500" /></div></div>
                    <div class="space-y-4 p-5 pt-2"><div><h3 class="truncate text-base font-semibold text-fg" title={instance.name}>{instance.name}</h3><p class="mt-2 text-xs text-fg-muted">Minecraft {installation?.version || "Bedrock"} · {instance.profileId === "managed" ? "Instalada pelo Luxmc" : "Instalação importada"}</p></div><div class="flex items-center gap-2 border-t border-border pt-4 text-xs text-fg-muted"><HardDrive class="h-4 w-4" />{installation ? "Arquivos disponíveis" : "Instalação indisponível"}</div><div class="flex items-center justify-between gap-3"><div class="flex gap-1"><button class={button({variant:"ghost",size:"icon"})} aria-label={`Editar ${instance.name}`} disabled={!!busy} onclick={() => { name = instance.name; editing = instance.id; error = ""; }}><Pencil class="h-4 w-4" /></button><button class={button({variant:"ghost",size:"icon"})} aria-label={`Excluir ${instance.name}`} disabled={!!busy} onclick={() => { removing = instance.id; error = ""; }}><Trash2 class="h-4 w-4 text-danger" /></button></div><button class={button({variant:"play"})} disabled={!!busy || !installation || bedrock.installing} aria-busy={busy === instance.id} onclick={() => run(instance.id, () => openInstance(instance.id))}>{#if busy === instance.id}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Play class="h-4 w-4" />{/if}Jogar</button></div></div>
                </article>
            {/each}
        </div>
        {#if error && !removing && !editing}<p role="alert" class="rounded-xl border border-danger/30 bg-danger/5 p-3 text-sm text-danger">{error}</p>{/if}
        {#if notice}<p role="status" class="rounded-xl border border-warning/30 bg-warning/5 p-3 text-sm text-warning">{notice}</p>{/if}
    </section>
{/if}
<Modal isOpen={!!removing} title="Excluir instância Bedrock" showClose={!busy} onClose={() => { if (!busy) removing = ""; }}>
    <div class="space-y-5"><p class="text-sm text-fg">Excluir <strong>{selected?.name}</strong> e seus arquivos?</p><p class="text-sm leading-relaxed text-fg-muted">A versão instalada é desinstalada do Windows quando corresponde a esta instância. O pacote baixado pelo Luxmc também é apagado. Dados da edição Bedrock podem ser compartilhados entre versões; faça um backup de seus mundos antes de continuar.</p>{#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}<div class="flex justify-end gap-3"><Button disabled={!!busy} onclick={() => removing = ""}>Cancelar</Button><Button variant="danger" loading={busy === removing} disabled={!!busy} onclick={() => run(removing, async () => { await bedrockRemove(removing); await bedrock.refresh(); removing = ""; })}>Excluir definitivamente</Button></div></div>
</Modal>
<Modal isOpen={!!editing} title="Editar instância Bedrock" showClose={!busy} onClose={() => { if (!busy) editing = ""; }}>
    <div class="space-y-5"><label class="block space-y-2 text-sm text-fg">Nome da instância<input class="w-full rounded-xl border border-border bg-bg-elevated p-3" bind:value={name} maxlength="80" disabled={!!busy} /></label>{#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}<div class="flex justify-end gap-3"><Button disabled={!!busy} onclick={() => editing = ""}>Cancelar</Button><Button variant="primary" loading={busy === editing} disabled={!!busy || !name.trim()} onclick={() => run(editing, async () => { await bedrockRename(editing, name); await bedrock.refresh(); editing = ""; })}>Salvar</Button></div></div>
</Modal>
