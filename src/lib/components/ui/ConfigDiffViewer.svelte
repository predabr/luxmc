<script lang="ts">
	import { diffLines, type Change } from "diff";

	type Props = {
		oldText: string;
		newText: string;
		oldTitle?: string;
		newTitle?: string;
	};

	let { oldText, newText, oldTitle = "Original", newTitle = "Modificado" }: Props = $props();

	let diffs = $derived<Change[]>(diffLines(oldText, newText));
</script>

<div class="flex flex-col gap-2 rounded-2xl border border-white/10 bg-bg-subtle p-4 font-mono text-xs overflow-hidden">
	<div class="flex items-center justify-between pb-2 border-b border-white/10 text-white/60 font-sans font-semibold">
		<span>{oldTitle} vs {newTitle}</span>
		<div class="flex items-center gap-3 text-[11px]">
			<span class="flex items-center gap-1 text-emerald-400">
				<span class="w-2 h-2 rounded-full bg-emerald-400"></span> Adicionado
			</span>
			<span class="flex items-center gap-1 text-rose-400">
				<span class="w-2 h-2 rounded-full bg-rose-400"></span> Removido
			</span>
		</div>
	</div>

	<div class="max-h-72 overflow-y-auto custom-scrollbar flex flex-col">
		{#each diffs as part}
			<div 
				class="px-2 py-0.5 whitespace-pre-wrap transition-colors {part.added ? 'bg-emerald-500/20 text-emerald-300' : part.removed ? 'bg-rose-500/20 text-rose-300' : 'text-white/80'}"
			>
				{part.added ? "+ " : part.removed ? "- " : "  "}{part.value}
			</div>
		{/each}
	</div>
</div>
