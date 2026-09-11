<script lang="ts">
	import Button from "./Button.svelte";
	import Tooltip from "./Tooltip.svelte";
	import type { Snippet } from "svelte";
	import type { ButtonVariant } from "./button";

	type Props = {
		variant?: ButtonVariant;
		tooltip?: string;
		tooltipPosition?: "top" | "bottom" | "left" | "right";
		loading?: boolean;
		disabled?: boolean;
		children?: Snippet;
		class?: string;
		onclick?: (e: MouseEvent) => void;
	};

	let {
		variant = "ghost",
		tooltip,
		tooltipPosition = "top",
		loading = false,
		disabled = false,
		children,
		class: klass = "",
		onclick
	}: Props = $props();
</script>

{#if tooltip}
	<Tooltip content={tooltip} position={tooltipPosition}>
		<Button {variant} size="icon" {loading} {disabled} {onclick} class={klass}>
			{#if children}{@render children()}{/if}
		</Button>
	</Tooltip>
{:else}
	<Button {variant} size="icon" {loading} {disabled} {onclick} class={klass}>
		{#if children}{@render children()}{/if}
	</Button>
{/if}
