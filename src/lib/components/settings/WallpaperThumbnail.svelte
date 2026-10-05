<script lang="ts">
    import { Film } from "lucide-svelte";
    import { mediaServerPort } from "$lib/api/system";
    import { loadWallpaperPoster, resolveWallpaperImageUrl, resolveWallpaperVideoUrl, resolveWallpaperStreamUrl, wallpaperLocalPath } from "$lib/utils/wallpaperSource";

    let { url, type }: { url: string; type: "image" | "video" } = $props();
    let poster = $state("");
    let videoSource = $state("");
    let failed = $state(false);

    $effect(() => {
        const currentUrl = url;
        poster = "";
        failed = false;
        videoSource = "";
        if (type !== "video") return;
        let disposed = false;
        const fallback = resolveWallpaperVideoUrl(currentUrl);
        videoSource = fallback;
        if (wallpaperLocalPath(currentUrl)) {
            void mediaServerPort().then(port => {
                if (!disposed && port) { failed = false; videoSource = resolveWallpaperStreamUrl(currentUrl, port); }
            }).catch(() => {});
        }
        void loadWallpaperPoster(currentUrl).then(image => {
            if (!disposed) poster = image;
        }).catch(() => {});
        return () => { disposed = true; };
    });
</script>

{#if type === "video"}
    {#if poster}
        <img loading="lazy" decoding="async" src={poster} alt="" class="h-full w-full object-cover" />
    {:else if videoSource && !failed}
        <video src={videoSource} muted playsinline preload="metadata" class="h-full w-full object-cover" onloadedmetadata={event => {
            const video = event.currentTarget;
            if (Number.isFinite(video.duration) && video.duration > 0) video.currentTime = Math.min(0.1, video.duration / 2);
        }} onerror={() => failed = true}></video>
    {:else}
        <Film class="h-6 w-6 text-brand-400" />
    {/if}
{:else}
    <img loading="lazy" decoding="async" src={resolveWallpaperImageUrl(url)} alt="" class="h-full w-full object-cover" />
{/if}
