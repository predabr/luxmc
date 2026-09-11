<script lang="ts">
  import { tick } from "svelte";
  
  type Props = {
    items: any[];
    itemHeight: number;
    height: string;
    children: any;
    class?: string;
  };

  let { items, itemHeight, height, children, class: klass = "" }: Props = $props();

  let viewport: HTMLDivElement;
  let scrollTop = $state(0);

  let start = $derived(Math.floor(scrollTop / itemHeight));
  let end = $derived(Math.min(items.length, start + Math.ceil(parseInt(height) / itemHeight) + 2));
  let visibleItems = $derived(items.slice(start, end));
  let totalHeight = $derived(items.length * itemHeight);
  let paddingTop = $derived(start * itemHeight);

  function handleScroll(e: Event) {
    scrollTop = (e.target as HTMLDivElement).scrollTop;
  }

  async function scrollToIndex(index: number) {
    await tick();
    if (viewport) {
      viewport.scrollTop = index * itemHeight;
    }
  }
</script>

<div
  bind:this={viewport}
  class="overflow-auto {klass}"
  style="height: {height};"
  onscroll={handleScroll}
>
  <div style="height: {totalHeight}px; padding-top: {paddingTop}px;">
    {#each visibleItems as item, i (item.id ?? item)}
      <div style="height: {itemHeight}px;">
        {@render children(item, start + i)}
      </div>
    {/each}
  </div>
</div>
