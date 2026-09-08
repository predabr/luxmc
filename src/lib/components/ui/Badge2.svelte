<script lang="ts">
	interface Props {
		label: string;
		variant?: 'default' | 'success' | 'warning' | 'error' | 'info' | 'brand';
		size?: 'sm' | 'md' | 'lg';
		closable?: boolean;
		onClose?: () => void;
	}

	let { label, variant = 'default', size = 'sm', closable = false, onClose }: Props = $props();

	const variantClasses = {
		default: 'bg-bg-elevated border border-border text-fg',
		success: 'bg-green-500/20 border border-green-500/50 text-green-600 dark:text-green-400',
		warning: 'bg-yellow-500/20 border border-yellow-500/50 text-yellow-600 dark:text-yellow-400',
		error: 'bg-red-500/20 border border-red-500/50 text-red-600 dark:text-red-400',
		info: 'bg-blue-500/20 border border-blue-500/50 text-blue-600 dark:text-blue-400',
		brand: 'bg-brand text-white',
	};

	const sizeClasses = {
		sm: 'px-2 py-1 text-xs',
		md: 'px-3 py-1.5 text-sm',
		lg: 'px-4 py-2 text-base',
	};
</script>

<span class={`inline-flex items-center gap-2 rounded-full font-medium ${variantClasses[variant]} ${sizeClasses[size]} transition-all`}>
	{label}
	{#if closable}
		<button onclick={onClose} class="ml-1 hover:opacity-70">×</button>
	{/if}
</span>
