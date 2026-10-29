<script lang="ts">
    import { goto } from "$app/navigation";
    import { ArrowLeft } from "lucide-svelte";
    import { button } from "$lib/components/ui/button";
    import RoomPanel from "$lib/components/friends/RoomPanel.svelte";
</script>
<svelte:head><title>Jogar juntos · Luxmc</title></svelte:head>
<div class="mx-auto w-full max-w-5xl space-y-5 pb-8">
    <button class={button({ variant: "ghost", size: "sm" })} onclick={() => goto("/friends")}><ArrowLeft class="h-4 w-4" />Amigos</button>
    <RoomPanel />
</div>
