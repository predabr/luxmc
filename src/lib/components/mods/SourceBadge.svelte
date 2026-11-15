<script lang="ts">
	import ProviderLogo from "./ProviderLogo.svelte";

	let { source, class: klass = "" }: { source: string; class?: string } = $props();
</script>

{#if source === "modrinth"}
	<div class="source-badge h-6 w-6 rounded-full bg-bg-elevated border border-fg/10 flex items-center justify-center shadow-md {klass}" title="Modrinth">
		<ProviderLogo source={source} class="h-3.5 w-3.5 text-success" />
	</div>
{:else if source === "curseforge"}
	<div class="source-badge h-6 w-6 rounded-full bg-bg-elevated border border-fg/10 flex items-center justify-center shadow-md {klass}" title="CurseForge">
		<ProviderLogo source={source} class="h-3.5 w-3.5 text-warning" />
	</div>
{:else if source === "ftb"}
	<div class="source-badge h-6 w-6 rounded-full bg-bg-elevated border border-fg/10 flex items-center justify-center shadow-md {klass}" title="Feed The Beast">
		<ProviderLogo source={source} class="h-3.5 w-3.5 text-danger" />
	</div>
{:else}
	<div class="source-badge h-6 w-6 rounded-full bg-bg-elevated border border-fg/10 flex items-center justify-center shadow-md {klass}" title={source}>
		<span class="font-black text-[11px] text-brand-400">?</span>
	</div>
{/if}

<style>
	.source-badge { flex: none; }
</style>
