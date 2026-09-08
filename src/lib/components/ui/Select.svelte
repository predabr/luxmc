<script lang="ts">
	import { ChevronDown } from 'lucide-svelte';

	interface Option {
		value: string | number;
		label: string;
		disabled?: boolean;
	}

	interface Props {
		options: Option[];
		value?: string | number;
		placeholder?: string;
		onChange?: (value: string | number) => void;
		disabled?: boolean;
		size?: 'sm' | 'md' | 'lg';
	}

	let { options, value, placeholder = 'Selecione...', onChange, disabled = false, size = 'md' }: Props = $props();
	let isOpen = $state(false);

	const sizeClasses: Record<NonNullable<Props['size']>, string> = {
		sm: 'px-3 py-1.5 text-sm',
		md: 'px-4 py-2 text-base',
		lg: 'px-5 py-3 text-lg',
	};

	const selectedLabel = $derived(
		options.find((opt) => opt.value === value)?.label ?? placeholder,
	);
</script>

<div class="relative">
	<button
		type="button"
		aria-haspopup="listbox"
		aria-expanded={isOpen}
		onclick={() => (isOpen = !isOpen)}
		disabled={disabled}
		class={`w-full flex items-center justify-between border border-border rounded-lg bg-bg-elevated hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${sizeClasses[size]}`}
	>
		<span class={value ? 'fg' : 'fg-muted'}>{selectedLabel}</span>
		<ChevronDown size={18} class={`transition-transform ${isOpen ? 'rotate-180' : ''}`} />
	</button>

	{#if isOpen}
		<div class="absolute top-full left-0 right-0 mt-1 bg-bg-elevated border border-border rounded-lg shadow-lg z-50" role="listbox">
			{#each options as option (option.value)}
				<button
					type="button"
					role="option"
					aria-selected={value === option.value}
					onclick={() => {
						if (!option.disabled) {
							onChange?.(option.value);
							isOpen = false;
						}
					}}
					class={`w-full text-left px-4 py-2 hover:bg-bg-hover transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
						value === option.value ? 'bg-brand/10 text-brand font-medium' : 'fg'
					}`}
					disabled={option.disabled}
				>
					{option.label}
				</button>
			{/each}
		</div>
	{/if}
</div>
