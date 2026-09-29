<script lang="ts">
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

<div class="relative h-full w-full overflow-hidden">
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
      class="h-full w-full transition-opacity duration-200 {loaded ? 'opacity-100' : 'opacity-0'} {klass}"
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
