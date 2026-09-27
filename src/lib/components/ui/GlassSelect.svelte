<script lang="ts">
    import { onMount } from "svelte";
    let { value = $bindable(""), options, label }: { value?: string; options: {value: string; label: string}[]; label: string } = $props();
    let open = $state(false);
    let root: HTMLDivElement;
    let button: HTMLButtonElement;
    const selected = $derived(options.find(option => option.value === value)?.label || label);
    function choose(next: string) { value = next; open = false; button.focus(); }
    onMount(() => {
        const outside = (event: PointerEvent) => { if (!root.contains(event.target as Node)) open = false; };
        document.addEventListener("pointerdown", outside);
        return () => document.removeEventListener("pointerdown", outside);
    });
</script>
<div class="relative min-w-24" bind:this={root}>
    <button bind:this={button} type="button" class="glass-select-trigger block w-full rounded-lg bg-transparent p-0 pr-4 text-left text-xs font-semibold text-fg shadow-none focus-visible:outline-none" aria-label={label} aria-haspopup="listbox" aria-expanded={open} onclick={() => open = !open} onkeydown={(event) => {
        if (event.key === "Escape") open = false;
        if (event.key === "ArrowDown" || event.key === "ArrowUp") { event.preventDefault(); const index = options.findIndex(option => option.value === value); const next = (index + (event.key === "ArrowDown" ? 1 : -1) + options.length) % options.length; if (options[next]) value = options[next].value; }
    }}>{selected}</button>
    {#if open}
        <div role="listbox" aria-label={label} tabindex="-1" class="absolute left-0 top-full z-40 mt-3 min-w-44 rounded-xl border border-fg/10 bg-bg/90 p-1.5 shadow-xl backdrop-blur-2xl" onkeydown={(event) => { if (event.key === "Escape") { open = false; button.focus(); } }}>
            {#each options as option}
                <button type="button" role="option" aria-selected={value === option.value} class="block w-full rounded-lg bg-transparent px-3 py-2 text-left text-xs text-fg hover:bg-fg/10 focus:bg-fg/10" onclick={() => choose(option.value)}>{option.label}</button>
            {/each}
        </div>
    {/if}
</div>
