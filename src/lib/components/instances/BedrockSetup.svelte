<script lang="ts">
	import { onMount } from "svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { bedrockConnect, bedrockAdd, bedrockOpen, bedrockVersions, type BedrockVersion } from "$lib/api/bedrock";
	import { bedrock } from "$lib/stores/bedrock.svelte";
	import { button } from "$lib/components/ui/button";
	import { Blocks, FolderOpen, Plus, RefreshCw, ExternalLink, LoaderCircle, Search } from "lucide-svelte";

	let { onCreated }: { onCreated?: () => void } = $props();
	let busy = $state(false);
	let failure = $state("");
	let selected = $state("");
	let name = $state("");
	let legacy = $state(false);
	let versions = $state<BedrockVersion[]>([]);
	let loadingVersions = $state(true);
	let versionError = $state("");
	let channel = $state("release");
	let remoteVersion = $state("");
	let search = $state("");
	const available = $derived(versions.filter(version => version.channel === channel));
	const filtered = $derived(available.filter(version => version.version.toLowerCase().includes(search.trim().toLowerCase())));
	const remoteChosen = $derived(available.find(version => version.id === remoteVersion));
	const items = $derived(bedrock.value?.installations.filter((item) =>
		!bedrock.value?.instances.some((instance) => instance.installationId === item.id && instance.profileId === item.profileId)) ?? []);
	const chosen = $derived(items.find((item) => JSON.stringify([item.profileId, item.id]) === selected));
	async function loadVersions() {
		loadingVersions = true; versionError = "";
		try { versions = await bedrockVersions() ?? []; } catch(error) { versionError = String(error); }
		finally { loadingVersions = false; }
	}
	$effect(() => { if (!available.some(version => version.id === remoteVersion)) remoteVersion = available[0]?.id ?? ""; });
	onMount(() => { void bedrock.refresh(); void loadVersions(); });

	async function run(action: () => Promise<unknown>) {
		busy = true;
		failure = "";
		try { await action(); } catch (error) { failure = String(error); }
		finally { busy = false; }
	}

	async function connect() {
		const executable = await open({ title: "Selecionar BedrockLauncher.exe", multiple: false, filters: [{ name: "BedrockLauncher", extensions: ["exe"] }] });
		if (typeof executable !== "string") return;
		const directory = await open({ title: "Pasta do BedrockLauncher que contém user_profile.json", directory: true, multiple: false });
		if (typeof directory !== "string") return;
		await bedrockConnect(executable, directory);
		await bedrock.refresh();
	}

	async function create() {
		if (!chosen) return;
		await bedrockAdd(name.trim() || chosen.name, chosen.profileId, chosen.id);
		await bedrock.refresh();
		onCreated?.();
	}
</script>

<div class="space-y-5">
	<div class="flex gap-4 rounded-2xl border border-brand-500/20 bg-brand-500/5 p-5">
		<Blocks class="h-8 w-8 shrink-0 text-brand-500" />
		<div><h4 class="font-semibold text-fg">Minecraft Bedrock</h4><p class="mt-1 text-sm leading-relaxed text-fg-muted">Escolha uma versão e instale pelo Luxmc. O Windows confere o pacote e a licença Microsoft do jogo.</p></div>
	</div>
	{#if bedrock.value && !bedrock.value.supported}
		<p role="status" class="text-sm text-fg-muted">Minecraft Bedrock para PC está disponível no Windows.</p>
	{:else}
		<section class="space-y-4 rounded-2xl border border-border bg-bg-elevated p-4">
			<div class="flex items-center justify-between gap-3"><h5 class="text-sm font-semibold text-fg">Versões do Bedrock <span class="ml-2 text-xs font-normal text-fg-muted">{available.length} disponíveis</span></h5><button aria-label="Atualizar catálogo Bedrock" class={button({variant:'ghost',size:'icon'})} disabled={loadingVersions || bedrock.installing} onclick={loadVersions}><RefreshCw class="h-4 w-4 {loadingVersions ? 'animate-spin' : ''}" /></button></div>
			<div class="flex gap-2" role="group" aria-label="Canal do Bedrock">{#each [['release','Estáveis'],['preview','Preview'],['beta','Betas']] as option}<button class={button({variant:channel===option[0]?'primary':'secondary',size:'sm'})} aria-pressed={channel===option[0]} disabled={bedrock.installing} onclick={() => channel=option[0]}>{option[1]}</button>{/each}</div>
			{#if loadingVersions}<p role="status" class="flex items-center gap-2 text-sm text-fg-muted"><LoaderCircle class="h-4 w-4 animate-spin" />Carregando versões…</p>{:else if versionError}<p role="alert" class="text-sm text-danger">{versionError}</p>{:else if available.length}
				<label class="relative block"><span class="sr-only">Buscar versão Bedrock</span><Search class="pointer-events-none absolute left-3 top-3 h-4 w-4 text-fg-muted" /><input aria-label="Buscar versão Bedrock" class="w-full rounded-xl border border-border bg-bg-subtle py-2.5 pl-10 pr-3 text-sm" bind:value={search} placeholder="Buscar versão, por exemplo 1.20" disabled={bedrock.installing} /></label>
				<div role="listbox" aria-label="Versões disponíveis do Bedrock" class="max-h-48 overflow-y-auto rounded-xl border border-border bg-bg-subtle p-1">{#each filtered as version (version.id)}<button role="option" aria-selected={remoteVersion === version.id} class="flex w-full items-center justify-between gap-3 rounded-lg px-3 py-2.5 text-left text-sm transition-colors {remoteVersion === version.id ? 'bg-brand-500/10 text-brand-400' : 'text-fg hover:bg-fg/5'}" disabled={bedrock.installing} onclick={() => remoteVersion = version.id}><span class="font-medium">{version.version}</span><span class="text-xs text-fg-muted">{version.packageType}</span></button>{:else}<p class="p-3 text-sm text-fg-muted">Nenhuma versão corresponde à busca.</p>{/each}</div>
				<label class="block space-y-2 text-sm text-fg">Nome da instância<input aria-label="Nome da instância Bedrock" class="w-full rounded-xl border border-border bg-bg-subtle p-3" bind:value={name} maxlength="80" placeholder={`Bedrock ${remoteChosen?.version ?? ''}`} disabled={bedrock.installing} /></label>
				<p class="text-xs leading-relaxed text-fg-muted">Contas offline do Luxmc podem organizar a biblioteca. Para jogar, sua conta da Microsoft Store precisa possuir Minecraft. A troca de versão faz backup dos dados existentes; os mundos continuam no local usado pelo Windows.</p>
				<button class={button({variant:'primary'})} disabled={busy || bedrock.installing || !remoteChosen} aria-busy={busy || bedrock.installing} onclick={() => run(async () => { if (!remoteChosen) return; await bedrock.install(remoteChosen.id,name.trim()||`Bedrock ${remoteChosen.version}`); onCreated?.(); })}>{#if busy || bedrock.installing}<LoaderCircle class="h-4 w-4 animate-spin" />Instalando…{:else}<Plus class="h-4 w-4" />Instalar versão{/if}</button>
			{:else}<p class="text-sm text-fg-muted">Não há versões disponíveis neste canal.</p>{/if}
		</section>
		<button class={button({variant:"ghost",size:"sm"})} aria-expanded={legacy} onclick={() => legacy = !legacy}>Importação opcional de versões antigas</button>
		{#if legacy}<div class="flex flex-wrap gap-2">
			<button class={button({variant:"secondary",size:"sm"})} disabled={busy} onclick={() => run(() => bedrock.refresh())}><RefreshCw class="h-4 w-4 {busy ? 'animate-spin' : ''}" />Atualizar jogos instalados</button>
			<button class={button({ variant: "secondary", size: "sm" })} disabled={busy} onclick={() => run(connect)}><FolderOpen class="mr-2 h-4 w-4" />{bedrock.value?.provider ? "Reconectar provedor" : "Conectar BedrockLauncher"}</button>
			<button class={button({ variant: "secondary", size: "sm" })} disabled={busy} onclick={() => run(() => openUrl("https://bedrocklauncher.github.io/"))}><ExternalLink class="mr-2 h-4 w-4" />Obter BedrockLauncher</button>
			{#if bedrock.value?.provider}<button class={button({ variant: "secondary", size: "sm" })} disabled={busy} onclick={() => run(() => bedrockOpen())}>Abrir provedor</button><button class={button({ variant: "secondary", size: "icon" })} aria-label="Atualizar instalações Bedrock" disabled={busy} onclick={() => run(() => bedrock.refresh())}><RefreshCw class="h-4 w-4" /></button>{/if}
		</div>
		{/if}
		{#if items.length}
			<p class="text-sm text-fg-muted">Selecione uma edição instalada. O Bedrock do Windows abre diretamente; instalações antigas importadas usam o provedor indicado.</p>
			<label class="block space-y-2 text-sm text-fg">Instalação<select class="w-full rounded-xl border border-border bg-bg-elevated p-3 text-fg" bind:value={selected} disabled={busy}><option value="">Escolha uma instalação</option>{#each items as item (JSON.stringify([item.profileId, item.id]))}<option value={JSON.stringify([item.profileId, item.id])}>{item.profileName} · {item.name}</option>{/each}</select></label>
			<label class="block space-y-2 text-sm text-fg">Nome na biblioteca<input class="w-full rounded-xl border border-border bg-bg-elevated p-3 text-fg" bind:value={name} placeholder={chosen?.name ?? "Minha instalação Bedrock"} maxlength="80" disabled={busy} /></label>
			{#if chosen?.directory}<p class="break-all text-xs text-fg-muted">Pasta própria: {chosen.directory}</p>{/if}
			<button class={button({ variant: "primary", size: "sm" })} disabled={busy || !chosen} onclick={() => run(create)}><Plus class="mr-2 h-4 w-4" />Adicionar à biblioteca</button>
		{/if}
	{/if}
	{#if failure || bedrock.error || bedrock.value?.warning}<p role="alert" class="rounded-xl border border-danger/30 bg-danger/5 p-3 text-sm text-danger">{failure || bedrock.error || bedrock.value?.warning}</p>{/if}
</div>
