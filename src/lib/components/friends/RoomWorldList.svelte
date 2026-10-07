<script lang="ts">
    import { Copy, RadioTower } from "lucide-svelte";
    import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import type { TunnelStatus } from "$lib/api/tunnel";
    import { button } from "$lib/components/ui/button";
    import { toast } from "$lib/stores/toasts.svelte";
    let { room }: { room: TunnelStatus } = $props();
    async function copy(address: string) {
        try { await navigator.clipboard.writeText(address); toast(uiText("multiplayer.addressCopied"), "success"); }
        catch (error) { toast(String(error), "error"); }
    }
</script>

<section class="surface-glass space-y-4 p-6">
    <div><h2 class="flex items-center gap-2 text-lg font-semibold text-fg"><RadioTower class="h-5 w-5 text-success" />{uiText("multiplayer.worldsTitle")}</h2><p class="mt-2 text-sm text-fg-muted">{uiText("multiplayer.worldsHelp")}</p></div>
    {#each room.worlds ?? [] as world (world.ownerId)}
        <div class="flex flex-wrap items-center gap-4 rounded-2xl border border-border bg-bg/30 p-4">
            <div class="min-w-0 flex-1"><p class="font-semibold text-fg">{world.ownerUsername}</p><p class="mt-1 break-words text-sm text-fg-muted">{world.motd.replace(/§./g, "")}</p></div>
            {#if world.localAddress}<code class="select-text text-sm text-success">{world.localAddress}</code><button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={() => copy(world.localAddress!)}><Copy class="h-4 w-4" />{uiText("common.copy")}</button>{:else}<span class="text-xs text-fg-muted">{uiText("multiplayer.yourWorld")}</span>{/if}
        </div>
    {:else}<p class="rounded-xl border border-border p-4 text-sm text-fg-muted">{uiText("multiplayer.waitingAnyWorld")}</p>{/each}
</section>
