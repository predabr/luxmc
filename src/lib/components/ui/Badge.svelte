<script lang="ts">
	import { twMerge } from "tailwind-merge";
	import type { Snippet } from "svelte";

	type Props = {
		variant?: "default" | "brand" | "success" | "warning" | "danger";
		children?: Snippet;
		class?: string;
	};

	let { variant = "default", children, class: klass = "" }: Props = $props();

	const variantClasses: Record<string, string> = {
		default: "border-border bg-bg-subtle text-fg-muted",
		brand: "border-brand-500/30 bg-brand-500/15 text-brand-300",
		success: "border-green-500/30 bg-green-500/15 text-green-400",
		warning: "border-yellow-500/30 bg-yellow-500/15 text-yellow-400",
		danger: "border-red-500/30 bg-red-500/15 text-red-400"
	};
</script>

<span
	class={twMerge(
		"inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-medium leading-none",
		variantClasses[variant],
		klass
	)}
>
	{#if children}{@render children()}{/if}
</span>
