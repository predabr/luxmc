<script lang="ts">
    import { goto } from "$app/navigation";
    import { ArrowLeft } from "lucide-svelte";
    import { button } from "$lib/components/ui/button";
    import VirtualLanPanel from "$lib/components/friends/VirtualLanPanel.svelte";
</script>
<svelte:head><title>Rede LAN · Luxmc</title></svelte:head>
<div class="mx-auto w-full max-w-5xl space-y-5 pb-8">
    <button class={button({ variant: "ghost", size: "sm" })} onclick={() => goto("/friends")}><ArrowLeft class="h-4 w-4" />Amigos</button>
    <VirtualLanPanel />
</div>