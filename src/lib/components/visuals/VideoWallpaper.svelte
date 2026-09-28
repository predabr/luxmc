<script lang="ts">
    import { onMount } from "svelte";
    import { settings } from "$lib/stores/settings.svelte";
    import { appState } from "$lib/stores/app.svelte";
    import { loadWallpaperPoster, resolveWallpaperVideoUrl } from "$lib/utils/wallpaperSource";

    let { src, preview = false }: { src: string; preview?: boolean } = $props();
    let video = $state<HTMLVideoElement | null>(null);
    let poster = $state("");
    let posterReady = $state(false);
    let visible = $state(true);
    let focused = $state(true);
    let failed = $state(false);

    const source = $derived(resolveWallpaperVideoUrl(src));
    const paused = $derived(
        preview || !visible ||
        (settings.value.pauseWallpaperOnBlur !== false && !focused) ||
        appState.isGameRunning || appState.performanceMode
    );

    onMount(() => {
        let blurTimer: ReturnType<typeof setTimeout> | null = null;
        const onVisibility = () => { visible = !document.hidden; };
        const onFocus = () => {
            if (blurTimer) clearTimeout(blurTimer);
            blurTimer = null;
            focused = true;
        };
        const onBlur = () => {
            if (blurTimer) clearTimeout(blurTimer);
            blurTimer = setTimeout(() => { focused = document.hasFocus(); }, 250);
        };
        onVisibility();
        focused = document.hasFocus();
        document.addEventListener("visibilitychange", onVisibility);
        window.addEventListener("focus", onFocus);
        window.addEventListener("blur", onBlur);
        return () => {
            if (blurTimer) clearTimeout(blurTimer);
            document.removeEventListener("visibilitychange", onVisibility);
            window.removeEventListener("focus", onFocus);
            window.removeEventListener("blur", onBlur);
        };
    });

    $effect(() => {
        const currentSource = src;
        poster = "";
        posterReady = false;
        if (!currentSource) return;
        let disposed = false;
        void loadWallpaperPoster(currentSource).then(image => {
            if (!disposed) poster = image;
        }).catch(() => {});
        return () => { disposed = true; };
    });

    $effect(() => {
        const el = video;
        const currentSource = source;
        if (!el || !currentSource) return;

        let disposed = false;
        let previousTime = 0;
        let frameCallback: number | null = null;
        let monitorCallback: number | null = null;
        let revealFrame: number | null = null;
        let recoveryTimer: ReturnType<typeof setTimeout> | null = null;
        failed = false;
        el.style.opacity = "1";

        const cover = () => {
            if (posterReady && !preview) el.style.opacity = "0";
        };
        const reveal = () => {
            if (disposed) return;
            el.style.opacity = "1";
        };
        const revealAfterFrame = () => {
            if (frameCallback !== null || revealFrame !== null) return;
            if (el.requestVideoFrameCallback) {
                frameCallback = el.requestVideoFrameCallback(() => {
                    frameCallback = null;
                    reveal();
                });
            } else {
                revealFrame = requestAnimationFrame(() => {
                    revealFrame = requestAnimationFrame(() => {
                        revealFrame = null;
                        if (el.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA && !el.seeking) reveal();
                    });
                });
            }
        };
        const monitorFrame: VideoFrameRequestCallback = (_now, metadata) => {
            monitorCallback = null;
            if (disposed) return;
            if (Number.isFinite(el.duration) && el.duration - metadata.mediaTime < 0.12) cover();
            else if (el.style.opacity === "0" && !el.seeking && el.duration - metadata.mediaTime > 0.2) reveal();
            monitorCallback = el.requestVideoFrameCallback(monitorFrame);
        };
        if (el.requestVideoFrameCallback && !preview) monitorCallback = el.requestVideoFrameCallback(monitorFrame);
        const onTimeUpdate = () => {
            if (disposed) return;
            if (el.currentTime + 0.2 < previousTime) revealAfterFrame();
            if (Number.isFinite(el.duration) && el.duration - el.currentTime < 0.18) cover();
            previousTime = el.currentTime;
        };
        const onSeeking = () => { if (!disposed) cover(); };
        const onSeeked = () => { if (!disposed) revealAfterFrame(); };
        const onWaiting = () => { if (!disposed) cover(); };
        const onPlaying = () => {
            if (!disposed && el.style.opacity === "0" && !el.seeking) revealAfterFrame();
        };
        const onPause = () => {
            if (disposed || paused || el.ended || preview) return;
            if (recoveryTimer) clearTimeout(recoveryTimer);
            recoveryTimer = setTimeout(() => {
                if (!disposed && !paused && el.paused && !el.ended) void el.play().catch(() => {});
            }, 250);
        };
        const onError = () => {
            if (disposed) return;
            failed = true;
            cover();
        };

        el.addEventListener("timeupdate", onTimeUpdate);
        el.addEventListener("seeking", onSeeking);
        el.addEventListener("seeked", onSeeked);
        el.addEventListener("waiting", onWaiting);
        el.addEventListener("playing", onPlaying);
        el.addEventListener("pause", onPause);
        el.addEventListener("error", onError);
        return () => {
            disposed = true;
            if (frameCallback !== null) el.cancelVideoFrameCallback?.(frameCallback);
            if (monitorCallback !== null) el.cancelVideoFrameCallback?.(monitorCallback);
            if (revealFrame !== null) cancelAnimationFrame(revealFrame);
            if (recoveryTimer) clearTimeout(recoveryTimer);
            el.removeEventListener("timeupdate", onTimeUpdate);
            el.removeEventListener("seeking", onSeeking);
            el.removeEventListener("seeked", onSeeked);
            el.removeEventListener("waiting", onWaiting);
            el.removeEventListener("playing", onPlaying);
            el.removeEventListener("pause", onPause);
            el.removeEventListener("error", onError);
        };
    });

    $effect(() => {
        const el = video;
        if (!el) return;
        if (paused) el.pause();
        else void el.play().catch(() => {});
    });
</script>

<div class="absolute inset-0 overflow-hidden bg-bg" aria-hidden="true">
    {#if poster}
        <img
            src={poster}
            alt=""
            class="absolute inset-0 h-full w-full object-cover pointer-events-none"
            onload={() => posterReady = true}
            onerror={() => posterReady = false}
        />
    {/if}
    {#if !failed}
        <video
            bind:this={video}
            src={source}
            autoplay={!paused}
            loop
            muted
            playsinline
            preload={preview ? "metadata" : "auto"}
            class="absolute inset-0 h-full w-full object-cover pointer-events-none"
            style="transform: translateZ(0); backface-visibility: hidden;"
        ></video>
    {/if}
    {#if failed && preview}
        <span class="absolute inset-0 flex items-center justify-center text-xs text-fg-muted">Prévia indisponível</span>
    {/if}
</div>
