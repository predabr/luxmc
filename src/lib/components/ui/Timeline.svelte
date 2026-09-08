<script lang="ts">
	interface TimelineItem {
		id: string;
		title: string;
		description?: string;
		time: string;
		icon?: any;
		color?: 'primary' | 'success' | 'warning' | 'error';
	}

	interface Props {
		items: TimelineItem[];
	}

	let { items }: Props = $props();

	const colorMap: Record<NonNullable<TimelineItem['color']>, string> = {
		primary: 'bg-brand',
		success: 'bg-green-500',
		warning: 'bg-yellow-500',
		error: 'bg-red-500',
	};
</script>

<div class="space-y-4">
	{#each items as item, idx (item.id)}
		<div class="flex gap-4">
			<div class="flex flex-col items-center">
				<div class={`w-10 h-10 rounded-full ${colorMap[item.color ?? 'primary']} flex items-center justify-center text-white`}>
					{#if item.icon}
						<item.icon size={20} />
					{:else}
						<span class="text-sm font-bold">{idx + 1}</span>
					{/if}
				</div>
				{#if idx < items.length - 1}
					<div class={`w-1 h-8 ${colorMap[item.color ?? 'primary']}/30`}></div>
				{/if}
			</div>
			<div class="flex-1 pt-1">
				<p class="font-semibold">{item.title}</p>
				{#if item.description}
					<p class="text-sm fg-muted">{item.description}</p>
				{/if}
				<p class="text-xs fg-subtle mt-1">{item.time}</p>
			</div>
		</div>
	{/each}
</div>
