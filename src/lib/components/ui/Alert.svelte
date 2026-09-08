<script lang="ts">
	import { fade } from 'svelte/transition';

	interface Props {
		title?: string;
		description?: string;
		variant?: 'info' | 'success' | 'warning' | 'error';
		dismissible?: boolean;
		onDismiss?: () => void;
	}

	let { title, description, variant = 'info', dismissible = true, onDismiss }: Props = $props();
	let visible = $state(true);

	const bgMap = {
		info: 'bg-blue-500/10 border-blue-500/30',
		success: 'bg-green-500/10 border-green-500/30',
		warning: 'bg-yellow-500/10 border-yellow-500/30',
		error: 'bg-red-500/10 border-red-500/30',
	};

	const textMap = {
		info: 'text-blue-600 dark:text-blue-400',
		success: 'text-green-600 dark:text-green-400',
		warning: 'text-yellow-600 dark:text-yellow-400',
		error: 'text-red-600 dark:text-red-400',
	};

	const handleDismiss = () => {
		visible = false;
		onDismiss?.();
	};
</script>

{#if visible}
	<div
		class={`p-4 rounded-lg border ${bgMap[variant]} flex items-start gap-3`}
		transition:fade={{ duration: 200 }}
	>
		<div class="flex-1">
			{#if title}
				<p class={`font-semibold ${textMap[variant]}`}>{title}</p>
			{/if}
			{#if description}
				<p class="text-sm fg-muted mt-1">{description}</p>
			{/if}
		</div>
		{#if dismissible}
			<button
				type="button"
				aria-label="Dismiss"
				onclick={handleDismiss}
				class="flex-shrink-0 fg-muted hover:fg transition-colors"
			>
				<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
				</svg>
			</button>
		{/if}
	</div>
{/if}
