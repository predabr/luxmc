<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { AlertCircle, Copy, Terminal, X } from "lucide-svelte";
    import { appState } from "$lib/stores/app.svelte";
    import { launchSessionStatus, sanitizeLaunchDiagnostic, type LaunchSession } from "$lib/api/launch";
    import { listen } from "$lib/api/client";
    import { button } from "$lib/components/ui/button";
    import { toast } from "$lib/stores/toasts.svelte";

    let session = $state<LaunchSession | null>(null);
    const opening = $derived(appState.isLaunching);

    function receive(value: LaunchSession | null) {
        if (!value || typeof value.sessionId !== "string" || typeof value.startedAt !== "string") return;
        if (opening && Date.parse(value.startedAt) < appState.launchAttemptStartedAt) return;
        if (appState.launchingProfileId && value.profileId !== appState.launchingProfileId) return;
        if (appState.activeLaunchSessionId && value.sessionId !== appState.activeLaunchSessionId) return;
        session = value;
        appState.activeLaunchSessionId = value.sessionId;
        if (value.error) appState.lastLaunchError = value.error;
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
            if (opening && !document.hidden) void refresh();
        }, 10000);
        return () => { disposed = true; clearInterval(timer); unlisten?.(); };
    });

    async function copy() {
        const error = sanitizeLaunchDiagnostic(appState.lastLaunchError || session?.error || "");
        const diagnostic = session ? { ...session, error } : { error };
        try { await navigator.clipboard.writeText(JSON.stringify(diagnostic, null, 2)); toast("Diagnóstico copiado.", "success"); }
        catch { toast("Não foi possível copiar o diagnóstico.", "error"); }
    }
</script>

{#if !opening && appState.lastLaunchError && !appState.wasManuallyTerminated}
    <aside aria-label="Inicialização do Minecraft" class="fixed bottom-5 right-5 z-[120] w-[420px] max-w-[calc(100vw-40px)] overflow-hidden rounded-2xl border border-border bg-bg-elevated text-fg shadow-elevated">
        <div class="flex items-start gap-3 p-4">
            <AlertCircle class="mt-1 h-5 w-5 shrink-0 text-danger" />
            <div class="min-w-0 flex-1">
                <p class="text-sm font-semibold">O Minecraft não iniciou corretamente</p>
                <p role="status" class="mt-2 max-h-40 overflow-y-auto whitespace-pre-wrap break-words text-xs leading-relaxed text-fg-muted">{appState.lastLaunchError}</p>
            </div>
            <button type="button" class={button({ variant: "ghost", size: "icon" })} aria-label="Fechar diagnóstico" onclick={() => appState.lastLaunchError = ""}><X class="h-4 w-4" /></button>
        </div>
        <div class="flex flex-wrap items-center gap-2 border-t border-border px-4 py-3">
            <button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={() => goto("/logs")}><Terminal class="h-4 w-4" />Ver logs</button>
            <button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={copy}><Copy class="h-4 w-4" />Copiar diagnóstico</button>
        </div>
    </aside>
{/if}
