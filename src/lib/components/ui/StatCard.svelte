<script lang="ts">
	interface Props {
		label: string;
		value: string | number;
		icon?: any;
		highlight?: boolean;
		trend?: 'up' | 'down' | 'stable';
		change?: string;
	}

	let { label, value, icon: Icon, highlight = false, trend, change }: Props = $props();
</script>

<div class={`p-4 rounded-lg border ${highlight ? 'border-brand bg-brand/5' : 'border-border bg-bg-elevated'} transition-all hover:shadow-lg`}>
	<div class="flex items-start justify-between">
		<div>
			<p class="text-sm font-medium fg-muted uppercase tracking-wide">{label}</p>
			<p class="text-2xl font-bold fg mt-2">{value}</p>
			{#if change}
				<p class={`text-xs mt-2 ${trend === 'up' ? 'text-green-500' : trend === 'down' ? 'text-red-500' : 'text-gray-500'}`}>
					{trend === 'up' ? '↑' : trend === 'down' ? '↓' : '→'} {change}
				</p>
			{/if}
		</div>
		{#if Icon}
			<div class={`p-3 rounded-lg ${highlight ? 'bg-brand/20 fg-brand' : 'bg-bg-subtle fg-muted'}`}>
				<Icon size={20} />
			</div>
		{/if}
	</div>
</div>
