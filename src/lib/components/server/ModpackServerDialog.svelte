<script lang="ts">
	import { onMount } from 'svelte';
	import { fly } from 'svelte/transition';
	import { X, Search, LoaderCircle, Download, AlertTriangle, Server, Check, Wifi } from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { createLocalServer, serverSettings, saveServerSettings, startLocalServer, type LocalServerConfig } from '$lib/api/localServers';
	import { modsSearch } from '$lib/api/mods';
	import type { ModSearchResultItem } from '$lib/api/types';
	import { formatGiB } from '$lib/mcserver/version';
	import { softwareByKind } from '$lib/mcserver/software';
	import { platform } from '$lib/mcserver/state.svelte';
	import { provisionPack, resolvePackPlan, type PackPlan } from '$lib/mcserver/packProvision';

	let { onclose, oncreated }: { onclose: () => void; oncreated: (server: LocalServerConfig) => void } = $props();

	const PAGE = 24;
	const LOADER_LABEL: Record<string, string> = { fabric: 'Fabric', forge: 'Forge', neoforge: 'NeoForge', quilt: 'Quilt', vanilla: 'Vanilla' };

	let dialog: HTMLDialogElement | null = $state(null);
	let query = $state('');
	let source = $state<'modrinth' | 'curseforge'>('modrinth');
	let results = $state<ModSearchResultItem[]>([]);
	let searching = $state(false);
	let searchError = $state('');
	let offset = $state(0);
	let exhausted = $state(false);
	let scroller: HTMLElement | null = $state(null);

	let picked = $state<ModSearchResultItem | null>(null);
	let plan = $state<PackPlan | null>(null);
	let planning = $state(false);
	let name = $state('');
	let ramMb = $state(6144);
	let busy = $state(false);
	let progress = $state('');
	let report = $state<string[]>([]);
	let error = $state('');

	const limitedRam = $derived(platform.totalRamMb > 0 ? Math.min(16384, Math.floor((platform.totalRamMb * 0.7) / 512) * 512) : 12288);
	const canCreate = $derived(Boolean(picked && plan && name.trim() && !busy));

	onMount(() => {
		dialog?.showModal();
		void runSearch(true);
	});

	async function runSearch(reset = false) {
		if (searching) return;
		if (reset) {
			offset = 0;
			exhausted = false;
			results = [];
		}
		searching = true;
		searchError = '';
		try {
			const found = await modsSearch(query.trim(), '', PAGE, offset, 'modpack', query.trim() ? 'relevance' : 'downloads', undefined, undefined, source);
			const usable = found.filter((pack) => (pack.categories ?? []).some((kind) => ['fabric', 'forge', 'neoforge', 'quilt'].includes(kind.toLowerCase())));
			results = offset === 0 ? usable : [...results, ...usable.filter((pack) => !results.some((item) => item.sourceId === pack.sourceId))];
			exhausted = found.length < PAGE;
			offset += found.length;
		} catch (cause) {
			searchError = serverMessage(cause);
		} finally {
			searching = false;
		}
	}

	function onScroll() {
		if (!scroller || searching || exhausted) return;
		if (scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < 260) void runSearch();
	}

	function switchSource(next: 'modrinth' | 'curseforge') {
		if (source === next) return;
		source = next;
		void runSearch(true);
	}

	async function pick(pack: ModSearchResultItem) {
		picked = pack;
		plan = null;
		name = pack.title.replace(/[^\w\s.-]/g, '').slice(0, 48).trim() || 'Servidor';
		error = '';
		planning = true;
		try {
			plan = await resolvePackPlan(pack);
			ramMb = Math.max(4096, Math.min(limitedRam, 4096 + (pack.downloads > 50000 ? 1024 : 0)));
		} catch (cause) {
			error = serverMessage(cause);
		} finally {
			planning = false;
		}
	}

	async function create() {
		const pack = picked;
		const chosen = plan;
		if (!pack || !chosen || !canCreate) return;
		busy = true;
		error = '';
		report = [];
		try {
			progress = 'Criando a pasta do servidor…';
			const server = await createLocalServer({ name: name.trim(), mcVersion: chosen.mcVersion, ramMb, maxPlayers: 10, eulaAccepted: true });
			const defaults = await serverSettings(server.id);
			await saveServerSettings(server.id, { ...defaults, motd: name.trim() });
			const applied = await provisionPack(server, pack, chosen, { fresh: true, onProgress: (message) => (progress = message) });
			report = applied;
			progress = 'Ligando o servidor…';
			await startLocalServer(server.id).catch((cause) => toast(serverMessage(cause), 'warning'));
			toast(`Servidor “${name.trim()}” criado com ${pack.title}.`, 'success');
			oncreated(server);
		} catch (cause) {
			error = serverMessage(cause);
			progress = '';
		} finally {
			busy = false;
		}
	}
</script>

<dialog
	bind:this={dialog}
	class="pack-dialog text-fg"
	aria-labelledby="pack-title"
	oncancel={(event) => { event.preventDefault(); if (!busy) onclose(); }}
>
	<div class="frame">
		<header class="head">
			<div>
				<p class="eyebrow">Luxmc · Modpack de servidor</p>
				<h2 id="pack-title">Servidor completo a partir de um modpack</h2>
				<p class="sub">O LUXMC fixa a versão, instala o loader exato, libera a rede e abre a porta para amigos de outros PCs.</p>
			</div>
			<button type="button" class="icon-btn" aria-label="Fechar" disabled={busy} onclick={onclose}><X class="h-5 w-5" /></button>
		</header>

		{#if !picked}
			<div class="search-row">
				<div class="search">
					<Search class="h-4 w-4" />
					<input
						type="search"
						placeholder="Buscar modpack com suporte a servidor…"
						bind:value={query}
						onkeydown={(event) => { if (event.key === 'Enter') void runSearch(true); }}
						aria-label="Buscar modpack"
					/>
				</div>
				<button type="button" class="chip" class:active={source === 'modrinth'} onclick={() => switchSource('modrinth')}>Modrinth</button>
				<button type="button" class="chip" class:active={source === 'curseforge'} onclick={() => switchSource('curseforge')}>CurseForge</button>
				<button type="button" class="go" disabled={searching} aria-label="Pesquisar" onclick={() => void runSearch(true)}>
					{#if searching}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Search class="h-4 w-4" />{/if}
				</button>
			</div>

			<div class="list custom-scrollbar" bind:this={scroller} onscroll={onScroll} role="list">
				{#if searchError}
					<p class="err"><AlertTriangle class="h-4 w-4" /> {searchError}</p>
				{:else if searching && results.length === 0}
					<p class="note"><LoaderCircle class="h-4 w-4 animate-spin" /> Consultando o catálogo oficial…</p>
				{:else if results.length === 0}
					<p class="note">Nenhum modpack com suporte a servidor encontrado.</p>
				{:else}
					{#each results as pack (pack.source + pack.sourceId)}
						<button type="button" class="pack" onclick={() => void pick(pack)} in:fly={{ y: 6, duration: 140 }}>
							{#if pack.iconUrl}<img src={pack.iconUrl} alt="" loading="lazy" />{:else}<span class="ph"><Server class="h-4 w-4" /></span>{/if}
							<span class="pack-main">
								<span class="pack-name">{pack.title}</span>
								<span class="pack-meta">{pack.author ?? 'Autor desconhecido'} · {new Intl.NumberFormat('pt-BR', { notation: 'compact' }).format(pack.downloads)} downloads</span>
								<span class="pack-desc">{pack.description}</span>
							</span>
						</button>
					{/each}
					{#if searching}<p class="note"><LoaderCircle class="h-4 w-4 animate-spin" /> Carregando mais…</p>{/if}
				{/if}
			</div>
		{:else}
			<div class="picked custom-scrollbar">
				<button type="button" class="back" disabled={busy} onclick={() => { picked = null; plan = null; }}>← Trocar pack</button>
				<div class="picked-head">
					{#if picked.iconUrl}<img src={picked.iconUrl} alt="" />{/if}
					<div>
						<p class="pack-name">{picked.title}</p>
						<p class="pack-meta">{picked.author ?? 'Autor desconhecido'}</p>
					</div>
				</div>

				{#if planning}
					<p class="note"><LoaderCircle class="h-4 w-4 animate-spin" /> Resolvendo loader e versão…</p>
				{:else if plan}
					<div class="auto">
						<p class="auto-title"><Wifi class="h-4 w-4" /> Ajustes automáticos</p>
						<ul>
							{#each plan.steps as step (step)}
								<li><Check class="h-3.5 w-3.5" /> {step}</li>
							{/each}
						</ul>
					</div>
					<label class="field">
						<span>Nome do servidor</span>
						<input type="text" maxlength="48" bind:value={name} disabled={busy} />
					</label>
					<label class="field">
						<span>RAM dedicada · {formatGiB(ramMb)}</span>
						<input type="range" min="2048" max={limitedRam} step="512" bind:value={ramMb} disabled={busy} />
					</label>
					<p class="hint">
						{softwareByKind(plan.loaderKind).name} {plan.loaderVersion} · Minecraft {plan.mcVersion} · Java {plan.javaMajor}+
					</p>
				{/if}

				{#if report.length > 0}
					<ul class="report">
						{#each report as line (line)}<li><Check class="h-3.5 w-3.5" /> {line}</li>{/each}
					</ul>
				{/if}
			</div>
		{/if}

		{#if progress}
			<div class="progress" role="status"><LoaderCircle class="h-4 w-4 animate-spin" /> {progress}</div>
		{/if}
		{#if error}<p class="err"><AlertTriangle class="h-4 w-4" /> {error}</p>{/if}

		<footer class="foot">
			<button type="button" class="ghost" disabled={busy} onclick={onclose}>Cancelar</button>
			<button type="button" class="primary" disabled={!canCreate} onclick={() => void create()}>
				{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Download class="h-4 w-4" />{/if}
				Criar servidor completo
			</button>
		</footer>
	</div>
</dialog>

<style>
	.pack-dialog { margin: auto; width: min(820px, calc(100vw - 32px)); height: min(780px, calc(100dvh - 80px)); padding: 0; border: none; background: transparent; color: rgb(var(--fg)); }
	.pack-dialog:not([open]) { display: none; }
	.pack-dialog::backdrop { background: rgb(var(--bg-overlay) / 0.86); }
	.frame { display: flex; flex-direction: column; gap: 12px; height: 100%; padding: 20px 22px; border: 1px solid rgb(var(--border-strong)); border-radius: 22px; background: rgb(var(--bg-elevated)); box-shadow: 0 24px 64px rgb(0 0 0 / 0.55); overflow: hidden; }
	.head { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; }
	.eyebrow { font-size: 10px; font-weight: 700; letter-spacing: 0.2em; text-transform: uppercase; color: rgb(var(--teal-400)); }
	h2 { margin-top: 4px; font-size: 20px; font-weight: 700; }
	.sub { margin-top: 4px; font-size: 12px; line-height: 1.6; color: rgb(var(--fg-muted)); max-width: 520px; }
	.icon-btn { display: grid; place-items: center; width: 34px; height: 34px; border: 1px solid rgb(var(--border-strong)); border-radius: 11px; color: rgb(var(--fg-muted)); }
	.search-row { display: flex; gap: 8px; }
	.search { display: flex; flex: 1; align-items: center; gap: 8px; padding: 0 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 13px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg-subtle)); }
	.search input { flex: 1; min-height: 42px; border: none; background: transparent; color: rgb(var(--fg)); font-size: 13px; outline: none; }
	.chip { padding: 0 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 11px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg-muted)); font-size: 12px; font-weight: 600; }
	.chip.active { border-color: rgb(var(--teal-400) / 0.6); color: rgb(var(--teal-300)); background: rgb(var(--teal-500) / 0.12); }
	.go { display: grid; place-items: center; width: 44px; border: 1px solid rgb(var(--teal-400) / 0.5); border-radius: 11px; background: rgb(var(--teal-500) / 0.15); color: rgb(var(--teal-300)); }
	.list { flex: 1; min-height: 0; display: flex; flex-direction: column; gap: 6px; overflow-y: auto; padding-right: 4px; }
	.pack { display: flex; gap: 12px; padding: 10px; text-align: left; border: 1px solid rgb(var(--border)); border-radius: 13px; background: rgb(var(--bg-overlay) / 0.4); transition: border-color 150ms ease, transform 150ms ease; }
	.pack:hover { border-color: rgb(var(--teal-400) / 0.45); transform: translateY(-1px); }
	.pack img, .pack .ph, .picked-head img { width: 44px; height: 44px; flex: none; border-radius: 11px; object-fit: cover; background: rgb(var(--bg-subtle)); }
	.pack .ph { display: grid; place-items: center; color: rgb(var(--fg-subtle)); }
	.pack-main { min-width: 0; display: flex; flex-direction: column; gap: 2px; }
	.pack-name { font-size: 13.5px; font-weight: 650; }
	.pack-meta { font-size: 11px; color: rgb(var(--fg-subtle)); }
	.pack-desc { font-size: 11.5px; color: rgb(var(--fg-muted)); overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; }
	.picked { flex: 1; min-height: 0; display: flex; flex-direction: column; gap: 10px; overflow-y: auto; padding-right: 4px; }
	.back { align-self: flex-start; font-size: 12px; color: rgb(var(--fg-muted)); }
	.picked-head { display: flex; align-items: center; gap: 12px; }
	.auto { padding: 12px 14px; border: 1px solid rgb(var(--teal-400) / 0.35); border-radius: 13px; background: rgb(var(--teal-500) / 0.08); }
	.auto-title { display: flex; align-items: center; gap: 8px; font-size: 12px; font-weight: 700; color: rgb(var(--teal-300)); }
	.auto ul, .report { margin-top: 8px; display: flex; flex-direction: column; gap: 4px; }
	.auto li, .report li { display: flex; align-items: center; gap: 7px; font-size: 12px; color: rgb(var(--fg-muted)); }
	.report { padding: 10px 12px; border: 1px solid rgb(var(--success) / 0.3); border-radius: 12px; background: rgb(var(--success) / 0.07); }
	.report li { color: rgb(var(--fg)); }
	.field { display: flex; flex-direction: column; gap: 5px; font-size: 11px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.field input[type='text'] { min-height: 40px; padding: 7px 11px; border: 1px solid rgb(var(--border-strong)); border-radius: 11px; background: rgb(var(--bg-overlay) / 0.7); color: rgb(var(--fg)); font-size: 13px; text-transform: none; letter-spacing: 0; }
	.field input[type='range'] { accent-color: rgb(var(--teal-400)); }
	.hint { font-size: 11.5px; color: rgb(var(--fg-subtle)); }
	.note, .err, .progress { display: flex; align-items: center; gap: 8px; font-size: 12.5px; color: rgb(var(--fg-muted)); }
	.err { color: rgb(var(--danger)); }
	.foot { display: flex; justify-content: flex-end; gap: 10px; border-top: 1px solid rgb(var(--border)); padding-top: 12px; }
	.ghost { padding: 10px 16px; border: 1px solid rgb(var(--border-strong)); border-radius: 11px; color: rgb(var(--fg-muted)); font-size: 13px; font-weight: 600; }
	.primary { display: inline-flex; align-items: center; gap: 8px; padding: 11px 20px; border: 1px solid rgb(var(--teal-300) / 0.6); border-radius: 11px; background: rgb(var(--teal-500) / 0.9); color: white; font-size: 13px; font-weight: 800; }
	.primary:disabled { opacity: 0.45; }
	@media (prefers-reduced-motion: reduce) { .pack { transition: none; } }
</style>
