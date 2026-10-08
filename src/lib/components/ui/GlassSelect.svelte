<script lang="ts">
    import { button as launcherButton } from "$lib/components/ui/button";
    import { onMount, tick } from "svelte";
    import { ChevronDown, Check, type Icon } from "lucide-svelte";
    let { value = $bindable(""), options, label, icon: IconComponent, disabled = false, onchange }: { value?: string; options: {value: string; label: string}[]; label: string; icon?: typeof Icon; disabled?: boolean; onchange?: (value:string)=>void } = $props();
    let open = $state(false);
    let root: HTMLDivElement;
    let button: HTMLButtonElement;
    const selected = $derived(options.find(option => option.value === value)?.label || label);
    function choose(next: string) { if (disabled) return; value = next; onchange?.(next); open = false; button.focus(); }
    $effect(() => {
        if (!open) return;
        void tick().then(() => { if (open) root.querySelector<HTMLElement>('[role="option"][aria-selected="true"]')?.focus(); });
    });
    function listKey(event: KeyboardEvent) {
        if (event.key === "Escape") { event.preventDefault(); open = false; button.focus(); return; }
        if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
        event.preventDefault();
        const items = Array.from(root.querySelectorAll<HTMLButtonElement>('[role="option"]'));
        if (!items.length) return;
        const index = items.findIndex(item => item === document.activeElement);
        const next = event.key === "Home" ? 0 : event.key === "End" ? items.length - 1 : (index + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
        items[next]?.focus();
    }
    onMount(() => {
        const outside = (event: PointerEvent) => { if (!root.contains(event.target as Node)) open = false; };
        document.addEventListener("pointerdown", outside);
        return () => document.removeEventListener("pointerdown", outside);
    });
</script>
<div class="relative min-w-24" bind:this={root}>
    <button bind:this={button} type="button" {disabled} class={launcherButton({ variant: "secondary", size: "sm", class: "glass-select-trigger w-full justify-between text-left" })} aria-label={label} aria-haspopup="listbox" aria-expanded={open} onclick={() => open = !open} onkeydown={(event) => {
        if (event.key === "Escape") open = false;
        if (event.key === "ArrowDown" || event.key === "ArrowUp") { event.preventDefault(); const index = options.findIndex(option => option.value === value); const next = (index + (event.key === "ArrowDown" ? 1 : -1) + options.length) % options.length; if (options[next]) choose(options[next].value); }
    }}>{#if IconComponent}<IconComponent class="h-4 w-4 shrink-0 text-brand-400" />{/if}<span class="min-w-0 flex-1 truncate">{selected}</span><ChevronDown class="h-4 w-4 shrink-0 text-fg-muted" /></button>
    {#if open}
        <div role="listbox" aria-label={label} tabindex="-1" class="absolute right-0 top-full z-40 mt-2 max-h-64 min-w-full overflow-y-auto rounded-xl border border-border-strong bg-bg-elevated p-1 shadow-elevated custom-scrollbar" onkeydown={listKey}>
            {#each options as option}
                <button type="button" role="option" {disabled} aria-selected={value === option.value} class={launcherButton({ variant: value === option.value ? "ghostBrand" : "ghost", size: "sm", class: "w-full justify-between text-left" })} onclick={() => choose(option.value)}><span class="whitespace-nowrap">{option.label}</span>{#if value === option.value}<Check class="h-4 w-4 shrink-0" />{/if}</button>
            {/each}
        </div>
    {/if}
</div>
