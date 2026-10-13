<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { AlertCircle, Copy, LoaderCircle, Terminal, X } from "lucide-svelte";
    import { appState } from "$lib/stores/app.svelte";
    import { cancelLaunch, launchSessionStatus, sanitizeLaunchDiagnostic, type LaunchSession } from "$lib/api/launch";
    import { listen } from "$lib/api/client";
    import { button } from "$lib/components/ui/button";
    import { toast } from "$lib/stores/toasts.svelte";

    let session = $state<LaunchSession | null>(null);
    let cancelling = $state(false);
    let elapsed = $state(0);
    const opening = $derived(appState.isLaunching);

    function receive(value: LaunchSession | null) {
        if (!value || typeof value.sessionId !== "string" || typeof value.startedAt !== "string") return;
        if (opening && Date.parse(value.startedAt) < appState.launchAttemptStartedAt) return;
        if (appState.launchingProfileId && value.profileId !== appState.launchingProfileId) return;
        if (appState.activeLaunchSessionId && value.sessionId !== appState.activeLaunchSessionId) return;
        session = value;
        appState.activeLaunchSessionId = value.sessionId;
        elapsed = value.elapsedSeconds;
        if (value.error) appState.lastLaunchError = value.error;
        if (["failed", "cancelled", "exited"].includes(value.phase)) cancelling = false;
    }

    onMount(() => {
        let disposed = false;
        let unlisten: (() => void) | undefined;
        let fetching = false;
        async function refresh() {
            if (fetching) return;
            fetching = true;
            try { const value = await launchSessionStatus(); if (!disposed) receive(value); } catch {}
            finally { fetching = false; }
        }
        void refresh();
        void listen<LaunchSession>("launch-session", event => { if (!disposed) receive(event.payload); })
            .then(value => { if (disposed) value(); else unlisten = value; }).catch(() => {});
        const timer = setInterval(() => {
            if (opening) {
                elapsed = Math.max(0, Math.floor((Date.now() - appState.launchAttemptStartedAt) / 1000));
                void refresh();
            }
        }, 1000);
        return () => { disposed = true; clearInterval(timer); unlisten?.(); };
    });

    async function cancel() {
        if (!session || cancelling) return;
        cancelling = true;
        appState.wasManuallyTerminated = true;
        try {
            if (!(await cancelLaunch(session.sessionId))) { cancelling = false; receive(await launchSessionStatus()); }
        } catch (error) { cancelling = false; toast(String(error), "error"); }
    }

    async function copy() {
        const error = sanitizeLaunchDiagnostic(appState.lastLaunchError || session?.error || "");
        const diagnostic = session ? { ...session, error } : { error };
        try { await navigator.clipboard.writeText(JSON.stringify(diagnostic, null, 2)); toast("Diagnóstico copiado.", "success"); }
        catch { toast("Não foi possível copiar o diagnóstico.", "error"); }
    }
</script>

{#if opening || appState.lastLaunchError}
    <aside aria-label="Inicialização do Minecraft" class="fixed bottom-5 right-5 z-[120] w-[420px] max-w-[calc(100vw-40px)] overflow-hidden rounded-2xl border border-border bg-bg-elevated text-fg shadow-elevated">
        <div class="flex items-start gap-3 p-4">
            {#if opening}<LoaderCircle class="mt-1 h-5 w-5 shrink-0 animate-spin text-brand-400" />{:else}<AlertCircle class="mt-1 h-5 w-5 shrink-0 text-danger" />{/if}
            <div class="min-w-0 flex-1">
                <p class="text-sm font-semibold">{opening ? "Abrindo seu Minecraft" : "O Minecraft não iniciou corretamente"}</p>
                <p role="status" class="mt-2 max-h-40 overflow-y-auto whitespace-pre-wrap break-words text-xs leading-relaxed text-fg-muted">{opening ? (cancelling ? "Cancelando a abertura..." : appState.launchStatusText || "Preparando a instância...") : appState.lastLaunchError}</p>
                {#if opening}<p class="mt-2 text-[11px] text-fg-subtle">{elapsed}s · Você pode navegar enquanto o jogo abre.</p>{/if}
            </div>
            {#if !opening}<button type="button" class={button({ variant: "ghost", size: "icon" })} aria-label="Fechar diagnóstico" onclick={() => appState.lastLaunchError = ""}><X class="h-4 w-4" /></button>{/if}
        </div>
        <div class="flex flex-wrap items-center gap-2 border-t border-border px-4 py-3">
            <button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={() => goto("/logs")}><Terminal class="h-4 w-4" />Ver logs</button>
            {#if opening}<button type="button" class={button({ variant: "danger", size: "sm" })} disabled={!session || cancelling} onclick={cancel}>Cancelar abertura</button>{:else}<button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={copy}><Copy class="h-4 w-4" />Copiar diagnóstico</button>{/if}
        </div>
    </aside>
{/if}
