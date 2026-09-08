<script lang="ts">
	interface Props {
		items: Array<{
			id: string;
			label: string;
			icon?: any;
			badge?: string | number;
			onClick?: () => void;
			active?: boolean;
		}>;
		vertical?: boolean;
	}

	let { items, vertical = false }: Props = $props();
</script>

<nav class={`flex ${vertical ? 'flex-col' : 'flex-row'} gap-1`}>
	{#each items as item (item.id)}
		<button
			type="button"
			aria-current={item.active ? "page" : undefined}
			aria-label={item.label}
			onclick={item.onClick}
			class={`flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-all relative ${
				item.active
					? 'bg-brand text-white shadow-lg'
					: 'hover:bg-bg-hover text-fg-muted hover:text-fg'
			}`}
		>
			{#if item.icon}
				<item.icon size={20} />
			{/if}
			<span>{item.label}</span>
			{#if item.badge}
				<span class="ml-2 px-2 py-0.5 bg-red-500 text-white text-xs rounded-full font-bold">
					{item.badge}
				</span>
			{/if}
		</button>
	{/each}
</nav>
