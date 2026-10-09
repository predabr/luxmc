<script lang="ts">
	import { bedrock } from "$lib/stores/bedrock.svelte";
	import { bedrockCancelInstall } from "$lib/api/bedrock";
	import { button } from "$lib/components/ui/button";
	import { Blocks } from "lucide-svelte";
	const status = $derived(bedrock.progress?.phase === 'registering' ? 'Instalando pelo Windows' : bedrock.progress?.phase === 'backing-up' ? 'Preservando seus mundos' : 'Baixando Bedrock');
	const canCancel = $derived(!['registering','backing-up'].includes(bedrock.progress?.phase ?? ''));
</script>
{#if bedrock.installing}
	<aside aria-live="polite" class="fixed bottom-6 left-1/2 z-50 w-[min(540px,calc(100vw-32px))] -translate-x-1/2 rounded-2xl border border-border bg-bg-elevated p-4 shadow-xl">
		<div class="flex items-center gap-3"><Blocks class="h-5 w-5 shrink-0 text-brand-400" /><div class="min-w-0 flex-1"><p class="truncate text-sm font-semibold text-fg">{bedrock.installationName}</p><p class="text-xs text-fg-muted">{status}</p></div><button class={button({variant:'secondary',size:'sm'})} disabled={!canCancel} onclick={() => bedrockCancelInstall()}>Cancelar</button></div>
		<progress aria-label="Instalação Bedrock" class="mt-3 h-1.5 w-full accent-brand-500" max="100" value={bedrock.progress?.percent ?? 0}></progress>
	</aside>
{/if}
