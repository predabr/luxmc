<script lang="ts">
    import { onMount } from "svelte";
    import QRCode from "qrcode";
    import { hostWorld, joinTunnel, stopSession, tunnelStatus, type TunnelStatus } from "$lib/api/tunnel";
    import { p2pScanLanWorlds, p2pGetLocalInfo } from "$lib/api/p2p";
    let { profileId: _profileId }: { profileId?: string } = $props();
    import { toast } from "$lib/stores/toasts.svelte";
    let status = $state<TunnelStatus | null>(null);
    let port = $state<number | undefined>();
    let choosePort = $state(false);
    let invitation = $state("");
    let busy = $state(false);
    let roomQrCode = $state<string | null>(null);
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
    $effect(() => {
        const code = status?.invitation;
        if (!code) {
            roomQrCode = null;
            return;
        }
        let active = true;
        const link = `luxmc://join/world?${new URLSearchParams({ invitation: code })}`;
        void QRCode.toDataURL(link, { errorCorrectionLevel: "M", margin: 1, width: 256 })
            .then(value => { if (active) roomQrCode = value; })
            .catch(() => { if (active) roomQrCode = null; });
        return () => { active = false; };
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
        toast("Conectado à rede do amigo. Abra o Minecraft em Multijogador para encontrar o mundo na Rede Local.", "success");
    }
</script>

<div class="grid gap-5 lg:grid-cols-2" aria-label="Jogar com amigos">
    <section class="luxmc-glass rounded-2xl p-6 flex flex-col gap-5">
        <div><h2 class="text-lg font-semibold text-fg">Hospedar meu mundo</h2><p class="mt-2 text-sm text-fg-muted">Abra seu mundo para LAN no Minecraft. O Luxmc encontra a porta e cria um convite privado.</p></div>
        {#if status?.mode === "host"}
            <label class="text-sm text-fg-muted">Código da sala · válido por uma hora<textarea readonly class="mt-2 w-full rounded-xl border-border bg-bg/30 text-fg text-xs" value={status.invitation || ""} rows="3"></textarea></label>
            <p class="rounded-xl border border-emerald-500/25 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-300">Sala {status.roomCode} · copie o código completo para que o túnel P2P seja configurado.</p>
            <button class="luxmc-control" disabled={busy} onclick={() => perform(async () => { await navigator.clipboard.writeText(status!.invitation!); toast(`Código da sala ${status!.roomCode} copiado.`, "success"); })}>Copiar código da sala</button>
            {#if roomQrCode}
                <div class="flex items-center gap-3 rounded-xl border border-fg/10 bg-bg/30 p-3">
                    <img class="h-20 w-20 rounded-lg bg-fg p-1" src={roomQrCode} alt="QR Code do convite da sala" />
                    <p class="text-xs text-fg-muted">Escaneie no outro dispositivo para abrir o Luxmc e entrar nesta sala.</p>
                </div>
            {/if}
            <div class="rounded-xl border border-emerald-500/25 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-300">Rede ativa · {status.transport}</div>
            <button class="luxmc-control" disabled={busy} onclick={() => perform(async () => { await stopSession(); status = null; port = undefined; })}>Encerrar partida</button>
        {:else}
            {#if choosePort}<label class="text-sm text-fg-muted">Porta exibida no Minecraft<input class="mt-2 w-full rounded-xl border-border bg-bg/30 text-fg" type="number" min="1" max="65535" bind:value={port} /></label>{/if}
            <button class="luxmc-control mt-auto" disabled={busy || !!status} onclick={() => perform(host)}>{busy ? "Aguarde…" : "Abrir para amigos"}</button>
        {/if}
    </section>
    <section class="luxmc-glass rounded-2xl p-6 flex flex-col gap-5">
        <div><h2 class="text-lg font-semibold text-fg">Entrar no mundo de um amigo</h2><p class="mt-2 text-sm text-fg-muted">Use o convite para entrar na rede privada do anfitrião.</p></div>
        {#if status?.mode === "client"}
            <div class="rounded-xl border border-emerald-500/25 bg-emerald-500/10 p-3 text-sm text-emerald-300" role="status">
                <p class="font-bold">Conectado à rede do amigo!</p>
                <p class="mt-1 text-xs text-emerald-200/80">Abra o Minecraft → Multijogador e o mundo aparecerá automaticamente na lista de Rede Local.</p>
                <p class="mt-2 text-xs font-mono">{status.transport}{status.pingMs !== null ? ` · ${status.pingMs} ms` : " · medindo ping…"}</p>
            </div>
        {:else}
            <label class="text-sm text-fg-muted">Código de convite<textarea class="mt-2 w-full rounded-xl border-border bg-bg/30 text-fg text-xs" bind:value={invitation} placeholder="LUX-4821|luxmc-world:…" maxlength="4096" rows="3"></textarea></label>
        {/if}
        <button class="luxmc-control mt-auto" disabled={busy || status?.mode === "host" || status?.mode === "client" || (!status && !invitation.trim())} onclick={() => perform(join)}>{busy ? "Conectando…" : "Entrar na rede"}</button>
        {#if status?.mode === "client"}<button class="luxmc-control" disabled={busy} onclick={() => perform(async () => { await stopSession(); status = null; })}>Sair da conexão</button>{/if}
    </section>
</div>
