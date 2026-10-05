<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { ArrowLeft, Copy, Lock, Unlock, RadioTower, Users, LogOut, UserMinus, RefreshCw, ShieldCheck } from "lucide-svelte";
    import { button } from "$lib/components/ui/button";
    import Modal from "$lib/components/ui/Modal.svelte";
    import RoomAvatar from "$lib/components/friends/RoomAvatar.svelte";
    import GuestRoom from "$lib/components/friends/GuestRoom.svelte";
    import MeshPanel from "$lib/components/friends/MeshPanel.svelte";
    import { tunnelStatus, stopSession, kickTunnelMember, setTunnelLocked, refreshTunnelInvitation, type TunnelStatus, type TunnelMember } from "$lib/api/tunnel";
    import { toast } from "$lib/stores/toasts.svelte";

    let room = $state<TunnelStatus | null>(null);
    let loading = $state(true);
    let error = $state("");
    let busy = $state(false);
    let pendingKick = $state<TunnelMember | null>(null);
    let confirmStop = $state(false);
    let disposed = false;
    let request = 0;
    const members = $derived(room?.members || []);
    const capacity = $derived(room?.maxPlayers || 10);
    const invite = $derived(room?.invitation || "");

    async function refresh() {
        const current = ++request;
        try {
            const next = await tunnelStatus();
            if (!disposed && current === request) { room = next; error = ""; }
        } catch (cause) { if (!disposed && current === request) error = String(cause); }
        finally { if (!disposed && current === request) loading = false; }
    }
    onMount(() => {
        let timer: ReturnType<typeof setTimeout>;
        const poll = async () => { await refresh(); if (!disposed) timer = setTimeout(poll, 1500); };
        void poll();
        return () => { disposed = true; request++; clearTimeout(timer); };
    });
    async function perform(action: () => Promise<void>) {
        if (busy) return;
        busy = true;
        try { await action(); await refresh(); }
        catch (cause) { error = String(cause); toast(error, "error"); }
        finally { busy = false; }
    }
    async function copy() {
        if (!invite) return;
        await navigator.clipboard.writeText(invite);
        toast(uiText("ui.c4af356d45d9a07c"), "success");
    }
    async function kick() {
        if (!pendingKick) return;
        await kickTunnelMember(pendingKick.id);
        pendingKick = null;
    }
    async function stop() { await stopSession(); confirmStop = false; await goto("/friends"); }
</script>

<svelte:head><title>{uiText("ui.85c55be42cd397fe")}</title></svelte:head>

<div class="mx-auto w-full max-w-6xl space-y-6 p-1 pb-8" aria-label={uiText("ui.89bacbf67d969125")}>
    <button type="button" class={button({ variant: "ghost", size: "sm" })} onclick={() => goto("/friends")}><ArrowLeft class="h-4 w-4" />{uiText("ui.075d1408f079c452")}</button>
    {#if error}<p role="alert" class="rounded-xl border border-danger/30 bg-danger/10 p-4 text-sm text-danger">{error}</p>{/if}
    {#if loading}
        <p role="status" class="py-10 text-fg-muted">{uiText("ui.a65824823aff1671")}</p>
    {:else if room?.mode === "host"}
        <header class="surface-glass space-y-5 p-6 sm:p-8">
            <div class="flex flex-wrap items-start justify-between gap-4">
                <div><span class="mb-3 inline-flex items-center gap-2 text-xs font-semibold text-success"><RadioTower class="h-4 w-4" />{uiText("ui.aeb99766f4f2f2dc")}</span><h1 class="text-3xl font-bold tracking-tight text-fg">{uiText("ui.16d4b8ac375c4ea5")}</h1><p class="mt-2 text-sm text-fg-muted">{uiText("ui.354a1eb5fd2680be")}</p></div>
                <div class="rounded-2xl border border-success/20 bg-success/10 px-5 py-4"><p class="flex items-center gap-2 text-xl font-bold text-success"><Users class="h-5 w-5" />{members.length} / {capacity}</p><p class="mt-1 text-[11px] text-fg-muted">{uiText("ui.2de83f2d4054df9b")}</p></div>
            </div>
            <div class="flex flex-wrap items-center gap-3 rounded-xl border border-border bg-bg/40 p-4">
                <div class="mr-auto"><p class="text-[10px] uppercase tracking-widest text-fg-subtle">{uiText("ui.4c11763887517997")}</p><p class="mt-1 font-mono text-2xl font-bold text-fg">{room.roomCode}</p></div>
                <button type="button" class={button({ variant: "primary", size: "lg" })} disabled={busy || !invite} onclick={() => perform(copy)}><Copy class="h-4 w-4" />{uiText("ui.808ccc2b022037ca")}</button>
                <button type="button" class={button({ variant: "ghost", size: "sm" })} disabled={busy} onclick={() => perform(async () => { room = await refreshTunnelInvitation(); toast(uiText("ui.7538754b30b65f6c"), "success"); })}><RefreshCw class="h-4 w-4" />{uiText("ui.1a7c52e5c5fda934")}</button>
                <button type="button" class={button({ variant: "secondary", size: "lg" })} disabled={busy} aria-pressed={room.roomLocked} onclick={() => perform(() => setTunnelLocked(!room!.roomLocked))}>{#if room.roomLocked}<Lock class="h-4 w-4" />{uiText("ui.489db7e8b247fde5")}{:else}<Unlock class="h-4 w-4" />{uiText("ui.84c3958edced8a1c")}{/if}</button>
            </div>
            <p class="flex items-center gap-2 text-xs text-fg-muted"><ShieldCheck class="h-4 w-4 text-success" />{room.roomLocked ? uiText("ui.6038bf94be5a0219") : uiText("ui.9db808252d6caa42")}</p>
        </header>
        <section class="surface-glass p-6" aria-label={uiText("ui.4b77e3efe74df84a")}>
            <div class="mb-5 flex items-center justify-between gap-3"><h2 class="text-lg font-semibold text-fg">{uiText("ui.aff4a93487ceb273")}</h2><button type="button" class={button({ variant: "ghost", size: "sm" })} disabled={busy} onclick={() => perform(refresh)}><RefreshCw class="h-4 w-4" />{uiText("news.refresh")}</button></div>
            <ul class="grid gap-3 sm:grid-cols-2">
                {#each members as member (member.id)}
                    <li class="flex items-center gap-4 rounded-2xl border border-border bg-bg/35 p-4" data-room-member={member.id}>
                        <RoomAvatar {member} />
                        <div class="min-w-0 flex-1"><p class="truncate font-semibold text-fg">{member.username}</p><p class="mt-1 text-xs text-fg-muted">{member.isHost ? uiText("ui.110732d2ccdd2086") : uiText("ui.24a3bd9986f30b86")}</p></div>
                        {#if !member.isHost}<button type="button" class={button({ variant: "danger", size: "icon" })} disabled={busy} aria-label={uiText("ui.8ef3a7fe8939e3bc", {arg0: (member.username)})} title={uiText("ui.8ef3a7fe8939e3bc", {arg0: (member.username)})} onclick={() => pendingKick = member}><UserMinus class="h-4 w-4" /></button>{/if}
                    </li>
                {/each}
            </ul>
            {#if members.length <= 1}<p class="mt-5 rounded-xl border border-dashed border-border p-5 text-center text-sm text-fg-muted">{uiText("ui.ce1248fef70289fe")}</p>{/if}
        </section>
        <footer class="flex flex-wrap items-center justify-between gap-3"><p class="text-xs text-fg-muted">{uiText("ui.502b69b918976a29")}</p><button type="button" class={button({ variant: "danger", size: "lg" })} disabled={busy} onclick={() => confirmStop = true}><LogOut class="h-4 w-4" />{uiText("ui.b6628e1195470983")}</button></footer>
    {:else if error && !room}
        <section class="surface-glass space-y-4 p-6"><h1 class="text-2xl font-bold text-fg">{uiText("ui.0899b054b6c2966b")}</h1><p class="text-sm text-fg-muted">{uiText("ui.f9a3ff7c7d14b7fd")}</p><button type="button" class={button({ variant: "primary" })} disabled={busy} onclick={() => perform(refresh)}>{uiText("ui.2ccb595b4413b502")}</button></section>
    {:else if room?.mode === "client"}
        <GuestRoom {room} />
    {:else}
        <div class="space-y-2"><h1 class="text-3xl font-bold text-fg">{uiText("ui.2c8630407fb05ef5")}</h1><p class="text-sm text-fg-muted">{uiText("ui.9d1a31f109aff575")}</p></div>
        <MeshPanel />
    {/if}
</div>

<Modal isOpen={pendingKick !== null} title={uiText("ui.837e0c964a33cb9c")} onClose={() => { if (!busy) pendingKick = null; }}>
    <p class="text-sm text-fg-muted">{uiText("ui.ffb1a9c946e708e2")} {pendingKick?.username}{uiText("ui.ee4de0cfe09d0b19")}</p>
    <div class="mt-6 flex justify-end gap-3"><button type="button" class={button({ variant: "secondary" })} disabled={busy} onclick={() => pendingKick = null}>{uiText("common.cancel")}</button><button type="button" class={button({ variant: "danger" })} disabled={busy} onclick={() => perform(kick)}>{uiText("ui.dd71f6a23c609050")}</button></div>
</Modal>
<Modal isOpen={confirmStop} title={uiText("ui.b6628e1195470983")} onClose={() => { if (!busy) confirmStop = false; }}>
    <p class="text-sm text-fg-muted">{uiText("ui.17a2331d75583d0f")}</p>
    <div class="mt-6 flex justify-end gap-3"><button type="button" class={button({ variant: "secondary" })} disabled={busy} onclick={() => confirmStop = false}>{uiText("common.cancel")}</button><button type="button" class={button({ variant: "danger" })} disabled={busy} onclick={() => perform(stop)}>{uiText("ui.71b96dcef1a240f4")}</button></div>
</Modal>
