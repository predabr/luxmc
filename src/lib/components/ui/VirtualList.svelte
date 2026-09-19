<script lang="ts" generics="Item">
    import { tick, type Snippet } from "svelte";
    let { items, itemHeight, height, children, class: klass = "" }: {
        items: Item[]; itemHeight: number; height: string; children: Snippet<[Item, number]>; class?: string;
    } = $props();
    let viewport = $state<HTMLDivElement | null>(null);
    let scrollTop = $state(0);
    let viewportHeight = $state(0);
    const rowHeight = $derived(Math.max(1, itemHeight));
    const totalHeight = $derived(items.length * rowHeight);
    const start = $derived(Math.min(Math.floor(scrollTop / rowHeight), Math.max(0, items.length - 1)));
    const end = $derived(Math.min(items.length, start + Math.ceil(viewportHeight / rowHeight) + 2));
    const visibleItems = $derived(items.slice(start, end));
    $effect(() => {
        if (!viewport) return;
        const element = viewport;
        const measure = () => { viewportHeight = element.clientHeight; };
        measure();
        const observer = new ResizeObserver(measure);
        observer.observe(element);
        return () => observer.disconnect();
    });
    $effect(() => {
        if (viewport && scrollTop > Math.max(0, totalHeight - viewportHeight)) {
            viewport.scrollTop = Math.max(0, totalHeight - viewportHeight);
            scrollTop = viewport.scrollTop;
        }
    });
    export async function scrollToIndex(index: number) {
        await tick();
        if (viewport) viewport.scrollTop = Math.max(0, Math.min(index, items.length - 1)) * rowHeight;
    }
</script>
<div bind:this={viewport} class="overflow-auto {klass}" style:height onscroll={(event) => scrollTop = event.currentTarget.scrollTop}>
    <div class="relative" style:height={`${totalHeight}px`}>
        <div class="absolute left-0 right-0 top-0" style:transform={`translateY(${start * rowHeight}px)`}>
            {#each visibleItems as item, index (start + index)}
                <div style:height={`${rowHeight}px`}>{@render children(item, start + index)}</div>
            {/each}
        </div>
    </div>
</div>
