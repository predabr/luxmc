<script lang="ts">
	import ServerWorkspace from '$lib/components/server/ServerWorkspace.svelte';
</script>

<svelte:head><title>Gerenciador de servidor · Luxmc</title></svelte:head>

<div class="dedicated-stage mx-auto flex w-full max-w-[1400px] min-h-0 flex-1 flex-col pb-2">
	<ServerWorkspace />
</div>

<style>
	.dedicated-stage :global(.workspace) {
		flex: 1 1 auto;
		height: auto;
		min-height: 0;
	}
</style>
