<script lang="ts">
	import { animate } from "motion";
	import { onMount } from "svelte";

	let { 
		initial = { opacity: 0, y: 20 },
		animate: target = { opacity: 1, y: 0 },
		duration = 0.5,
		delay = 0,
		children
	}: {
		initial?: Record<string, number>;
		animate?: Record<string, number>;
		duration?: number;
		delay?: number;
		children?: any;
	} = $props();

	let el: HTMLElement;

	onMount(() => {
		if (el) {
			animate(el, target, { duration, delay });
		}
	});
</script>

<div bind:this={el} style="opacity: 0;">
	{@render children?.()}
</div>
