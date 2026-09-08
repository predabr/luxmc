<script lang="ts">
	import { Check, AlertCircle } from 'lucide-svelte';

	interface Props {
		label: string;
		checked?: boolean;
		onChange?: (checked: boolean) => void;
		disabled?: boolean;
		size?: 'sm' | 'md' | 'lg';
		indeterminate?: boolean;
	}

	let { label, checked = false, onChange, disabled = false, size = 'md', indeterminate = false }: Props = $props();

	const sizeClasses = {
		sm: 'w-4 h-4',
		md: 'w-5 h-5',
		lg: 'w-6 h-6',
	};
</script>

<label class="flex items-center gap-3 cursor-pointer" class:opacity-50={disabled} class:cursor-not-allowed={disabled}>
	<div class={`relative flex items-center justify-center ${sizeClasses[size]} rounded border-2 transition-all ${
		checked ? 'bg-brand border-brand' : 'border-border hover:border-brand'
	}`}>
		<input
			type="checkbox"
			checked={checked}
			onchange={(e) => !disabled && onChange?.(e.currentTarget.checked)}
			disabled={disabled}
			class="sr-only"
		/>
		{#if checked || indeterminate}
			<Check size={16} class="text-white" />
		{/if}
	</div>
	<span class="select-none">{label}</span>
</label>
