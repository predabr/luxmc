<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { goto } from "$app/navigation";
    import { Copy, Play, LogOut, Users, ShieldCheck } from "lucide-svelte";
    import { button } from "$lib/components/ui/button";
    import RoomAvatar from "$lib/components/friends/RoomAvatar.svelte";
    import { saveTunnelServer, stopSession, type TunnelStatus } from "$lib/api/tunnel";
    import { profiles } from "$lib/stores/profiles.svelte";
    import { appState } from "$lib/stores/app.svelte";
    import { joinWorld } from "$lib/utils/directJoin";
    import { toast } from "$lib/stores/toasts.svelte";

    let { room }: { room: TunnelStatus } = $props();
    let profileId = $state(profiles.active?.id || profiles.list[0]?.id || "");
    let savedKey = $state("");
    let saved = $state(false);
    let error = $state("");
    let busy = $state(false);
    let generation = 0;

    async function save() {
        if (!profileId || !room.localAddress) return;
        const current = ++generation;
        const key = `${profileId}:${room.localAddress}`;
        savedKey = key;
        saved = false;
        error = "";
        try { await saveTunnelServer(profileId); if (current === generation) saved = true; }
        catch (cause) { if (current === generation) error = String(cause); }
    }

    $effect(() => { if (!profileId) profileId = appState.activeGameDetails?.profileId || profiles.active?.id || profiles.list[0]?.id || ""; });

    $effect(() => {
        const key = `${profileId}:${room.localAddress}`;
        if (profileId && room.localAddress && key !== savedKey) void save();
    });

    async function perform(action: () => Promise<void>) {
        if (busy) return;
        busy = true;
        try { await action(); }
        catch (cause) { error = String(cause); toast(error, "error"); }
        finally { busy = false; }
    }

    async function copy() {
        if (!room.localAddress) return;
        await navigator.clipboard.writeText(room.localAddress);
        toast(uiText("ui.070b794ecd3daf56"), "success");
    }
</script>

<div class="space-y-6" aria-label={uiText("ui.20c3db1c8098dbd6")}>
    <header class="surface-glass space-y-5 p-6 sm:p-8">
        <div class="flex flex-wrap items-start justify-between gap-4">
            <div><span class="mb-3 inline-flex items-center gap-2 text-xs font-semibold text-success"><ShieldCheck class="h-4 w-4" />{uiText("ui.24a3bd9986f30b86")}</span><h1 class="text-3xl font-bold text-fg">{uiText("ui.8739aab31a4cce37")}</h1><p class="mt-2 text-sm text-fg-muted">{room.worldReady === false ? uiText("multiplayer.waitingWorld") : uiText("multiplayer.worldReady")}</p></div>
            <div class="rounded-2xl border border-success/20 bg-success/10 px-5 py-4"><p class="flex items-center gap-2 text-xl font-bold text-success"><Users class="h-5 w-5" />{room.members.length} / {room.maxPlayers}</p><p class="mt-1 text-xs text-fg-muted">{room.roomCode || uiText("ui.39b8a9d7111676e1")}</p></div>
        </div>
        <div class="space-y-3 rounded-xl border border-border bg-bg/40 p-5">
            <p class="text-xs font-semibold text-fg-muted">{uiText("ui.3242caadbef1c82c")}</p>
            <div class="flex flex-wrap items-center justify-between gap-3"><p class="select-text font-mono text-2xl font-bold text-fg">{room.localAddress}</p><button type="button" class={button({ variant: "primary", size: "lg" })} disabled={busy || !room.localAddress} onclick={() => perform(copy)}><Copy class="h-4 w-4" />{uiText("ui.adca1d03fbacd50b")}</button></div>
            <p class="text-sm text-fg-muted">{uiText("ui.dca34fd019f252ac")}</p>
            <p class="text-xs text-fg-subtle">{room.transport}{room.pingMs !== null ? ` · ${room.pingMs} ms` : ""}</p>
        </div>
        <div class="space-y-3">
            <label class="block text-xs font-semibold text-fg-muted">{uiText("ui.2f0231c10c57ef8b")}<select class="mt-2 w-full rounded-xl border border-border bg-bg-elevated p-3 text-sm text-fg" bind:value={profileId}><option value="">{uiText("ui.4fb110d2c5c36c1d")}</option>{#each profiles.list as profile}<option value={profile.id}>{profile.name} · {profile.mcVersion}</option>{/each}</select></label>
            {#if saved}<p role="status" class="text-sm text-success">{uiText("ui.47e59f2e26b276a7")}</p>{/if}
            {#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}
            <div class="flex flex-wrap gap-3"><button type="button" class={button({ variant: "secondary" })} disabled={busy || !profileId} onclick={() => perform(save)}>{uiText("ui.bfff1ee45c541b7e")}</button>{#if !appState.isGameRunning}<button type="button" class={button({ variant: "primary" })} disabled={busy || appState.isLaunching || !profileId || !room.localAddress || room.worldReady === false} onclick={() => perform(() => joinWorld(room.localAddress!, undefined, profileId))}><Play class="h-4 w-4" />{uiText("ui.7576e61f51f933e1")}</button>{/if}</div>
            <p class="text-xs text-fg-muted">{uiText("multiplayer.sameVersions")}</p>
        </div>
    </header>
    <section class="surface-glass p-6" aria-label={uiText("ui.4b77e3efe74df84a")}><h2 class="mb-5 text-lg font-semibold text-fg">{uiText("ui.aff4a93487ceb273")}</h2><ul class="grid gap-3 sm:grid-cols-2">{#each room.members as member (member.id)}<li class="flex items-center gap-4 rounded-2xl border border-border bg-bg/35 p-4" data-room-member={member.id}><RoomAvatar {member} /><div><p class="font-semibold text-fg">{member.username}</p><p class="mt-1 text-xs text-fg-muted">{member.isHost ? "Hospedeiro" : "Convidado"}</p></div></li>{/each}</ul></section>
    <footer class="flex justify-end"><button type="button" class={button({ variant: "danger", size: "lg" })} disabled={busy} onclick={() => perform(async () => { await stopSession(); await goto("/friends"); })}><LogOut class="h-4 w-4" />{uiText("ui.7d9b7fe65c86fb5f")}</button></footer>
</div>
