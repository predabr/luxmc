<script lang="ts">
    import { Film } from "lucide-svelte";
    import { loadWallpaperPoster, resolveWallpaperImageUrl } from "$lib/utils/wallpaperSource";

    let { url, type }: { url: string; type: "image" | "video" } = $props();
    let poster = $state("");

    $effect(() => {
        if (type !== "video") return;
        let disposed = false;
        poster = "";
        void loadWallpaperPoster(url).then(image => {
            if (!disposed) poster = image;
        }).catch(() => {});
        return () => { disposed = true; };
    });
</script>

{#if type === "video"}
    {#if poster}
        <img loading="lazy" decoding="async" src={poster} alt="" class="h-full w-full object-cover" />
    {:else}
        <Film class="h-6 w-6 text-brand-400" />
    {/if}
{:else}
    <img loading="lazy" decoding="async" src={resolveWallpaperImageUrl(url)} alt="" class="h-full w-full object-cover" />
{/if}
