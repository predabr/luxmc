<script lang="ts" generics="Item">
    import { type Snippet, onMount, untrack } from "svelte";
    import { Virtualizer, observeElementRect, observeElementOffset, elementScroll, type VirtualItem } from "@tanstack/svelte-virtual";

    let { items, itemHeight = 72, height = "600px", overscan = 4, fixedHeight = false, getKey, children, class: klass = "" }:
        { items: Item[]; itemHeight?: number; height?: string; overscan?: number; fixedHeight?: boolean; getKey?: (item: Item) => string | number; children: Snippet<[Item, number]>; class?: string } = $props();
    let viewport = $state<HTMLDivElement | null>(null);
    let rows = $state.raw<VirtualItem[]>([]);
    let totalSize = $state(0);
    const virtualizer = new Virtualizer<HTMLDivElement, HTMLDivElement>({
        count: 0, getScrollElement: () => viewport, estimateSize: () => 72,
        observeElementRect, observeElementOffset, scrollToFn: elementScroll,
        onChange: instance => { rows = instance.getVirtualItems(); totalSize = instance.getTotalSize(); }
    });
    onMount(() => virtualizer._didMount());
    $effect(() => {
        const currentItems = items;
        const count = currentItems.length;
        const key = getKey;
        const estimate = Math.max(1, itemHeight);
        const extra = overscan;
        const element = viewport;
        untrack(() => {
            virtualizer.setOptions({ ...virtualizer.options, count, estimateSize: () => estimate, overscan: extra, getItemKey: index => key ? key(currentItems[index]) : index, getScrollElement: () => element });
            virtualizer._willUpdate();
            if (element && element.scrollTop > Math.max(0, virtualizer.getTotalSize() - element.clientHeight)) {
                element.scrollTop = Math.max(0, virtualizer.getTotalSize() - element.clientHeight);
            }
            rows = virtualizer.getVirtualItems();
            totalSize = virtualizer.getTotalSize();
        });
    });
    function measure(node: HTMLDivElement, fixed: boolean) {
        if (fixed) return;
        virtualizer.measureElement(node);
        return { destroy() { queueMicrotask(() => virtualizer.measureElement(null)); } };
    }
    export function scrollToIndex(index: number): void { virtualizer.scrollToIndex(index); }
</script>

<div bind:this={viewport} class="virtual-scroll-viewport overflow-auto custom-scrollbar {klass}" style:height style:max-height={`${Math.max(1, itemHeight) * 16}px`} data-virtual-list>
    <div class="relative w-full" style:height={`${totalSize}px`}>
        {#each rows as row (row.key)}
            {@const item = items[row.index]}
            {#if item !== undefined}
                <div use:measure={fixedHeight} data-index={row.index} class="absolute left-0 top-0 w-full focus-within:z-20" style:min-height={`${itemHeight}px`} style:height={fixedHeight ? `${itemHeight}px` : undefined} style:transform={`translateY(${row.start}px)`}>
                    {@render children(item, row.index)}
                </div>
            {/if}
        {/each}
    </div>
</div>
