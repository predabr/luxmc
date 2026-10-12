<script lang="ts">
    import { onMount } from 'svelte';
    import { fade, fly } from 'svelte/transition';
    import { Users, Plus, ArrowRight, Copy, LogOut, LoaderCircle, Lock, Unlock, RefreshCw, Wifi, ShieldCheck, Gamepad2, X } from 'lucide-svelte';
    import { hostWorld, joinTunnel, tunnelStatus, stopSession, kickTunnelMember, setTunnelLocked, refreshTunnelInvitation, reconnectTunnel, canReconnectTunnel, type TunnelStatus, type TunnelMember } from '$lib/api/tunnel';
    import { settings } from '$lib/stores/settings.svelte';
    import { friendsState } from '$lib/stores/friends.svelte';
    import { friendsNavigation } from '$lib/stores/friendsNavigation.svelte';
    import { deepLinks } from '$lib/stores/deepLinks.svelte';
    import { toast } from '$lib/stores/toasts.svelte';
    import { button } from '$lib/components/ui/button';
    import AccountBadges from '$lib/components/ui/AccountBadges.svelte';
    import RoomAvatar from './RoomAvatar.svelte';
    import type { ProfileStyle } from '$lib/utils/profileStyle';
    let status = $state<TunnelStatus | null>(null);
    let invitation = $state('');
    let busy = $state(false);
    let error = $state('');
    let ready = $state(false);
    let lost = $state(false);
    let hadSession = false;
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    const duration = $derived(settings.value.animations === false || settings.value.performanceMode || (settings.value.respectReducedMotion !== false && typeof window !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches) ? 0 : 260);
    const worlds = $derived(status?.worlds ?? []);
    $effect(() => { if (deepLinks.lan) { invitation = deepLinks.lan; deepLinks.lan = null; } });
    function visible(member: TunnelMember): TunnelMember & {profileStyle?:ProfileStyle;role?:'owner'|'member';tags?:string[]} {
        const friend = friendsState.list.find(friend => friend.username === member.username || friend.displayName === member.username);
        const own = friendsState.ownProfile;
        const self = own && (own.username === member.username || own.displayName === member.username);
        return { ...member, profileStyle:self ? own.profileStyle : friend?.profileStyle, role:self ? own.role : friend?.role, tags:self ? own.tags : friend?.tags, displayName: self ? own.displayName || member.username : friend?.displayName || member.displayName || member.username, avatarUrl: self ? own.portrait || member.avatarUrl : friend?.avatarUrl || member.avatarUrl };
    }
    onMount(() => {
        friendsNavigation.tab = 'p2p';
        async function refresh() {
            if (busy) { if (!disposed) timer = setTimeout(refresh,1000); return; }
            try {
                const next = await tunnelStatus();
                if (disposed) return;
                status = next; lost = hadSession && !next; hadSession ||= !!next; ready = true;
            } catch (failure) { if (!disposed) error = String(failure); }
            if (!disposed) timer = setTimeout(refresh, document.hidden ? 6000 : 2000);
        }
        void refresh();
        return () => { disposed = true; clearTimeout(timer); };
    });
    async function run(action: () => Promise<unknown>) {
        if (busy) return;
        busy = true; error = '';
        try { await action(); const next = await tunnelStatus(); if (!disposed) { status = next; hadSession = !!next; lost = false; } }
        catch (failure) { if (!disposed) error = String(failure); }
        finally { if (!disposed) busy = false; }
    }
    async function copy() {
        const value = status?.invitation || status?.roomCode;
        if (!value) return;
        try { await navigator.clipboard.writeText(value); toast('Convite copiado. Seu amigo cola em Entrar na sala.', 'success'); }
        catch (failure) { error = String(failure); }
    }
</script>

<section class="room-panel space-y-5">
    <header class="product-header relative overflow-hidden rounded-3xl border border-brand-400/20 bg-bg-elevated luxmc-card p-6 sm:p-8">
        <div class="pointer-events-none absolute -right-12 -top-16 h-56 w-56 rounded-full bg-brand-500/10 blur-3xl"></div>
        <div class="relative flex flex-wrap items-start justify-between gap-4">
            <div><p class="page-eyebrow flex items-center gap-2"><Users class="h-4 w-4" />Jogar juntos</p><h1 class="mt-3 text-3xl font-semibold tracking-tight text-fg">Sua turma. Seu mundo.</h1><p class="mt-3 max-w-xl text-sm leading-6 text-fg-muted">Crie uma sala, convide seus amigos e encontre o mundo na lista LAN do Minecraft.</p></div>
            <span class="inline-flex items-center gap-2 rounded-full border border-success/20 bg-success/10 px-3 py-2 text-xs text-success"><ShieldCheck class="h-4 w-4" />Conexão criptografada</span>
        </div>
    </header>
    {#if error}<p role="alert" in:fade class="rounded-2xl border border-danger/25 bg-danger/10 p-4 text-sm text-danger">{error}</p>{/if}
    {#if !ready || (busy && !status)}
        <div role="status" class="flex items-center gap-4 rounded-3xl border border-border bg-bg-elevated luxmc-card p-8"><LoaderCircle class="h-7 w-7 animate-spin text-brand-400" /><div><p class="font-semibold text-fg">{busy ? 'Conectando sua sala…' : 'Conferindo sua conexão…'}</p><p class="mt-1 text-sm text-fg-muted">O Luxmc prepara a conexão automaticamente.</p></div></div>
    {:else if !status}
        {#if lost}<div role="status" class="rounded-2xl border border-warning/30 bg-warning/10 p-4 text-sm text-fg">A conexão foi interrompida. {#if canReconnectTunnel()}<button class={button({variant:'secondary',size:'sm',class:'ml-2'})} onclick={() => run(reconnectTunnel)}>Reconectar</button>{/if}</div>{/if}
        <div class="grid gap-5 md:grid-cols-2" in:fly={{y:14,duration}}>
            <article class="space-y-5 rounded-3xl border border-brand-400/25 bg-bg-elevated luxmc-card p-6"><div class="flex h-12 w-12 items-center justify-center rounded-2xl bg-brand-500/10 text-brand-400"><Plus class="h-6 w-6" /></div><div><h2 class="text-xl font-semibold text-fg">Criar sala</h2><p class="mt-2 text-sm leading-6 text-fg-muted">Um convite para reunir até 10 pessoas. Qualquer participante pode abrir seu mundo.</p></div><button class={button({variant:'primary',class:'w-full'})} onclick={() => run(() => hostWorld())}><Plus class="h-4 w-4" />Criar minha sala</button></article>
            <article class="space-y-5 rounded-3xl border border-border bg-bg-elevated luxmc-card p-6"><div class="flex h-12 w-12 items-center justify-center rounded-2xl bg-fg/5 text-fg"><ArrowRight class="h-6 w-6" /></div><div><h2 class="text-xl font-semibold text-fg">Entrar na sala</h2><p class="mt-2 text-sm leading-6 text-fg-muted">Cole o convite que seu amigo enviou. Os mundos aparecem no jogo automaticamente.</p></div><label class="block"><span class="sr-only">Convite da sala</span><input bind:value={invitation} placeholder="Cole o convite aqui" class="w-full rounded-xl border border-border bg-bg px-4 py-3 text-sm text-fg outline-none focus:border-brand-400" /></label><button class={button({variant:'primary',class:'w-full'})} disabled={!invitation.trim()} onclick={() => run(() => joinTunnel(invitation.trim()))}>Entrar na sala<ArrowRight class="h-4 w-4" /></button></article>
        </div>
    {:else}
        <div class="grid items-start gap-5 lg:grid-cols-[minmax(0,1fr)_20rem]" in:fly={{y:14,duration}}>
            <div class="space-y-5">
                <section class="overflow-hidden rounded-3xl border border-border bg-bg-elevated luxmc-card">
                    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-border p-5"><div><h2 class="font-semibold text-fg">Sua sala está conectada</h2><p class="mt-1 flex items-center gap-2 text-xs text-fg-muted"><span class="h-2 w-2 rounded-full bg-success"></span>{status.members.length} de {status.maxPlayers} participantes{#if status.pingMs !== null} · {status.pingMs} ms{/if}</p></div><button class={button({variant:'secondary',size:'sm'})} onclick={() => run(stopSession)}><LogOut class="h-4 w-4" />Sair</button></div>
                    <div class="grid gap-3 p-5 sm:grid-cols-2">{#each status.members as member (member.id)}{@const person = visible(member)}<article class="flex items-center gap-3 rounded-2xl border border-border bg-bg/35 p-4" in:fly={{y:8,duration:duration ? 200 : 0}}><RoomAvatar member={person} /><div class="min-w-0 flex-1"><p class="truncate font-semibold text-fg">{person.displayName || person.username}</p><AccountBadges role={person.role} tags={person.tags} /><p class="mt-1 text-xs text-fg-muted">{member.isHost ? 'Criador da sala' : 'Conectado'}</p></div>{#if status.mode === 'host' && !member.isHost}<button aria-label={`Remover ${person.username}`} class={button({variant:'ghost',size:'sm'})} onclick={() => run(() => kickTunnelMember(member.id))}><X class="h-4 w-4" /></button>{/if}</article>{/each}</div>
                    {#if status.members.length < 2}<p class="px-5 pb-5 text-sm text-fg-muted">Convide seu primeiro amigo para começar.</p>{/if}
                </section>
                <section class="rounded-3xl border border-border bg-bg-elevated luxmc-card p-6"><div class="mb-4 flex items-center gap-3"><Gamepad2 class="h-5 w-5 text-brand-400" /><h2 class="text-lg font-semibold text-fg">Mundos da turma</h2></div>{#if worlds.length}{#each worlds as world (world.ownerId)}<div class="mt-3 flex items-center gap-3 rounded-2xl border border-success/20 bg-success/5 p-4"><Wifi class="h-5 w-5 text-success" /><div><p class="font-semibold text-fg">{world.ownerUsername}</p><p class="mt-1 text-sm text-fg-muted">{world.motd} · {world.compatibility?.mcVersion || 'Minecraft Java'}</p></div><span class="ml-auto rounded-full bg-success/10 px-3 py-1 text-xs text-success">LAN disponível</span></div>{/each}<p class="mt-4 text-sm leading-6 text-fg-muted">No Minecraft, abra <strong class="text-fg">Multijogador</strong> e aguarde o mundo aparecer em <strong class="text-fg">Jogos na rede local</strong>.</p>{:else}<div class="rounded-2xl border border-dashed border-border p-6 text-center"><Wifi class="mx-auto mb-3 h-8 w-8 text-brand-400" /><p class="font-medium text-fg">Aguardando um mundo</p><p class="mx-auto mt-2 max-w-lg text-sm leading-6 text-fg-muted">Quem vai hospedar abre seu mundo no Minecraft e escolhe <strong class="text-fg">Abrir para LAN</strong>. O Luxmc encontra o mundo automaticamente.</p></div>{/if}</section>
            </div>
            <aside class="space-y-5"><section class="space-y-4 rounded-3xl border border-border bg-bg-elevated luxmc-card p-5"><h2 class="font-semibold text-fg">Convide sua turma</h2><p class="text-sm leading-6 text-fg-muted">Seu amigo só precisa colar o convite em Entrar na sala.</p>{#if status.invitation || status.roomCode}<button class={button({variant:'primary',class:'w-full'})} onclick={copy}><Copy class="h-4 w-4" />Copiar convite</button>{/if}{#if status.mode === 'host'}<button class={button({variant:'ghost',size:'sm',class:'w-full'})} disabled={busy} onclick={() => run(refreshTunnelInvitation)}><RefreshCw class="h-4 w-4" />Renovar convite</button><p class="text-xs text-fg-muted">O convite vale por 24 horas. Renovar mantém os participantes conectados.</p><button class={button({variant:'secondary',class:'w-full'})} onclick={() => run(() => setTunnelLocked(!status!.roomLocked))}>{#if status.roomLocked}<Unlock class="h-4 w-4" />Permitir entradas{:else}<Lock class="h-4 w-4" />Fechar novas entradas{/if}</button>{/if}</section><section class="rounded-3xl border border-border bg-bg-elevated luxmc-card p-5"><h2 class="font-semibold text-fg">Prontos em 3 passos</h2><ol class="mt-4 space-y-4 text-sm leading-6 text-fg-muted"><li><strong class="text-brand-400">1.</strong> Todos entram nesta sala.</li><li><strong class="text-brand-400">2.</strong> Quem hospeda abre o mundo para LAN.</li><li><strong class="text-brand-400">3.</strong> Os amigos escolhem o mundo na lista LAN.</li></ol><p class="mt-4 border-t border-border pt-4 text-xs leading-5 text-fg-muted">Este modo usa Minecraft Java. Usem a mesma versão do Minecraft e os mesmos mods. Mantenha o Luxmc aberto em segundo plano durante a partida.</p></section></aside>
        </div>
    {/if}
</section>
