<script lang="ts">
	import { twMerge } from "tailwind-merge";
	import type { Snippet } from "svelte";

	type Props = {
		variant?: "default" | "brand" | "success" | "warning" | "danger" | "info";
		children?: Snippet;
		class?: string;
	};

	let { variant = "default", children, class: klass = "" }: Props = $props();

	const variantClasses: Record<string, string> = {
		default: "border-border bg-bg-subtle text-fg-muted",
		brand: "border-brand-500/30 bg-brand-500/15 text-brand-300",
		success: "border-success/30 bg-success/15 text-success",
		warning: "border-warning/30 bg-warning/15 text-warning",
		danger: "border-danger/30 bg-danger/15 text-danger",
		info: "border-info/30 bg-info/15 text-info"
	};
</script>

<span
	class={twMerge(
		"inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-medium leading-none transition-colors",
		variantClasses[variant],
		klass
	)}
>
	{#if children}{@render children()}{/if}
</span>
