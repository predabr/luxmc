<script lang="ts">
	import { fade, slide } from 'svelte/transition';
	import { X } from 'lucide-svelte';
	import type { Snippet } from 'svelte';
	import { useTranslation } from '$lib/i18n/useTranslation.svelte';

	const { t } = useTranslation();

	interface Props {
		isOpen: boolean;
		onClose: () => void;
		title: string;
		maxWidth?: string;
		showClose?: boolean;
		children?: Snippet;
	}

	let { isOpen, onClose, title, maxWidth = 'max-w-md', showClose = true, children }: Props = $props();

	const titleId = $derived(`modal-title-${title.replace(/\s+/g, '-').toLowerCase()}`);

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) onClose();
	}

	function handleBackdropKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}
</script>

{#if isOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		role="dialog"
		aria-modal="true"
		aria-labelledby={titleId}
		tabindex="-1"
		onclick={handleBackdropClick}
		onkeydown={handleBackdropKeydown}
		transition:fade={{ duration: 200 }}
	>
		<div
			class={`bg-bg-elevated border border-border rounded-lg shadow-2xl ${maxWidth} w-full mx-4`}
			transition:slide={{ duration: 300 }}
		>
			<div class="flex items-center justify-between border-b border-border p-6">
				<h2 id={titleId} class="text-xl font-bold">{title}</h2>
				{#if showClose}
					<button
						type="button"
						aria-label={t("common.close")}
						onclick={onClose}
						class="rounded-lg p-2 transition-colors hover:bg-bg-hover"
					>
						<X size={20} />
					</button>
				{/if}
			</div>
			<div class="p-6">
				{#if children}{@render children()}{/if}
			</div>
		</div>
	</div>
{/if}
