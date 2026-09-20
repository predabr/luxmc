<script lang="ts" generics="Item">
    import { tick, type Snippet } from "svelte";

    let { items, itemHeight, height, children, class: klass = "" }: {
        items: Item[]; itemHeight: number; height: string; children: Snippet<[Item, number]>; class?: string;
    } = $props();

    let viewport = $state<HTMLDivElement | null>(null);
    let scrollTop = $state(0);
    let viewportHeight = $state(600);
    let rafId: number | null = null;

    const rowHeight = $derived(Math.max(1, itemHeight));
    const totalHeight = $derived(items.length * rowHeight);
    const overscan = 4;
    const start = $derived(Math.max(0, Math.min(Math.floor(scrollTop / rowHeight) - overscan, items.length - 1)));
    const end = $derived(Math.min(items.length, Math.ceil((scrollTop + viewportHeight) / rowHeight) + overscan));
    const visibleItems = $derived(items.slice(start, end));

    $effect(() => {
        if (!viewport) return;
        const element = viewport;
        const measure = () => {
            if (element.clientHeight > 0) {
                viewportHeight = element.clientHeight;
            }
        };
        measure();
        const observer = new ResizeObserver(measure);
        observer.observe(element);
        return () => {
            observer.disconnect();
            if (rafId !== null) {
                cancelAnimationFrame(rafId);
            }
        };
    });

    function handleScroll(e: UIEvent) {
        const target = e.currentTarget as HTMLDivElement;
        if (!target) return;
        if (rafId !== null) return;
        rafId = requestAnimationFrame(() => {
            rafId = null;
            scrollTop = target.scrollTop;
        });
    }

    export async function scrollToIndex(index: number) {
        await tick();
        if (viewport) {
            viewport.scrollTop = Math.max(0, Math.min(index, items.length - 1)) * rowHeight;
            scrollTop = viewport.scrollTop;
        }
    }
</script>

<div 
    bind:this={viewport} 
    class="overflow-auto {klass}" 
    style:height 
    onscroll={handleScroll}
>
    <div class="relative w-full" style:height={`${totalHeight}px`}>
        <div class="absolute left-0 right-0 top-0 will-change-transform" style:transform={`translateY(${start * rowHeight}px)`}>
            {#each visibleItems as item, index (start + index)}
                <div style:height={`${rowHeight}px`}>
                    {@render children(item, start + index)}
                </div>
            {/each}
        </div>
    </div>
</div>
