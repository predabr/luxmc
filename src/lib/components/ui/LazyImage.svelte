<script lang="ts">
  import { cn } from "$lib/utils/cn";
  let { 
    src, 
    alt = "", 
    class: klass = "",
    fallback = "",
    loading = "lazy"
  }: { 
    src: string; 
    alt?: string; 
    class?: string; 
    fallback?: string;
    loading?: "lazy" | "eager";
  } = $props();
  
  let loaded = $state(false);
  let error = $state(false);
  
  $effect(() => {
    src;
    loaded = false;
    error = false;
  });

  function handleLoad() {
    loaded = true;
  }
  
  function handleError() {
    error = true;
    loaded = true;
  }
</script>

<div class={cn("relative h-full w-full overflow-hidden", klass)}>
  {#if !loaded && !error}
    <div class="absolute inset-0 bg-bg-subtle rounded animate-pulse"></div>
  {/if}
  {#if !error}
    <img
      {src}
      {alt}
      {loading}
      decoding="async"
      referrerpolicy="no-referrer"
      class="h-full w-full transition-opacity duration-150 {loaded ? 'opacity-100' : 'opacity-0'} {klass.includes('object-cover') ? 'object-cover' : 'object-contain'}"
      onload={handleLoad}
      onerror={handleError}
    />
  {:else}
    <img
      src={fallback || "/grass_block.png"}
      {alt}
      class="h-full w-full object-contain p-1 rounded-xl"
      onerror={(e) => { const image = e.currentTarget as HTMLImageElement; if (!image.src.endsWith("/grass_block.png")) image.src = "/grass_block.png"; }}
    />
  {/if}
</div>
