<script lang="ts" generics="Item">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { onMount, type Snippet } from "svelte";
    let { items, columns = 1, rowHeight = 440, children }: { items: Item[]; columns?: number; rowHeight?: number; children: Snippet<[Item]> } = $props();
    let container = $state<HTMLDivElement | null>(null);
    let first = $state(0);
    let visible = $state(4);
    const rowCount = $derived(Math.ceil(items.length / columns));
    const start = $derived(Math.min(first, Math.max(0, rowCount - 1)));
    const end = $derived(Math.min(rowCount, start + visible));
    const rows = $derived(Array.from({ length: Math.max(0, end - start) }, (_, index) => start + index));
    onMount(() => {
        const root = container?.closest("main");
        if (!root || !container) return;
        let frame = 0;
        const update = () => {
            frame = 0;
            if (!container) return;
            const top = container.getBoundingClientRect().top - root.getBoundingClientRect().top;
            first = Math.max(0, Math.floor(-top / rowHeight) - 1);
            visible = Math.ceil(root.clientHeight / rowHeight) + 3;
        };
        const schedule = () => { if (!frame) frame = requestAnimationFrame(update); };
        const observer = new ResizeObserver(schedule);
        observer.observe(root);
        observer.observe(container);
        root.addEventListener("scroll", schedule, { passive: true });
        update();
        return () => { observer.disconnect(); root.removeEventListener("scroll", schedule); cancelAnimationFrame(frame); };
    });
</script>

<div bind:this={container} class="relative w-full" style:height={`${rowCount * rowHeight}px`} data-catalog-results aria-label={uiText("ui.6c6b7279d01d77d4")}>
    {#each rows as row (row)}
        <div class="catalog-result-row absolute left-0 top-0 grid w-full gap-4 pb-4" style:height={`${rowHeight}px`} style:transform={`translateY(${row * rowHeight}px)`} style:grid-template-columns={`repeat(${columns}, minmax(0, 1fr))`}>
            {#each items.slice(row * columns, (row + 1) * columns) as item}
                {@render children(item)}
            {/each}
        </div>
    {/each}
</div>
