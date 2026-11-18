<script lang="ts">
	import { onMount } from 'svelte';
	import { Globe2, Search, RotateCcw, LoaderCircle, Info } from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { saveServerSettings, sendServerCommand, type ServerSettings } from '$lib/api/localServers';
	import { platform } from '$lib/mcserver/state.svelte';
	import { GAMERULE_CATEGORIES, gamerulesForVersion, type ResolvedGamerule } from '$lib/mcserver/gamerules';
	import GlassSelect from './GlassSelect.svelte';

	let draft = $state<ServerSettings | null>(null);
	let dirty = $state(false);
	let saving = $state(false);
	let search = $state('');
	let category = $state('all');

	const running = $derived(platform.running);
	const catalog = $derived(platform.config ? gamerulesForVersion(platform.config.mcVersion) : []);
	const filtered = $derived.by(() => {
		const q = search.trim().toLowerCase();
		return catalog.filter(
			(rule) => (category === 'all' || rule.category === category) && (!q || rule.id.toLowerCase().includes(q) || rule.label.toLowerCase().includes(q))
		);
	});

	function patch(patchValues: Partial<ServerSettings>) {
		if (!draft) return;
		draft = { ...draft, ...patchValues };
		dirty = true;
	}

	async function load() {
		if (!platform.selectedId) return;
		try {
			const raw = await import('$lib/api/localServers').then(({ serverSettings }) => serverSettings(platform.selectedId));
			draft = {
				networkCompressionThreshold: 256,
				entityBroadcastRangePercentage: 100,
				maxTickTime: 60000,
				opPermissionLevel: 4,
				...raw
			};
			dirty = false;
		} catch {
			// Recarrega na próxima varredura.
		}
	}

	async function save() {
		if (!draft || !platform.selectedId) return;
		saving = true;
		try {
			await saveServerSettings(platform.selectedId, draft);
			dirty = false;
			void platform.loadServerData(platform.selectedId);
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			saving = false;
		}
	}

	async function resetDraft() {
		await load();
	}

	async function applyGamerule(rule: ResolvedGamerule, value: string) {
		if (!platform.selectedId) return;
		if (!running) {
			toast('Inicie o servidor para aplicar regras com o comando gamerule.', 'warning');
			return;
		}
		const mapped = rule.mapValue ? rule.mapValue(value) : value;
		try {
			await sendServerCommand(platform.selectedId, `gamerule ${rule.serverName} ${mapped}`);
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		}
	}

	onMount(() => void load());
	$effect(() => {
		platform.selectedId;
		void load();
	});
</script>

<div class="grid">
	<header class="head">
		<div>
			<h1 class="title">Mundo</h1>
			<p class="sub">Ajustes do mundo e das regras de jogo do servidor selecionado.</p>
		</div>
		{#if dirty}
			<span class="dirty">Alterações não salvas</span>
		{/if}
	</header>

	{#if !draft}
		<p class="loading"><LoaderCircle class="h-4 w-4 animate-spin" /> Carregando configuração…</p>
	{:else}
		<div class="grid gap-5 lg:grid-cols-2">
			<section class="card">
				<h2 class="card-title"><Globe2 class="h-4 w-4 text-amethyst-400" /> Mundo e jogabilidade</h2>
				<div class="grid grid-cols-2 gap-3">
					<label class="field">
						<span>Modo de jogo</span>
					<GlassSelect
						ariaLabel="Modo de jogo"
						options={[
							{ value: 'survival' as const, label: 'Sobrevivência' },
							{ value: 'creative' as const, label: 'Criativo' },
							{ value: 'adventure' as const, label: 'Aventura' },
							{ value: 'spectator' as const, label: 'Espectador' }
					]}
					bind:value={draft.gamemode}
					/>
					</label>
					<label class="field">
						<span>Dificuldade</span>
					<GlassSelect
						ariaLabel="Dificuldade"
						options={[
							{ value: 'peaceful' as const, label: 'Pacífico' },
							{ value: 'easy' as const, label: 'Fácil' },
							{ value: 'normal' as const, label: 'Normal' },
							{ value: 'hard' as const, label: 'Difícil' }
					]}
					bind:value={draft.difficulty}
					/>
					</label>
				</div>
				<div class="toggles">
					<label class="toggle-row"><input type="checkbox" checked={draft.hardcore} onchange={(e) => patch({ hardcore: e.currentTarget.checked })} /><span><strong>Hardcore</strong><small>Trava a dificuldade no nível mais difícil.</small></span></label>
					<label class="toggle-row"><input type="checkbox" checked={draft.generateStructures} onchange={(e) => patch({ generateStructures: e.currentTarget.checked })} /><span><strong>Gerar estruturas</strong><small>Aldeias, templos e fortalezas no mundo.</small></span></label>
					<label class="toggle-row"><input type="checkbox" checked={draft.pvp} onchange={(e) => patch({ pvp: e.currentTarget.checked })} /><span><strong>PvP</strong><small>Jogadores podem se atacar.</small></span></label>
					<label class="toggle-row"><input type="checkbox" checked={draft.commandBlocks} onchange={(e) => patch({ commandBlocks: e.currentTarget.checked })} /><span><strong>Blocos de comando</strong><small>Habilita blocos de comando no mundo.</small></span></label>
					<label class="toggle-row"><input type="checkbox" checked={draft.allowFlight} onchange={(e) => patch({ allowFlight: e.currentTarget.checked })} /><span><strong>Voo livre</strong><small>Tolera voo no modo sobrevivência.</small></span></label>
				</div>
				{#if draft.seed}
					<p class="kv-line"><span class="kv-key">Semente</span><span class="font-mono">{draft.seed}</span></p>
				{/if}
				<p class="note">Mudanças de mundo valem para as próximas inicializações do servidor.</p>
			</section>

			<section class="card">
				<h2 class="card-title">Experiência dos jogadores</h2>
				<div class="grid grid-cols-2 gap-3">
					<label class="field"><span>Distância de visão</span><input type="number" min="3" max="32" value={draft.viewDistance} onchange={(e) => patch({ viewDistance: Number(e.currentTarget.value) })} /></label>
					<label class="field"><span>Distância de simulação</span><input type="number" min="3" max="32" value={draft.simulationDistance} onchange={(e) => patch({ simulationDistance: Number(e.currentTarget.value) })} /></label>
					<label class="field"><span>Proteção do spawn (blocos)</span><input type="number" min="0" max="64" value={draft.spawnProtection} onchange={(e) => patch({ spawnProtection: Number(e.currentTarget.value) })} /></label>
					<label class="field"><span>Autosave (minutos)</span><input type="number" min="1" max="60" value={draft.autosaveMinutes} onchange={(e) => patch({ autosaveMinutes: Number(e.currentTarget.value) })} /></label>
				</div>
				<div class="toggles">
					<label class="toggle-row"><input type="checkbox" checked={draft.whitelist} onchange={(e) => patch({ whitelist: e.currentTarget.checked })} /><span><strong>Lista de permissões</strong><small>Somente jogadores autorizados entram.</small></span></label>
					<label class="toggle-row"><input type="checkbox" checked={draft.forceGamemode} onchange={(e) => patch({ forceGamemode: e.currentTarget.checked })} /><span><strong>Forçar modo de jogo</strong><small>Jogadores voltam ao modo padrão ao entrar.</small></span></label>
				</div>
				{#if dirty}
					<div class="save-row">
						<button type="button" class="ghost" onclick={() => void resetDraft()}><RotateCcw class="h-4 w-4" /> Descartar</button>
						<button type="button" class="primary" disabled={saving} onclick={() => void save()}>
							{#if saving}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}
							Salvar alterações
						</button>
					</div>
				{/if}
			</section>
		</div>

		<section class="card">
			<div class="catalog-head">
				<h2 class="card-title mb-0">Regras de jogo · Minecraft {platform.config?.mcVersion ?? ''}</h2>
				<div class="catalog-tools">
					<div class="search-box">
						<Search class="h-4 w-4" />
						<input type="search" placeholder="Buscar regra…" bind:value={search} aria-label="Buscar regra do jogo" />
					</div>
					<div class="chips">
						<button type="button" class={category === 'all' ? 'chip active' : 'chip'} onclick={() => (category = 'all')}>Todas</button>
						{#each GAMERULE_CATEGORIES as item (item.id)}
							<button type="button" class={category === item.id ? 'chip active' : 'chip'} onclick={() => (category = item.id)}>{item.label}</button>
						{/each}
					</div>
				</div>
			</div>
			<p class="note-inline">
				<Info class="h-4 w-4 shrink-0" />
				{#if running}
					Alterar uma regra envia o comando <code class="font-mono">gamerule</code> ao servidor em execução; o efeito é imediato.
				{:else}
					O catálogo mostra o padrão de cada regra nesta versão. Inicie o servidor para alterá-las ao vivo.
				{/if}
			</p>
			<div class="catalog custom-scrollbar">
				{#each filtered as rule (rule.id)}
					<div class="rule">
						<div class="rule-main">
							<p class="rule-label">{rule.label} <span class="font-mono rule-id">{rule.serverName}</span></p>
							<p class="rule-desc">{rule.description}</p>
						</div>
						{#if rule.type === 'bool'}
							<div class="rule-actions">
								<button type="button" class="chip" disabled={!running} onclick={() => void applyGamerule(rule, 'true')}>true</button>
								<button type="button" class="chip" disabled={!running} onclick={() => void applyGamerule(rule, 'false')}>false</button>
							</div>
						{:else}
							<label class="rule-number">
								<input
									type="number"
									min={rule.min ?? 0}
									max={rule.max ?? 2147483647}
									value={Number(rule.defaultValue)}
									disabled={!running}
									onchange={(event) => void applyGamerule(rule, String(Number(event.currentTarget.value)))}
								/>
							</label>
						{/if}
					</div>
				{:else}
					<p class="empty-line">Nenhuma regra corresponde à busca.</p>
				{/each}
			</div>
		</section>
	{/if}
</div>

<style>
	.grid { display: grid; gap: 18px; }
	.head { display: flex; flex-wrap: wrap; align-items: flex-end; justify-content: space-between; gap: 14px; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: 700; }
	.sub { margin-top: 4px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.dirty { padding: 6px 12px; border: 1px solid rgb(var(--warning) / 0.4); border-radius: 999px; background: rgb(var(--warning) / 0.1); font-size: 11.5px; font-weight: 700; color: rgb(var(--warning)); }
	.loading { display: flex; align-items: center; gap: 10px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.card { padding: 16px 18px; border: 1px solid rgb(var(--border-strong)); border-radius: 18px; background: rgb(var(--bg-elevated) / 0.8); }
	.card-title { display: flex; align-items: center; gap: 8px; margin-bottom: 12px; font-size: 12px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.field { display: flex; flex-direction: column; gap: 6px; }
	.field > span { font-size: 10.5px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.field input { min-height: 40px; padding: 8px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; background: rgb(var(--bg-overlay) / 0.7); color: rgb(var(--fg)); font-size: 13.5px; }
	.field input:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 1px; }
	.toggles { display: flex; flex-direction: column; gap: 6px; margin-top: 12px; }
	.toggle-row { display: flex; align-items: flex-start; gap: 10px; padding: 8px 10px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.4); cursor: pointer; }
	.toggle-row input { margin-top: 3px; accent-color: rgb(var(--amethyst-400)); }
	.toggle-row strong { font-size: 13px; font-weight: 600; }
	.toggle-row small { display: block; font-size: 11px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.kv-line { display: flex; gap: 10px; margin-top: 12px; font-size: 12.5px; }
	.kv-key { color: rgb(var(--fg-subtle)); }
	.note { margin-top: 10px; font-size: 11.5px; line-height: 1.6; color: rgb(var(--fg-subtle)); }
	.save-row { display: flex; justify-content: flex-end; gap: 10px; margin-top: 14px; }
	.ghost { display: inline-flex; align-items: center; gap: 8px; padding: 9px 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 9px; color: rgb(var(--fg-muted)); font-size: 13px; font-weight: 600; background: transparent; }
	.ghost:hover { color: rgb(var(--fg)); }
	.primary { display: inline-flex; align-items: center; gap: 8px; padding: 9px 16px; border: 1px solid rgb(var(--amethyst-300) / 0.5); border-radius: 9px; background: rgb(var(--amethyst-500) / 0.85); color: white; font-size: 13px; font-weight: 700; }
	.primary:disabled { opacity: 0.5; }
	.catalog-head { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 8px; }
	.catalog-tools { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; }
	.search-box { display: flex; align-items: center; gap: 8px; padding: 0 12px; min-height: 36px; border: 1px solid rgb(var(--border-strong)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg-subtle)); }
	.search-box input { width: 170px; border: none; background: transparent; color: rgb(var(--fg)); font-size: 13px; outline: none; }
	.chips { display: flex; flex-wrap: wrap; gap: 6px; }
	.chip { padding: 7px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg-muted)); font-size: 11.5px; font-weight: 600; font-family: 'JetBrains Mono', ui-monospace, monospace; }
	.chip:hover:not(:disabled), .chip.active { color: rgb(var(--fg)); border-color: rgb(var(--amethyst-400) / 0.6); }
	.chip:disabled { opacity: 0.45; cursor: not-allowed; }
	.note-inline { display: flex; align-items: flex-start; gap: 9px; margin: 8px 0 12px; font-size: 12.5px; line-height: 1.6; color: rgb(var(--fg-muted)); }
	.catalog { display: grid; gap: 6px; max-height: 380px; overflow-y: auto; padding-right: 4px; }
	.rule { display: flex; align-items: center; gap: 14px; padding: 9px 12px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.35); }
	.rule-main { min-width: 0; flex: 1; }
	.rule-label { font-size: 13px; font-weight: 600; }
	.rule-id { margin-left: 6px; font-size: 10.5px; color: rgb(var(--brass-300)); }
	.rule-desc { margin-top: 2px; font-size: 11.5px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.rule-actions { display: flex; gap: 6px; flex: none; }
	.rule-number input { width: 110px; min-height: 34px; padding: 4px 10px; border: 1px solid rgb(var(--border-strong)); border-radius: 8px; background: rgb(var(--bg-overlay) / 0.7); color: rgb(var(--fg)); font-size: 12.5px; font-family: 'JetBrains Mono', ui-monospace, monospace; }
	.empty-line { font-size: 13px; color: rgb(var(--fg-subtle)); }
</style>
