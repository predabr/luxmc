<script lang="ts">
	import { fade } from 'svelte/transition';
	import { Loader } from 'lucide-svelte';

	interface Props {
		isOpen: boolean;
		message?: string;
		progress?: number;
	}

	let { isOpen, message = 'Carregando...', progress }: Props = $props();
</script>

{#if isOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
		role="status"
		aria-live="polite"
		aria-label={message}
		transition:fade={{ duration: 200 }}
	>
		<div class="bg-bg-elevated border border-border rounded-lg p-8 flex flex-col items-center gap-4">
			<Loader class="h-8 w-8 text-brand animate-spin" />
			<p class="text-center fg">{message}</p>
			{#if progress !== undefined}
				<div class="w-32 h-1 bg-bg-hover rounded-full overflow-hidden">
					<div
						class="h-full bg-brand transition-all duration-300"
						style={`width: ${progress}%`}
					></div>
				</div>
				<p class="text-xs fg-muted">{progress}%</p>
			{/if}
		</div>
	</div>
{/if}
