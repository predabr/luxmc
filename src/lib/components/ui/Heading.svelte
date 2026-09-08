<script lang="ts">
	import { twMerge } from "tailwind-merge";
	import type { Snippet } from "svelte";

	type Props = {
		children?: Snippet;
		class?: string;
	};

	let { children, class: klass = "" }: Props = $props();
</script>

<h1
	class={twMerge(
		"text-2xl font-semibold tracking-tight text-fg",
		klass
	)}
>
	{#if children}{@render children()}{/if}
</h1>
