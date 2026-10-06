<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { Copy, Link2, LogOut, Network, Play, QrCode, RadioTower } from "lucide-svelte";
    import { hostWorld, joinTunnel, stopSession, tunnelStatus, type TunnelStatus } from "$lib/api/tunnel";
    import { toast } from "$lib/stores/toasts.svelte";

    let { profileId: _profileId }: { profileId?: string } = $props();
    let status = $state<TunnelStatus | null>(null);
    let actionError = $state("");
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
        actionError = "";
        try {
            await action();
        } catch (error) {
            actionError = String(error);
            toast(actionError, "error");
        } finally {
            busy = false;
        }
    }

    async function host() {
        status = await hostWorld();
        await goto("/hosting");
    }

    function invitationValue(value: string): string {
        const trimmed = value.trim();
        if (!trimmed.toLowerCase().startsWith("luxmc://")) return trimmed;
        const parsed = new URL(trimmed);
        return parsed.searchParams.get("invitation") || "";
    }

    async function join() {
        if (!status) status = await joinTunnel(invitationValue(invitation));
        if (!status.localAddress) throw new Error(uiText("ui.be429bcb3db8683b"));
        await goto("/hosting");
        toast(uiText("ui.db7636da2b6357d8"), "success");
    }

    async function copyInvite() {
        if (!inviteLink) return;
        await navigator.clipboard.writeText(inviteLink);
        toast(`Link da sala ${status?.roomCode || "P2P"} copiado.`, "success");
    }

    async function closeSession() {
        await stopSession();
        status = null;
        showQrCode = false;
    }
</script>

<div class="grid items-start gap-5 xl:grid-cols-[1.15fr_0.85fr]" aria-label={uiText("ui.093b9cfdd3e22b82")}>
    <section class="surface-glass relative overflow-hidden border-success/20 p-6 hover:border-success/35">
        <div class="pointer-events-none absolute inset-0 bg-gradient-to-br from-success/10 via-transparent to-brand-500/5"></div>
        <div class="relative flex flex-col gap-5">
            <div class="flex items-start justify-between gap-4">
                <div>
                    <div class="mb-3 inline-flex items-center gap-2 rounded-full border border-success/25 bg-success/10 px-3 py-1 text-[10px] font-black uppercase tracking-widest text-success"><RadioTower class="h-3.5 w-3.5" /> {uiText("ui.4457e711663bca74")}</div>
                    <h2 class="text-xl font-black text-fg">{uiText("ui.adcb51a850322231")}</h2>
                    <p class="mt-2 max-w-xl text-sm leading-relaxed text-fg/70">{uiText("multiplayer.createAnytime")}</p>
                </div>
                <div class="grid h-12 w-12 shrink-0 place-items-center rounded-2xl border border-success/25 bg-success/10 text-success shadow-glow"><Network class="h-6 w-6" /></div>
            </div>
            <ol class="grid gap-2 text-xs text-fg-muted sm:grid-cols-3" aria-label={uiText("ui.a0996d4eca5aa651")}>
                <li class="rounded-xl border border-fg/10 bg-bg/45 p-3"><strong class="mb-1 block text-fg">{uiText("ui.36674be5215f5f24")}</strong>{uiText("ui.452bb4b1e70adb7a")}</li>
                <li class="rounded-xl border border-fg/10 bg-bg/45 p-3"><strong class="mb-1 block text-fg">{uiText("ui.2e2d6fc6d51333cf")}</strong>{uiText("ui.ddc3f5afc4101f2d")}</li>
                <li class="rounded-xl border border-fg/10 bg-bg/45 p-3"><strong class="mb-1 block text-fg">{uiText("ui.1bb8197e9a465bb3")}</strong>{uiText("ui.4d312dab820e7496")}</li>
            </ol>
            {#if actionError}<p class="rounded-xl border border-danger/30 bg-danger/10 p-3 text-xs text-danger" role="alert">{actionError}</p>{/if}

            {#if status?.mode === "host"}
                <button type="button" class={launcherButton({ variant: "primary", size: "lg" })} onclick={() => goto("/hosting")}>{uiText("ui.531c8fca7703588a")}</button>
                <div class="rounded-2xl border border-success/25 bg-bg/60 p-5 text-center">
                    <p class="text-[10px] font-bold uppercase tracking-[0.24em] text-fg-subtle">{uiText("ui.c4ad21c272ed73fd")}</p>
                    <p class="mt-2 font-mono text-4xl font-black tracking-wider text-success">{status.roomCode}</p>
                    <p class="mt-2 text-xs text-fg-muted">{uiText("ui.98ffdf0881e44b01")} {status.transport}</p>
                </div>
                <div class="grid gap-3 sm:grid-cols-2">
                    <button type="button" class={launcherButton({ variant: "primary", size: "lg", class: "inline-flex items-center justify-center gap-2" })} disabled={busy} onclick={() => perform(copyInvite)}><Link2 class="h-4 w-4" /> {uiText("ui.82276f48efc21b37")}</button>
                    <button type="button" class={launcherButton({ variant: "ghost", size: "lg", class: "luxmc-control inline-flex items-center justify-center gap-2" })} disabled={busy} onclick={() => showQrCode = !showQrCode}><QrCode class="h-4 w-4" /> {showQrCode ? uiText("ui.25e9a918675270e2") : "Gerar QR Code"}</button>
                </div>
                {#if showQrCode && roomQrCode}
                    <div class="flex flex-col items-center gap-4 rounded-2xl border border-fg/10 bg-bg/55 p-4 sm:flex-row">
                        <img loading="lazy" decoding="async" class="h-28 w-28 rounded-xl bg-fg p-1" src={roomQrCode} alt={uiText("ui.97e01819f6f87736")} />
                        <div><p class="text-sm font-bold text-fg">{uiText("ui.f6a28d1298d2f279")}</p><p class="mt-1 text-xs leading-relaxed text-fg-muted">{uiText("ui.d3f28df7257a9ea2")}</p></div>
                    </div>
                {/if}
                <button type="button" class={launcherButton({ variant: "danger", size: "sm", class: "inline-flex items-center justify-center gap-2" })} disabled={busy} onclick={() => perform(closeSession)}><LogOut class="h-4 w-4" />{uiText("ui.71b96dcef1a240f4")}</button>
            {:else}
                <p class="rounded-2xl border border-success/20 bg-success/5 p-4 text-sm leading-relaxed text-fg-muted">{uiText("multiplayer.autoLan")}</p>
                <button type="button" class={launcherButton({ variant: "primary", size: "lg", class: "mt-2 inline-flex items-center justify-center gap-2 disabled:opacity-50" })} disabled={busy || !!status} onclick={() => perform(host)}><RadioTower class="h-4 w-4" /> {busy ? uiText("ui.8072a1679aaf96f2") : uiText("ui.a57d1b367f1a3034")}</button>
            {/if}
        </div>
    </section>

    <section class="surface-glass relative overflow-hidden p-6 hover:border-brand-500/30">
        <div class="pointer-events-none absolute inset-0 bg-gradient-to-br from-brand-500/10 via-transparent to-info/5"></div>
        <div class="relative flex flex-col gap-5">
            <div>
                <div class="mb-3 inline-flex items-center gap-2 rounded-full border border-brand-500/25 bg-brand-500/10 px-3 py-1 text-[10px] font-black uppercase tracking-widest text-brand-300"><Play class="h-3.5 w-3.5" /> {uiText("ui.09734f8580f8a1ac")}</div>
                <h2 class="text-xl font-black text-fg">{uiText("ui.c280780b6f1025f3")}</h2>
                <p class="mt-2 text-sm leading-relaxed text-fg/70">{uiText("ui.86ece3da96bcde03")}</p>
            </div>
            {#if status?.mode === "client"}
                <div class="rounded-2xl border border-success/25 bg-success/10 p-4" role="status">
                    <p class="font-bold text-success">{uiText("ui.c0c59016932fc3b8")}</p>
                    <p class="mt-1 text-xs text-fg-muted">{uiText("ui.8e3579ff9bbff33c")}</p>
                    <p class="mt-3 select-text font-mono text-sm text-success">{status.localAddress}</p><button type="button" class={launcherButton({ variant: "primary", size: "sm" })} onclick={() => goto("/hosting")}>{uiText("ui.31b34fd59207a45d")}</button><p class="mt-3 font-mono text-xs text-success">{status.transport}{status.pingMs !== null ? ` · ${status.pingMs} ms` : uiText("ui.e7955a952babc778")}</p>
                </div>
                <button type="button" class={launcherButton({ variant: "ghost", size: "lg", class: "luxmc-control mt-auto inline-flex items-center justify-center gap-2" })} disabled={busy} onclick={() => perform(closeSession)}><LogOut class="h-4 w-4" />{uiText("ui.30243d5d7e645abb")}</button>
            {:else}
                <label class="text-xs font-bold text-fg-muted">{uiText("ui.fd777080c5cbd2dc")}<textarea class="mt-2 min-h-28 w-full resize-none rounded-xl border border-border bg-bg/60 px-4 py-3 font-mono text-xs text-fg outline-none placeholder:text-fg-subtle focus:border-brand-500/50" bind:value={invitation} placeholder={uiText("ui.a11d8206931fce10")} maxlength="4096"></textarea></label>
                <button type="button" class={launcherButton({ variant: "primary", size: "lg", class: "mt-2 inline-flex items-center justify-center gap-2 disabled:opacity-50" })} disabled={busy || status?.mode === "host" || !invitation.trim()} onclick={() => perform(join)}><Play class="h-4 w-4" /> {busy ? uiText("ui.cc190ab9f5ec058f") : uiText("ui.44f68496b27a46b7")}</button>
                <button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "inline-flex items-center justify-center gap-2" })} onclick={() => navigator.clipboard.readText().then(value => invitation = value)}><Copy class="h-3.5 w-3.5" />{uiText("ui.1bc650e965fe107a")}</button>
            {/if}
        </div>
    </section>
</div>
