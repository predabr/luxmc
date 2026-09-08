<script lang="ts">
	interface Props {
		items: Array<{ label: string; href?: string; active?: boolean }>;
		separator?: string;
	}

	let { items, separator = '/' }: Props = $props();
</script>

<nav class="flex items-center gap-2 text-sm">
	{#each items as item, i (i)}
		<div class="flex items-center gap-2">
			{#if item.href}
				<a href={item.href} class="hover:text-brand transition-colors fg-muted hover:fg">
					{item.label}
				</a>
			{:else}
				<span class={item.active ? 'fg-brand font-semibold' : 'fg-muted'}>
					{item.label}
				</span>
			{/if}
			{#if i < items.length - 1}
				<span class="fg-subtle">{separator}</span>
			{/if}
		</div>
	{/each}
</nav>
