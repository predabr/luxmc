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
		children,
		class: klass = "",
		...rest
	}: Props = $props();

	const classes = $derived(button({ variant, size, block, class: klass }));
</script>

{#if href}
	<a {href} class={classes} {...rest as HTMLAnchorAttributes}>
		{#if children}{@render children()}{/if}
	</a>
{:else}
	<button
		class={classes}
		disabled={loading || (rest as HTMLButtonAttributes).disabled}
		{...rest as HTMLButtonAttributes}
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
