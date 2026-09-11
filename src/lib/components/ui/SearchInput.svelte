<script lang="ts">
	import { Search, X } from 'lucide-svelte';
	import { useTranslation } from '$lib/i18n/useTranslation.svelte';

	const { t } = useTranslation();

	interface Props {
		placeholder?: string;
		value?: string;
		onChange?: (value: string) => void;
		onSearch?: (value: string) => void;
		icon?: any;
		size?: 'sm' | 'md' | 'lg';
		clearable?: boolean;
	}

	let {
		placeholder,
		value = '',
		onChange,
		onSearch,
		icon: Icon = Search,
		size = 'md',
		clearable = true,
	}: Props = $props();

	const sizeClasses = {
		sm: 'px-3 py-1.5 text-sm',
		md: 'px-4 py-2 text-base',
		lg: 'px-5 py-3 text-lg',
	};

	const handleClear = () => {
		onChange?.('');
	};
</script>

<div class="relative">
	<div class="absolute left-3 top-1/2 -translate-y-1/2 text-fg-muted pointer-events-none">
		<Icon size={20} />
	</div>
	<input
		type="text"
		placeholder={placeholder ?? t("common.search")}
		value={value}
		oninput={(e) => onChange?.(e.currentTarget.value)}
		onkeydown={(e) => e.key === 'Enter' && onSearch?.(value)}
		class={`w-full pl-10 pr-10 rounded-lg border border-border bg-bg-elevated focus:outline-none focus:ring-2 focus:ring-brand/50 transition-all ${sizeClasses[size]}`}
	/>
	{#if clearable && value}
		<button
			onclick={handleClear}
			class="absolute right-3 top-1/2 -translate-y-1/2 p-1 hover:bg-bg-hover rounded transition-colors"
			aria-label={t("common.clear")}
		>
			<X size={18} class="text-fg-muted" />
		</button>
	{/if}
</div>
