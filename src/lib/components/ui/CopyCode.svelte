<script lang="ts">
	import { Copy, Check } from 'lucide-svelte';
	import { useTranslation } from '$lib/i18n/useTranslation.svelte';

	const { t } = useTranslation();

	interface Props {
		value: string;
		label?: string;
		variant?: 'default' | 'code';
	}

	let { value, label, variant = 'default' }: Props = $props();
	let copied = $state(false);

	const handleCopy = async () => {
		await navigator.clipboard.writeText(value);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	};
</script>

<div class="space-y-2">
	{#if label}
		<p class="text-sm font-medium fg-muted uppercase tracking-wide">{label}</p>
	{/if}
	<div class={`flex items-center justify-between p-3 rounded-lg border border-border transition-all ${
		variant === 'code' ? 'bg-black/20 font-mono text-sm' : 'bg-bg-elevated'
	}`}>
		<span class="truncate fg-muted">{value}</span>
		<button
			onclick={handleCopy}
			class="flex-shrink-0 ml-2 p-2 hover:bg-bg-hover rounded transition-colors"
			title={copied ? t("common.copied") : t("common.copy")}
			aria-label={copied ? t("common.copied") : t("common.copy")}
		>
			{#if copied}
				<Check size={18} class="text-green-500" />
			{:else}
				<Copy size={18} class="fg-muted hover:fg" />
			{/if}
		</button>
	</div>
</div>
