<script lang="ts">
	interface Props {
		value: number;
		max?: number;
		animated?: boolean;
		variant?: 'success' | 'warning' | 'error' | 'primary';
		size?: 'sm' | 'md' | 'lg';
	}

	let {
		value,
		max = 100,
		animated = true,
		variant = 'primary',
		size = 'md',
	}: Props = $props();

	const sizeClasses: Record<NonNullable<Props['size']>, string> = {
		sm: 'h-1',
		md: 'h-2',
		lg: 'h-3',
	};

	const variantClasses: Record<NonNullable<Props['variant']>, string> = {
		success: 'bg-gradient-to-r from-green-500 to-emerald-500',
		warning: 'bg-gradient-to-r from-yellow-500 to-orange-500',
		error: 'bg-gradient-to-r from-red-500 to-pink-500',
		primary: 'bg-gradient-to-r from-blue-500 to-cyan-500',
	};

	const percentage = $derived(Math.min((value / max) * 100, 100));
</script>

<div class={`w-full bg-bg-subtle rounded-full overflow-hidden ${sizeClasses[size]}`}>
	<div
		class={`h-full ${variantClasses[variant]} rounded-full transition-all duration-300 ${
			animated ? 'animate-pulse' : ''
		}`}
		style={`width: ${percentage}%`}
	></div>
</div>
