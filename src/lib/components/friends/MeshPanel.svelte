<script lang="ts">
    import { onMount } from "svelte";
    import { Copy, Link2, LogOut, Network, Play, QrCode, RadioTower } from "lucide-svelte";
    import { hostWorld, joinTunnel, stopSession, tunnelStatus, type TunnelStatus } from "$lib/api/tunnel";
    import { p2pScanLanWorlds, p2pGetLocalInfo } from "$lib/api/p2p";
    import { toast } from "$lib/stores/toasts.svelte";

    let { profileId: _profileId }: { profileId?: string } = $props();
    let status = $state<TunnelStatus | null>(null);
    let port = $state<number | undefined>();
    let choosePort = $state(false);
    let invitation = $state("");
    let busy = $state(false);
    let roomQrCode = $state<string | null>(null);
    let showQrCode = $state(false);
    let disposed = false;

    const inviteLink = $derived(status?.invitation ? `luxmc://join/world?${new URLSearchParams({ invitation: status.invitation })}` : "");

    onMount(() => {
        let timer: ReturnType<typeof setTimeout>;
        const refresh = async () => {
            try {
                const next = await tunnelStatus();
                if (!disposed && !busy) status = next;
            } catch {}
            if (!disposed) timer = setTimeout(refresh, 3000);
        };
        void refresh();
        return () => {
            disposed = true;
            clearTimeout(timer);
        };
    });

    $effect(() => {
        const link = inviteLink;
        if (!link || !showQrCode) {
            roomQrCode = null;
            return;
        }
        let active = true;
        void import("qrcode")
            .then(({ toDataURL }) => toDataURL(link, { errorCorrectionLevel: "M", margin: 1, width: 256 }))
            .then(value => { if (active) roomQrCode = value; })
            .catch(() => { if (active) roomQrCode = null; });
        return () => { active = false; };
    });

    async function perform(action: () => Promise<void>) {
        if (busy) return;
        busy = true;
        try {
            await action();
        } catch (error) {
            toast(String(error), "error");
        } finally {
            busy = false;
        }
    }

    async function host() {
        if (!port) {
            const [discovered, local] = await Promise.all([p2pScanLanWorlds(), p2pGetLocalInfo()]);
            const worlds = discovered.filter(world => world.host === local.ip || world.host === "127.0.0.1" || world.host === "::1");
            if (worlds.length === 1) {
                port = worlds[0].port;
            } else {
                choosePort = true;
                toast(worlds.length ? "Escolha a porta do mundo que deseja compartilhar." : "Abra o Minecraft para LAN e informe a porta exibida no jogo.", "info");
                return;
            }
        }
        if (!Number.isInteger(port) || !port || port < 1 || port > 65535) throw new Error("Porta LAN inválida");
        status = await hostWorld(port);
    }

    function invitationValue(value: string): string {
        const trimmed = value.trim();
        if (!trimmed.toLowerCase().startsWith("luxmc://")) return trimmed;
        const parsed = new URL(trimmed);
        return parsed.searchParams.get("invitation") || "";
    }

    async function join() {
        if (!status) status = await joinTunnel(invitationValue(invitation));
        if (!status.localAddress) throw new Error("Encerre a hospedagem antes de entrar em outro mundo.");
        toast("Rede P2P conectada. Abra o Minecraft em Multijogador para encontrar o mundo LAN.", "success");
    }

    async function copyInvite() {
        if (!inviteLink) return;
        await navigator.clipboard.writeText(inviteLink);
        toast(`Link da sala ${status?.roomCode || "P2P"} copiado.`, "success");
    }

    async function closeSession() {
        await stopSession();
        status = null;
        port = undefined;
        showQrCode = false;
    }
</script>

<div class="grid gap-5 xl:grid-cols-[1.15fr_0.85fr]" aria-label="Salas diretas P2P">
    <section class="surface-glass relative overflow-hidden border-success/20 p-6 hover:border-success/35">
        <div class="pointer-events-none absolute inset-0 bg-gradient-to-br from-success/10 via-transparent to-brand-500/5"></div>
        <div class="relative flex h-full flex-col gap-5">
            <div class="flex items-start justify-between gap-4">
                <div>
                    <div class="mb-3 inline-flex items-center gap-2 rounded-full border border-success/25 bg-success/10 px-3 py-1 text-[10px] font-black uppercase tracking-widest text-success"><RadioTower class="h-3.5 w-3.5" /> Sala direta P2P</div>
                    <h2 class="text-xl font-black text-fg">Seu mundo, um convite</h2>
                    <p class="mt-2 max-w-xl text-sm leading-relaxed text-fg-muted">Abra o mundo para LAN. O Luxmc cria um túnel privado e um link que funciona mesmo sem o serviço social.</p>
                </div>
                <div class="grid h-12 w-12 shrink-0 place-items-center rounded-2xl border border-success/25 bg-success/10 text-success shadow-glow"><Network class="h-6 w-6" /></div>
            </div>

            {#if status?.mode === "host"}
                <div class="rounded-2xl border border-success/25 bg-bg/60 p-5 text-center">
                    <p class="text-[10px] font-bold uppercase tracking-[0.24em] text-fg-subtle">Código rápido da sala</p>
                    <p class="mt-2 font-mono text-4xl font-black tracking-wider text-success">{status.roomCode}</p>
                    <p class="mt-2 text-xs text-fg-muted">Válido por uma hora · {status.transport}</p>
                </div>
                <div class="grid gap-3 sm:grid-cols-2">
                    <button type="button" class="inline-flex items-center justify-center gap-2 rounded-xl bg-brand-500 px-4 py-3 text-sm font-black text-brand-foreground shadow-button hover:bg-brand-400" disabled={busy} onclick={() => perform(copyInvite)}><Link2 class="h-4 w-4" /> Copiar link de convite</button>
                    <button type="button" class="luxmc-control inline-flex items-center justify-center gap-2 py-3" disabled={busy} onclick={() => showQrCode = !showQrCode}><QrCode class="h-4 w-4" /> {showQrCode ? "Ocultar QR Code" : "Gerar QR Code"}</button>
                </div>
                {#if showQrCode && roomQrCode}
                    <div class="flex flex-col items-center gap-4 rounded-2xl border border-fg/10 bg-bg/55 p-4 sm:flex-row">
                        <img class="h-28 w-28 rounded-xl bg-fg p-1" src={roomQrCode} alt="QR Code do convite da sala" />
                        <div><p class="text-sm font-bold text-fg">Convite pronto para escanear</p><p class="mt-1 text-xs leading-relaxed text-fg-muted">O QR abre o Luxmc no outro dispositivo com os dados completos e privados da sala.</p></div>
                    </div>
                {/if}
                <button type="button" class="inline-flex items-center justify-center gap-2 text-xs font-bold text-danger hover:text-danger/80" disabled={busy} onclick={() => perform(closeSession)}><LogOut class="h-4 w-4" />Encerrar sala</button>
            {:else}
                {#if choosePort}<label class="text-xs font-bold text-fg-muted">Porta exibida pelo Minecraft<input class="mt-2 w-full rounded-xl border border-border bg-bg/60 px-4 py-3 text-fg outline-none focus:border-success/50" type="number" min="1" max="65535" bind:value={port} /></label>{/if}
                <button type="button" class="mt-auto inline-flex items-center justify-center gap-2 rounded-xl bg-brand-500 px-5 py-3.5 text-sm font-black text-brand-foreground shadow-button hover:bg-brand-400 disabled:opacity-50" disabled={busy || !!status} onclick={() => perform(host)}><RadioTower class="h-4 w-4" /> {busy ? "Preparando sala…" : "Hospedar meu mundo"}</button>
            {/if}
        </div>
    </section>

    <section class="surface-glass relative overflow-hidden p-6 hover:border-brand-500/30">
        <div class="pointer-events-none absolute inset-0 bg-gradient-to-br from-brand-500/10 via-transparent to-info/5"></div>
        <div class="relative flex h-full flex-col gap-5">
            <div>
                <div class="mb-3 inline-flex items-center gap-2 rounded-full border border-brand-500/25 bg-brand-500/10 px-3 py-1 text-[10px] font-black uppercase tracking-widest text-brand-300"><Play class="h-3.5 w-3.5" /> Entrar em uma sala</div>
                <h2 class="text-xl font-black text-fg">Cole o convite do amigo</h2>
                <p class="mt-2 text-sm leading-relaxed text-fg-muted">Aceita o link luxmc:// ou o convite completo iniciado por LUX-XXXX.</p>
            </div>
            {#if status?.mode === "client"}
                <div class="rounded-2xl border border-success/25 bg-success/10 p-4" role="status">
                    <p class="font-bold text-success">Conectado à rede do amigo</p>
                    <p class="mt-1 text-xs text-fg-muted">Abra Minecraft → Multijogador. O mundo aparecerá como uma partida LAN.</p>
                    <p class="mt-3 font-mono text-xs text-success">{status.transport}{status.pingMs !== null ? ` · ${status.pingMs} ms` : " · conexão ativa"}</p>
                </div>
                <button type="button" class="luxmc-control mt-auto inline-flex items-center justify-center gap-2 py-3" disabled={busy} onclick={() => perform(closeSession)}><LogOut class="h-4 w-4" />Sair da conexão</button>
            {:else}
                <label class="text-xs font-bold text-fg-muted">Link ou convite completo<textarea class="mt-2 min-h-28 w-full resize-none rounded-xl border border-border bg-bg/60 px-4 py-3 font-mono text-xs text-fg outline-none placeholder:text-fg-subtle focus:border-brand-500/50" bind:value={invitation} placeholder="LUX-4821|luxmc-world:…" maxlength="4096"></textarea></label>
                <button type="button" class="mt-auto inline-flex items-center justify-center gap-2 rounded-xl bg-brand-500 px-5 py-3.5 text-sm font-black text-brand-foreground shadow-button hover:bg-brand-400 disabled:opacity-50" disabled={busy || status?.mode === "host" || !invitation.trim()} onclick={() => perform(join)}><Play class="h-4 w-4" /> {busy ? "Conectando…" : "Entrar no mundo"}</button>
                <button type="button" class="inline-flex items-center justify-center gap-2 text-xs font-bold text-fg-muted hover:text-fg" onclick={() => navigator.clipboard.readText().then(value => invitation = value)}><Copy class="h-3.5 w-3.5" />Colar da área de transferência</button>
            {/if}
        </div>
    </section>
</div>
