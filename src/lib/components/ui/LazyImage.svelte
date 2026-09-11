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
  let imgEl: HTMLImageElement;
  
  function handleLoad() {
    loaded = true;
  }
  
  function handleError() {
    error = true;
    loaded = true;
  }
</script>

<div class="relative {klass}">
  {#if !loaded}
    <div class="absolute inset-0 animate-pulse bg-bg-subtle rounded"></div>
  {/if}
  <img
    bind:this={imgEl}
    {src}
    {alt}
    {loading}
    class="transition-opacity duration-300 {loaded ? 'opacity-100' : 'opacity-0'} {klass}"
    onload={handleLoad}
    onerror={handleError}
  />
  {#if error && fallback}
    <img
      src={fallback}
      {alt}
      class="absolute inset-0 {klass}"
    />
  {/if}
</div>
