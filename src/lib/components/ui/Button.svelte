<script lang="ts">
	import type { HTMLButtonAttributes, HTMLAnchorAttributes } from "svelte/elements";
	import { button, type ButtonVariant, type ButtonSize } from "./button";
	import type { Snippet } from "svelte";

	type Props = {
		variant?: ButtonVariant;
		size?: ButtonSize;
		block?: boolean;
		href?: string;
		loading?: boolean;
		children?: Snippet;
		class?: string;
	} & Omit<HTMLButtonAttributes & HTMLAnchorAttributes, "class" | "children">;

	let {
		variant = "secondary",
		size = "md",
		block = false,
		href,
		loading = false,
		disabled = false,
		children,
		class: klass = "",
		...rest
	}: Props = $props();

	const classes = $derived(button({ variant, size, block, class: klass }));
</script>

{#if href}
	<a
        {...rest as HTMLAnchorAttributes}
        href={disabled || loading ? undefined : href}
        class={classes}
        aria-disabled={disabled || loading}
        aria-busy={loading}
        tabindex={disabled || loading ? -1 : rest.tabindex}
        onclick={(event) => {
            if (disabled || loading) { event.preventDefault(); event.stopImmediatePropagation(); return; }
            (rest as HTMLAnchorAttributes).onclick?.(event);
        }}
    >
		{#if children}{@render children()}{/if}
	</a>
{:else}
	<button
		class={classes}
		{...rest as HTMLButtonAttributes}
        disabled={loading || disabled}
        aria-busy={loading}
	>
		{#if loading}
			<span
				class="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
				aria-hidden="true"
			></span>
		{/if}
		{#if children}{@render children()}{/if}
	</button>
{/if}
