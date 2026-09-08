<script lang="ts">
	import type { Snippet } from "svelte";

	interface Tab {
		id: string;
		label: string;
		icon?: any;
	}

	interface Props {
		tabs: Tab[];
		active?: string;
		onChange?: (id: string) => void;
		size?: 'sm' | 'md' | 'lg';
		variant?: 'line' | 'pill' | 'box';
		children?: Snippet;
	}

	let { tabs, active = tabs[0]?.id, onChange, size = 'md', variant = 'line', children }: Props = $props();

	const sizeClasses: Record<NonNullable<Props['size']>, string> = {
		sm: 'text-sm px-3 py-1.5',
		md: 'text-base px-4 py-2',
		lg: 'text-lg px-6 py-3',
	};
</script>

<div class="flex gap-1 border-b border-border" role="tablist">
	{#each tabs as tab (tab.id)}
		<button
			type="button"
			role="tab"
			aria-selected={active === tab.id}
			onclick={() => onChange?.(tab.id)}
			class={`flex items-center gap-2 font-medium transition-all ${sizeClasses[size]} ${
				active === tab.id
					? 'text-brand border-b-2 border-brand -mb-[2px]'
					: 'text-fg-muted hover:text-fg'
			}`}
		>
			{#if tab.icon}
				<tab.icon size={18} />
			{/if}
			{tab.label}
		</button>
	{/each}
</div>

<div class="mt-4">
	{#if children}{@render children()}{/if}
</div>
