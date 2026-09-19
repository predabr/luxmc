<script lang="ts">
	interface Props {
		message: string;
		type?: 'success' | 'error' | 'info' | 'warning';
		duration?: number;
		onClose?: () => void;
	}

	let { message, type = 'info', duration = 3000, onClose }: Props = $props();
	let visible = $state(true);

	const typeClasses: Record<NonNullable<Props['type']>, string> = {
		success: 'bg-green-500/90 text-fg',
		error: 'bg-red-500/90 text-fg',
		info: 'bg-blue-500/90 text-fg',
		warning: 'bg-yellow-500/90 text-fg',
	};

	$effect(() => {
		const ms = duration;
		if (ms > 0) {
			const id = setTimeout(() => {
				visible = false;
				onClose?.();
			}, ms);
			return () => clearTimeout(id);
		}
	});
</script>

{#if visible}
	<div class={`fixed bottom-4 right-4 px-6 py-3 rounded-lg shadow-lg ${typeClasses[type]} animate-slide-in z-50`} role="status" aria-live="polite">
		{message}
	</div>
{/if}
