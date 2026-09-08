<script lang="ts">
	import { ChevronUp, ChevronDown } from 'lucide-svelte';
	import { slide } from 'svelte/transition';
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		defaultOpen?: boolean;
		icon?: any;
		children?: Snippet;
	}

	// svelte-ignore state_referenced_locally
	let { title, defaultOpen = false, icon: Icon, children }: Props = $props();
	// svelte-ignore state_referenced_locally
	let isOpen = $state(defaultOpen);
</script>

<div class="border border-border rounded-lg overflow-hidden">
	<button
		type="button"
		aria-expanded={isOpen}
		onclick={() => (isOpen = !isOpen)}
		class="w-full flex items-center justify-between p-4 hover:bg-bg-hover transition-colors"
	>
		<div class="flex items-center gap-3">
			{#if Icon}
				<Icon size={20} />
			{/if}
			<span class="font-semibold">{title}</span>
		</div>
		<ChevronDown size={20} class={`transition-transform ${isOpen ? 'rotate-180' : ''}`} />
	</button>

	{#if isOpen}
		<div class="p-4 border-t border-border" transition:slide={{ duration: 200 }}>
			{#if children}{@render children()}{/if}
		</div>
	{/if}
</div>
