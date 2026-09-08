<script lang="ts">
	import type { Snippet } from "svelte";

	interface Props {
		content: string;
		position?: 'top' | 'bottom' | 'left' | 'right';
		delay?: number;
		maxWidth?: string;
		children?: Snippet;
	}

	let { content, position = 'top', delay = 300, maxWidth = 'max-w-xs', children }: Props = $props();
	let visible = $state(false);
	let timeout: ReturnType<typeof setTimeout> | undefined;
	let triggerEl: HTMLDivElement | undefined = $state();
	let tooltipEl: HTMLDivElement | undefined = $state();

	const handleMouseEnter = () => {
		timeout = setTimeout(() => (visible = true), delay);
	};

	const handleMouseLeave = () => {
		clearTimeout(timeout);
		visible = false;
	};

	const handleFocus = () => (visible = true);
	const handleBlur = () => (visible = false);
</script>

<div
	bind:this={triggerEl}
	role="presentation"
	onmouseenter={handleMouseEnter}
	onmouseleave={handleMouseLeave}
	onfocusin={handleFocus}
	onfocusout={handleBlur}
>
	{#if children}{@render children()}{/if}
</div>

{#if visible}
	<div
		bind:this={tooltipEl}
		role="tooltip"
		class={`${maxWidth} bg-bg-elevated border border-border text-sm px-3 py-2 rounded-lg shadow-lg z-50 pointer-events-none text-center`}
	>
		{content}
	</div>
{/if}
