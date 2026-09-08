<script lang="ts">
	import { Star, Heart } from 'lucide-svelte';

	interface Props {
		rating?: number;
		maxRating?: number;
		interactive?: boolean;
		size?: 'sm' | 'md' | 'lg';
		onRate?: (rating: number) => void;
	}

	let { rating = 0, maxRating = 5, interactive = false, size = 'md', onRate }: Props = $props();
	let hoverRating = $state(0);

	const sizeMap = {
		sm: 'w-4 h-4',
		md: 'w-5 h-5',
		lg: 'w-6 h-6',
	};
</script>

<div class="flex gap-1">
	{#each Array.from({ length: maxRating }) as _, i}
		<button
			class={`transition-all ${sizeMap[size]}`}
			onmouseenter={() => (hoverRating = i + 1)}
			onmouseleave={() => (hoverRating = 0)}
			onclick={() => onRate?.(i + 1)}
			disabled={!interactive}
		>
			<Star
				size={16}
				class={`w-full h-full transition-all ${
					(interactive && hoverRating > i) || (!interactive && rating > i)
						? 'fill-yellow-400 text-yellow-400'
						: 'text-fg-muted'
				}`}
			/>
		</button>
	{/each}
</div>
