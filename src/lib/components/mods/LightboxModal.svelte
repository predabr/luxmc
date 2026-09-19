<script lang="ts">
	import { X } from "lucide-svelte";

	let {
		imageUrl,
		title = null,
		description = null,
		onClose
	}: {
		imageUrl: string;
		title?: string | null;
		description?: string | null;
		onClose: () => void;
	} = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-[999999] bg-bg-overlay/90 backdrop-blur-md flex flex-col items-center justify-center p-6 cursor-pointer"
	onclick={onClose}
>
	<div class="relative max-w-5xl max-h-[85vh] flex flex-col items-center" onclick={(e) => e.stopPropagation()}>
		<button
			type="button"
			class="absolute -top-12 right-0 p-2 text-fg/60 hover:text-fg bg-fg/10 rounded-full transition-all cursor-pointer"
			onclick={onClose}
		>
			<X class="w-5 h-5" />
		</button>
		<img src={imageUrl} alt={title || "Screenshot"} class="max-w-full max-h-[75vh] object-contain rounded-2xl shadow-2xl border border-fg/10" />
		{#if title}
			<div class="mt-4 text-center">
				<h3 class="text-sm font-bold text-fg">{title}</h3>
				{#if description}
					<p class="text-xs text-fg/60 mt-1 max-w-xl">{description}</p>
				{/if}
			</div>
		{/if}
	</div>
</div>
