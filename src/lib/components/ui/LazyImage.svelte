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
      class="h-full w-full transition-opacity duration-200 {loaded ? 'opacity-100' : 'opacity-0'} {klass}"
      onload={handleLoad}
      onerror={handleError}
    />
  {:else}
    <img
      src={fallback || "/grass_block.png"}
      {alt}
      class="h-full w-full object-contain p-1 rounded-xl"
      onerror={(e) => { (e.currentTarget as HTMLImageElement).src = "/grass_block.png"; }}
    />
  {/if}
</div>
