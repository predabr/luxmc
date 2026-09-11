<script lang="ts">
	import { Flame } from "lucide-svelte";

	let { source, class: klass = "" }: { source: string; class?: string } = $props();
</script>

{#if source === "modrinth"}
	<div class="h-6 w-6 rounded-full bg-black/70 backdrop-blur-sm border border-white/10 flex items-center justify-center shadow-md {klass}" title="Modrinth">
		<span class="font-black text-[11px] text-[#1bd96a]">m</span>
	</div>
{:else if source === "curseforge"}
	<div class="h-6 w-6 rounded-full bg-black/70 backdrop-blur-sm border border-white/10 flex items-center justify-center shadow-md {klass}" title="CurseForge">
		<Flame class="w-3 h-3 text-[#f16436]" />
	</div>
{:else}
	<div class="h-6 w-6 rounded-full bg-black/70 backdrop-blur-sm border border-white/10 flex items-center justify-center shadow-md {klass}" title={source}>
		<span class="font-black text-[11px] text-[#a29bfe]">?</span>
	</div>
{/if}
