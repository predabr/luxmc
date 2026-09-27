<script lang="ts">
    import { onMount } from "svelte";
    import { hostWorld, joinTunnel, stopSession, tunnelStatus, type TunnelStatus } from "$lib/api/tunnel";
    import { p2pScanLanWorlds, p2pGetLocalInfo } from "$lib/api/p2p";
    import { joinWorld } from "$lib/utils/directJoin";
    import { toast } from "$lib/stores/toasts.svelte";
    let { profileId }: { profileId?: string } = $props();
    let status = $state<TunnelStatus | null>(null);
    let port = $state<number | undefined>();
    let choosePort = $state(false);
    let invitation = $state("");
    let busy = $state(false);
    let disposed = false;
    onMount(() => {
        let timer: ReturnType<typeof setTimeout>;
        const refresh = async () => {
            try { const next = await tunnelStatus(); if (!disposed && !busy) status = next; } catch {}
            if (!disposed) timer = setTimeout(refresh, 3000);
        };
        void refresh();
        return () => { disposed = true; clearTimeout(timer); };
    });
    async function perform(action: () => Promise<void>) {
        if (busy) return;
        busy = true;
        try { await action(); } catch (error) { toast(String(error), "error"); }
        finally { busy = false; }
    }
    async function host() {
        if (!port) {
            const [discovered, local] = await Promise.all([p2pScanLanWorlds(), p2pGetLocalInfo()]);
            const worlds = discovered.filter(world => world.host === local.ip || world.host === "127.0.0.1" || world.host === "::1");
            if (worlds.length === 1) port = worlds[0].port;
            else { choosePort = true; toast(worlds.length ? "Informe a porta do mundo que deseja compartilhar." : "Abra o Minecraft para LAN e informe a porta exibida no jogo.", "info"); return; }
        }
        if (!Number.isInteger(port) || !port || port < 1 || port > 65535) throw new Error("Porta LAN inválida");
        status = await hostWorld(port);
    }
    async function join() {
        if (!status) status = await joinTunnel(invitation.trim());
        if (!status.localAddress) throw new Error("Encerre a hospedagem antes de entrar em outro mundo.");
        await joinWorld(status.localAddress, undefined, profileId);
    }
</script>

<div class="grid gap-5 lg:grid-cols-2" aria-label="Jogar com amigos">
    <section class="luxmc-glass rounded-2xl p-6 flex flex-col gap-5">
        <div><h2 class="text-lg font-semibold text-fg">Hospedar meu mundo</h2><p class="mt-2 text-sm text-fg-muted">Abra seu mundo para LAN no Minecraft. O Luxmc cuida da conexão.</p></div>
        {#if status?.mode === "host"}
            <label class="text-sm text-fg-muted">Convite privado · válido por uma hora<textarea readonly class="mt-2 w-full rounded-xl border-border bg-bg/30 text-fg text-xs" value={status.invitation || ""} rows="3"></textarea></label>
            <button class="luxmc-control" disabled={busy} onclick={() => perform(async () => { await navigator.clipboard.writeText(status!.invitation!); toast("Código copiado. Compartilhe apenas com seus amigos.", "success"); })}>Copiar código</button>
            <button class="luxmc-control" disabled={busy} onclick={() => perform(async () => { await stopSession(); status = null; port = undefined; })}>Encerrar partida</button>
        {:else}
            {#if choosePort}<label class="text-sm text-fg-muted">Porta exibida no Minecraft<input class="mt-2 w-full rounded-xl border-border bg-bg/30 text-fg" type="number" min="1" max="65535" bind:value={port} /></label>{/if}
            <button class="luxmc-control mt-auto" disabled={busy || !!status} onclick={() => perform(host)}>{busy ? "Aguarde…" : "Abrir para amigos"}</button>
        {/if}
    </section>
    <section class="luxmc-glass rounded-2xl p-6 flex flex-col gap-5">
        <div><h2 class="text-lg font-semibold text-fg">Entrar no mundo de um amigo</h2><p class="mt-2 text-sm text-fg-muted">Use o convite do anfitrião para iniciar sua instância ativa.</p></div>
        {#if status?.mode === "client"}<p class="text-sm text-success" role="status">Conectado ao mundo do seu amigo.</p>{:else}
            <label class="text-sm text-fg-muted">Código de convite<textarea class="mt-2 w-full rounded-xl border-border bg-bg/30 text-fg text-xs" bind:value={invitation} placeholder="Cole o código de convite aqui…" maxlength="4096" rows="3"></textarea></label>
        {/if}
        <button class="luxmc-control mt-auto" disabled={busy || status?.mode === "host" || (!status && !invitation.trim())} onclick={() => perform(join)}>{busy ? "Conectando…" : "Entrar no mundo"}</button>
        {#if status?.mode === "client"}<button class="luxmc-control" disabled={busy} onclick={() => perform(async () => { await stopSession(); status = null; })}>Sair da conexão</button>{/if}
    </section>
</div>
