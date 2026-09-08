<script lang="ts">
	interface Props {
		items: Array<{
			id: string;
			name: string;
			avatar?: string;
			status?: 'online' | 'offline' | 'away';
			action?: () => void;
		}>;
	}

	let { items }: Props = $props();

	const statusColors: Record<NonNullable<Props['items'][number]['status']>, string> = {
		online: 'bg-green-500',
		offline: 'bg-gray-500',
		away: 'bg-yellow-500',
	};
</script>

<div class="space-y-2">
	{#each items as item (item.id)}
		<div
			class="p-3 rounded-lg bg-bg-elevated border border-border hover:border-brand transition-all flex items-center justify-between cursor-pointer"
			role="button"
			tabindex="0"
			onclick={item.action}
			onkeydown={(e) => {
				if ((e.key === 'Enter' || e.key === ' ') && item.action) {
					e.preventDefault();
					item.action();
				}
			}}
		>
			<div class="flex items-center gap-3">
				<div class="relative">
					<img
						src={item.avatar}
						alt={item.name}
						class="w-10 h-10 rounded-full bg-brand/20"
						onerror={(e) => {
							const target = e.target as HTMLImageElement;
							target.src = 'data:image/svg+xml,<svg></svg>';
						}}
					/>
					{#if item.status}
						<div
							class={`absolute bottom-0 right-0 w-3 h-3 rounded-full border-2 border-bg-elevated ${statusColors[item.status]}`}
							aria-label={item.status}
						></div>
					{/if}
				</div>
				<div>
					<p class="font-medium fg">{item.name}</p>
					{#if item.status}
						<p class="text-xs fg-muted capitalize">{item.status === 'online' ? '🟢 Online' : item.status === 'away' ? '🟡 Away' : '⚪ Offline'}</p>
					{/if}
				</div>
			</div>
		</div>
	{/each}
</div>
