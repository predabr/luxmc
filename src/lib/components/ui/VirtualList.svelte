<script lang="ts" generics="Item">
    import { type Snippet, onMount, untrack } from "svelte";
    import { Virtualizer, observeElementRect, observeElementOffset, elementScroll, defaultRangeExtractor, type VirtualItem } from "@tanstack/svelte-virtual";

    let { items, itemHeight = 72, height = "600px", overscan = 2, children, class: klass = "" }:
        { items: Item[]; itemHeight?: number; height?: string; overscan?: number; children: Snippet<[Item, number]>; class?: string } = $props();
    let viewport = $state<HTMLDivElement | null>(null);
    let rows = $state.raw<VirtualItem[]>([]);
    let totalSize = $state(0);
    const virtualizer = new Virtualizer<HTMLDivElement, HTMLDivElement>({
        count: 0, getScrollElement: () => viewport, estimateSize: () => 72,
        observeElementRect, observeElementOffset, scrollToFn: elementScroll,
        onChange: instance => { rows = instance.getVirtualItems(); totalSize = instance.getTotalSize(); },
        rangeExtractor: range => {
            const visible = range.endIndex - range.startIndex + 1;
            return defaultRangeExtractor({ ...range, overscan: Math.max(0, Math.min(range.overscan, Math.floor((20 - visible) / 2))) }).slice(0, 20);
        }
    });
    onMount(() => virtualizer._didMount());
    $effect(() => {
        const count = items.length;
        const estimate = Math.max(1, itemHeight);
        const extra = overscan;
        const element = viewport;
        untrack(() => {
            virtualizer.setOptions({ ...virtualizer.options, count, estimateSize: () => estimate, overscan: extra, getScrollElement: () => element });
            virtualizer._willUpdate();
            rows = virtualizer.getVirtualItems();
            totalSize = virtualizer.getTotalSize();
        });
    });
    function measure(node: HTMLDivElement) {
        virtualizer.measureElement(node);
        return { destroy() { queueMicrotask(() => virtualizer.measureElement(null)); } };
    }
    export function scrollToIndex(index: number): void { virtualizer.scrollToIndex(index); }
</script>

<div bind:this={viewport} class="overflow-auto {klass}" style:height style:max-height={`${Math.max(1, itemHeight) * 16}px`} data-virtual-list>
    <div class="relative w-full" style:height={`${totalSize}px`}>
        {#each rows as row (row.key)}
            {@const item = items[row.index]}
            {#if item !== undefined}
                <div use:measure data-index={row.index} class="absolute left-0 top-0 w-full" style:min-height={`${itemHeight}px`} style:transform={`translateY(${row.start}px)`}>
                    {@render children(item, row.index)}
                </div>
            {/if}
        {/each}
    </div>
</div>
