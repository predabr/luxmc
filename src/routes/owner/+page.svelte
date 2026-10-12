<script lang="ts">
    import { onMount } from 'svelte';
    import { Users, Gamepad2, ShieldCheck, Search, LoaderCircle, ArrowLeft, ArrowRight, RefreshCw, Tags, Trash2 } from 'lucide-svelte';
    import { friendsState } from '$lib/stores/friends.svelte';
    import { ownerUsers, ownerSetTags, ownerDeleteProfile, type OwnerOverview, type OwnerUser } from '$lib/api/owner';
    import Modal from '$lib/components/ui/Modal.svelte';
    import MinecraftAvatar from '$lib/components/ui/MinecraftAvatar.svelte';
    import ProfileBadge from '$lib/components/ui/ProfileBadge.svelte';
    import { button } from '$lib/components/ui/button';
    import { pageMotion } from '$lib/actions/pageMotion';
    let overview = $state<OwnerOverview | null>(null);
    let query = $state('');
    let offset = $state(0);
    let busy = $state(false);
    let error = $state('');
    let saving = $state('');
    let drafts = $state<Record<string,string>>({});
    let updatedAt = $state('');
    let deletion = $state<OwnerUser | null>(null);
    let confirmation = $state('');
    let deleting = $state(false);
    let deletionError = $state('');
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    const allowed = $derived(friendsState.ownProfile?.role === 'owner');
    async function refresh(search = query, start = offset) {
        if (busy || !allowed) return;
        busy = true; error = '';
        try { const result = await ownerUsers(friendsState.accountId,search,start); if (!disposed) { overview = result; offset = start; updatedAt = new Date().toLocaleTimeString(); } }
        catch (failure) { if (!disposed) error = String(failure); }
        finally { if (!disposed) busy = false; }
    }
    onMount(() => {
        async function poll() { await refresh(); if (!disposed) timer = setTimeout(poll,15000); }
        void poll();
        return () => { disposed = true; clearTimeout(timer); };
    });
    $effect(() => { if (allowed && !overview && !busy && !error) void refresh(); });
    async function save(user: OwnerUser, tags: string[]) {
        if (saving) return;
        saving = user.id; error = '';
        try { const saved = await ownerSetTags(friendsState.accountId,user.id,tags); if (overview) overview.users = overview.users.map(item => item.id === user.id ? {...item,tags:saved} : item); drafts[user.id] = ''; if (user.id === friendsState.me?.id) await friendsState.refreshProfile(); await friendsState.refresh(); }
        catch (failure) { error = String(failure); }
        finally { saving = ''; }
    }
    function askDelete(user: OwnerUser) { deletion = user; confirmation = ''; deletionError = ''; }
    function closeDelete() { if (!deleting) deletion = null; }
    async function removeProfile() {
        if (!deletion || deleting || confirmation !== deletion.username) return;
        deleting = true; deletionError = '';
        try {
            await ownerDeleteProfile(friendsState.accountId,deletion.id,confirmation);
            deletion = null;
            await refresh(query,0);
            await friendsState.refresh();
        } catch (failure) { deletionError = String(failure); }
        finally { deleting = false; }
    }
</script>

<svelte:head><title>Administração · Luxmc</title></svelte:head>
<div class="mx-auto w-full max-w-6xl space-y-6 pb-10" use:pageMotion={'owner'}>
    <a href="/friends" class={button({variant:'ghost',size:'sm'})}><ArrowLeft class="h-4 w-4" />Amigos</a>
    <header class="product-header rounded-3xl border border-brand-400/20 bg-bg-elevated luxmc-card p-6"><p class="page-eyebrow flex items-center gap-2"><ShieldCheck class="h-4 w-4" />Acesso do dono</p><h1 class="mt-3 text-3xl font-semibold text-fg">Comunidade Luxmc</h1><p class="mt-2 text-sm text-fg-muted">Acompanhe a comunidade. Só você pode editar as tags e remover perfis; as tags aparecem para todos no Luxmc.</p></header>
    {#if !allowed}<p role="status" class="rounded-3xl border border-border bg-bg-elevated luxmc-card p-6 text-sm text-fg-muted">Este painel está disponível apenas para a conta autenticada do dono.</p>{:else}
        {#if error}<p role="alert" class="rounded-2xl border border-danger/25 bg-danger/10 p-4 text-sm text-danger">{error}</p>{/if}
        <div class="grid gap-4 sm:grid-cols-3">{#each [{label:'Online agora',value:overview?.stats.online,icon:Users},{label:'Jogando agora',value:overview?.stats.playing,icon:Gamepad2},{label:'Perfis cadastrados',value:overview?.stats.total,icon:ShieldCheck}] as metric}<section class="rounded-3xl border border-border bg-bg-elevated luxmc-card p-5"><metric.icon class="h-5 w-5 text-brand-400" /><p class="mt-4 text-3xl font-semibold text-fg">{metric.value ?? '—'}</p><p class="mt-1 text-sm text-fg-muted">{metric.label}</p></section>{/each}</div>
        <section class="overflow-hidden rounded-3xl border border-border bg-bg-elevated luxmc-card"><form class="flex flex-wrap items-center gap-3 border-b border-border p-5" onsubmit={(event) => {event.preventDefault(); void refresh(query,0);}}><label class="relative min-w-56 flex-1"><span class="sr-only">Buscar usuário</span><Search class="absolute left-3 top-3.5 h-4 w-4 text-fg-muted" /><input bind:value={query} maxlength="80" placeholder="Buscar nome ou perfil" class="w-full rounded-xl border border-border bg-bg py-3 pl-10 pr-4 text-sm text-fg" /></label><button type="submit" class={button({variant:'primary'})} disabled={busy}>Buscar</button><button type="button" aria-label="Atualizar usuários" class={button({variant:'secondary'})} disabled={busy} onclick={() => refresh()}>{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<RefreshCw class="h-4 w-4" />{/if}</button></form>
            <div class="space-y-3 p-5">
                {#each overview?.users ?? [] as user (user.id)}
                    <article class="rounded-2xl border border-border bg-bg/35 p-4">
                        <div class="flex flex-wrap items-center gap-3">
                            <MinecraftAvatar username={user.username} avatarUrl={user.avatarUrl} status={user.status === 'offline' ? 'offline' : 'online'} class="h-12 w-12" />
                            <div class="min-w-0 flex-1"><p class="truncate font-semibold text-fg">{user.displayName || user.username}</p><p class="mt-1 text-xs text-fg-muted">{user.username} · {user.status === 'in_game' ? 'Jogando' : user.status === 'online' ? 'Online' : 'Offline'}</p></div>
                            {#if user.id === friendsState.me?.id}<span class="rounded-full bg-brand-500/10 px-3 py-1 text-xs text-brand-400">Você</span>{:else}<button type="button" aria-label={`Excluir perfil de ${user.username}`} class={button({variant:'ghostDanger',size:'sm'})} disabled={!!saving || deleting} onclick={() => askDelete(user)}><Trash2 class="h-4 w-4" />Excluir perfil</button>{/if}
                        </div>
                        <div class="mt-5 rounded-xl border border-border bg-bg-elevated/40 p-3.5">
                            <div class="mb-3 flex items-center gap-2 text-[11px] font-medium text-fg-muted"><Tags class="h-3.5 w-3.5" /><span>Tags do perfil</span><span class="ml-auto tabular-nums">{user.tags.length}/5</span></div>
                            <div class="flex min-h-9 flex-wrap items-center gap-2">
                                {#each user.tags as tag (tag)}<ProfileBadge label={tag} size="md" action="remove" disabled={!!saving} onaction={() => save(user,user.tags.filter(value => value !== tag))} />{:else}<p class="text-xs text-fg-subtle">Escolha uma tag abaixo para destacar este perfil.</p>{/each}
                            </div>
                            <div class="mt-3 flex flex-wrap items-center gap-2 border-t border-border pt-3">
                                {#each ['Testador','Amigo','Plus','Pro'].filter(tag => !user.tags.includes(tag)) as tag}<ProfileBadge label={tag} action="add" disabled={!!saving || user.tags.length >= 5} onaction={() => save(user,[...user.tags,tag])} />{/each}
                                <form class="flex min-w-0 flex-wrap gap-2 sm:ml-auto" onsubmit={(event) => {event.preventDefault(); const tag = drafts[user.id]?.trim(); if (tag) void save(user,[...user.tags,tag]);}}><input aria-label={`Nova tag para ${user.username}`} bind:value={drafts[user.id]} maxlength="24" placeholder="Tag personalizada" disabled={user.tags.length >= 5} class="w-40 rounded-lg border border-border bg-bg px-3 py-2 text-xs text-fg" /><button type="submit" disabled={!!saving || user.tags.length >= 5 || !drafts[user.id]?.trim()} class={button({variant:'secondary',size:'sm'})}>{saving === user.id ? 'Salvando…' : 'Adicionar'}</button></form>
                            </div>
                        </div>
                    </article>
                {:else}{#if !busy}<p class="p-5 text-center text-sm text-fg-muted">Nenhum perfil encontrado.</p>{/if}{/each}
            </div>
            <footer class="flex flex-wrap items-center justify-between gap-3 border-t border-border p-5 text-xs text-fg-muted"><p>{overview?.total ?? 0} perfis nesta busca · {updatedAt ? `Atualizado às ${updatedAt}` : 'Carregando…'}</p><div class="flex gap-2"><button class={button({variant:'secondary',size:'sm'})} disabled={busy || offset === 0} onclick={() => refresh(query,Math.max(0,offset-24))}><ArrowLeft class="h-4 w-4" />Anterior</button><button class={button({variant:'secondary',size:'sm'})} disabled={busy || overview?.nextOffset == null} onclick={() => refresh(query,overview!.nextOffset!)}>Próxima<ArrowRight class="h-4 w-4" /></button></div></footer>
        </section><p class="text-xs leading-5 text-fg-muted">A contagem usa a presença recebida nos últimos 75 segundos. Perfis sem conexão ou com presença desativada não entram no total online. As tags são públicas e não concedem permissões de administração.</p>
    {/if}
</div>

<Modal isOpen={!!deletion} onClose={closeDelete} title="Excluir perfil da comunidade" showClose={!deleting}>
    {#if deletion}
        <form class="space-y-5" onsubmit={(event) => { event.preventDefault(); void removeProfile(); }}>
            <p class="text-sm leading-6 text-fg-muted">O perfil de <strong class="text-fg">{deletion.username}</strong>, suas amizades, tags e personalização pública serão removidos da comunidade. A conta do jogo e os mundos no computador da pessoa são preservados.</p>
            <p class="text-sm leading-6 text-fg-muted">Este perfil deixará de acessar o serviço social do Luxmc.</p>
            <label class="block space-y-2 text-sm text-fg"><span>Digite <strong>{deletion.username}</strong> para confirmar</span><input bind:value={confirmation} autocomplete="off" disabled={deleting} class="w-full rounded-xl border border-border bg-bg px-4 py-3" /></label>
            {#if deletionError}<p role="alert" class="text-sm text-danger">{deletionError}</p>{/if}
            <div class="flex justify-end gap-3"><button type="button" class={button({variant:'secondary'})} disabled={deleting} onclick={closeDelete}>Cancelar</button><button type="submit" class={button({variant:'ghostDanger'})} disabled={deleting || confirmation !== deletion.username}>{#if deleting}<LoaderCircle class="h-4 w-4 animate-spin" />Excluindo…{:else}<Trash2 class="h-4 w-4" />Excluir perfil{/if}</button></div>
        </form>
    {/if}
</Modal>
