<script lang="ts">
	import { onMount } from "svelte";
	import { bedrock } from "$lib/stores/bedrock.svelte";
	import { bedrockOpen, bedrockRemove } from "$lib/api/bedrock";
	import { button } from "$lib/components/ui/button";
	import { Blocks, ExternalLink, Trash2 } from "lucide-svelte";
	let busy = $state("");
	let error = $state("");
	let removing = $state("");
	let notice = $state("");
	onMount(() => { void bedrock.refresh(); });
	async function run(id: string, action: () => Promise<unknown>) {
		busy = id;
		error = "";
		try { await action(); } catch (failure) { error = String(failure); }
		finally { busy = ""; }
	}
	async function openInstance(id: string) {
		const result = await bedrockOpen(id);
		notice = result.notice ?? "";
	}
</script>

{#if bedrock.value?.instances.length}
	<section class="space-y-4" aria-label="Instâncias Bedrock">
		<div class="flex items-center justify-between gap-3"><h2 class="text-lg font-semibold text-fg">Bedrock Edition</h2>{#if bedrock.value.provider}<button class={button({ variant: "secondary", size: "sm" })} disabled={!!busy} onclick={() => run("provider", () => bedrockOpen())}><ExternalLink class="mr-2 h-4 w-4" />Abrir provedor</button>{/if}</div>
		<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
			{#each bedrock.value.instances as instance (instance.id)}
				{@const installation = bedrock.value.installations.find((item) => item.profileId === instance.profileId && item.id === instance.installationId)}
				<article class="rounded-3xl border border-border bg-bg/35 p-5 backdrop-blur-xl">
					<Blocks class="mb-4 h-10 w-10 text-brand-500" /><h3 class="font-semibold text-fg">{instance.name}</h3><p class="mt-1 text-sm text-fg-muted">{installation?.name ?? "Instalação indisponível no provedor"}</p>
					{#if instance.profileId==='managed'}<p class="mt-2 text-xs text-fg-muted">Versão {installation?.version} · Instalada pelo Luxmc</p>{/if}
					<div class="mt-5 flex gap-2"><button class={button({ variant: "primary", size: "sm" })} disabled={!!busy || !installation || bedrock.installing} onclick={() => run(instance.id, () => openInstance(instance.id))}>{['windows','managed'].includes(instance.profileId) ? "Jogar Bedrock" : "Jogar versão importada"}</button><button class={button({ variant: "secondary", size: "icon" })} aria-label={`Remover vínculo de ${instance.name}`} disabled={!!busy} onclick={() => removing = instance.id}><Trash2 class="h-4 w-4" /></button></div>
					{#if removing === instance.id}<div class="mt-4 space-y-2 text-sm text-fg-muted"><p>Remover apenas o vínculo? Os mundos e arquivos do jogo serão preservados.</p><button class={button({ variant: "secondary", size: "sm" })} disabled={!!busy} onclick={() => run(instance.id, async () => { await bedrockRemove(instance.id); await bedrock.refresh(); removing = ""; })}>Remover vínculo</button><button class={button({ variant: "ghost", size: "sm" })} onclick={() => removing = ""}>Cancelar</button></div>{/if}
				</article>
			{/each}
		</div>
		{#if error}<p role="alert" class="rounded-xl border border-danger/30 bg-danger/5 p-3 text-sm text-danger">{error}</p>{/if}
		{#if notice}<p role="status" class="rounded-xl border border-warning/30 bg-warning/5 p-3 text-sm text-warning">{notice}</p>{/if}
	</section>
{/if}
