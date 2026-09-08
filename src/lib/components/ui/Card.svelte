<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";
	import { tv, type VariantProps } from "tailwind-variants";
	import type { Snippet } from "svelte";

	const card = tv({
		base: "rounded-xl border border-border bg-bg-elevated",
		variants: {
			padding: { none: "", sm: "p-3", md: "p-5", lg: "p-7" },
			interactive: { true: "transition-all duration-150 hover:border-brand-500/40 cursor-pointer" },
			elevated: { true: "shadow-sm shadow-black/10" }
		},
		defaultVariants: { padding: "md" }
	});

	type Props = {
		padding?: VariantProps<typeof card>["padding"];
		interactive?: boolean;
		elevated?: boolean;
		header?: Snippet;
		footer?: Snippet;
		children?: Snippet;
		class?: string;
	} & HTMLAttributes<HTMLElement>;

	let { padding = "md", interactive = false, elevated = false, header, footer, children, class: klass = "", ...rest }: Props = $props();
	const classes = $derived(card({ padding, interactive, elevated, class: klass }));
</script>

<section class={classes} {...rest}>
	{#if header}
		<header class="mb-4 flex items-center justify-between">
			{@render header()}
		</header>
	{/if}
	{#if children}{@render children()}{/if}
	{#if footer}
		<footer class="mt-4 border-t border-border pt-4">
			{@render footer()}
		</footer>
	{/if}
</section>
