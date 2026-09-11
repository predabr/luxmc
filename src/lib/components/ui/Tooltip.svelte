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

	const positionClasses: Record<string, string> = {
		top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
		bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
		left: 'right-full top-1/2 -translate-y-1/2 mr-2',
		right: 'left-full top-1/2 -translate-y-1/2 ml-2',
	};

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
	class="relative inline-flex"
	onmouseenter={handleMouseEnter}
	onmouseleave={handleMouseLeave}
	onfocusin={handleFocus}
	onfocusout={handleBlur}
	role="presentation"
>
	{#if children}{@render children()}{/if}

	{#if visible}
		<div
			role="tooltip"
			class="absolute {positionClasses[position]} {maxWidth} bg-bg-elevated border border-border text-sm px-3 py-2 rounded-lg shadow-lg z-50 pointer-events-none text-center whitespace-nowrap"
		>
			{content}
		</div>
	{/if}
</div>
