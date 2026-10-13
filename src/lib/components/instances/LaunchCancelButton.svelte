<script lang="ts">
    import { Square, LoaderCircle } from "lucide-svelte";
    import { appState } from "$lib/stores/app.svelte";
    import { cancelLaunch } from "$lib/api/launch";
    import { button } from "$lib/components/ui/button";
    import { toast } from "$lib/stores/toasts.svelte";

    let { expanded = true }: { expanded?: boolean } = $props();
    let cancelling = $state(false);
    const sessionId = $derived(appState.activeLaunchSessionId);
    $effect(() => { sessionId; cancelling = false; });

    async function cancel() {
        if (!sessionId || cancelling) return;
        const target = sessionId;
        cancelling = true;
        try {
            const accepted = await cancelLaunch(target);
            if (target !== appState.activeLaunchSessionId) return;
            if (accepted) appState.wasManuallyTerminated = true;
            else cancelling = false;
        } catch (error) { cancelling = false; toast(String(error), "error"); }
    }
</script>

{#if appState.isLaunching}
    <button type="button" aria-label="Cancelar abertura" title="Cancelar abertura" disabled={!sessionId || cancelling} onclick={cancel}
        class={button({ variant: "ghost", size: expanded ? "md" : "icon", class: expanded ? "w-[calc(100%_-_1.5rem)] justify-start" : "" })}>
        {#if cancelling}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Square class="h-4 w-4" />{/if}
        {#if expanded}<span>{cancelling ? "Cancelando..." : "Cancelar abertura"}</span>{/if}
    </button>
{/if}
