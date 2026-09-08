<script lang="ts">
	import { twMerge } from "tailwind-merge";

	type Props = {
		class?: string;
		lines?: number;
	};

	let { class: klass = "", lines = 1 }: Props = $props();
</script>

<div class={twMerge("animate-pulse space-y-2", klass)}>
	{#each Array(lines) as _, i}
		<div
			class="h-4 rounded-md bg-bg-subtle"
			style="width: {i === lines - 1 ? '60%' : '100%'}"
		></div>
	{/each}
</div>
