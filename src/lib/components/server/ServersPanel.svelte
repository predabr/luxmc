<script lang="ts">
	import { onMount } from 'svelte';
	import { Plus, Trash2, Play, Square, LoaderCircle, Server, AlertTriangle, Palette, GripVertical, Users, MemoryStick, Box } from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { serverManage, startLocalServer, stopLocalServer, type LocalServerConfig } from '$lib/api/localServers';
	import { platform } from '$lib/mcserver/state.svelte';
	import { softwareByKind } from '$lib/mcserver/software';
	import { formatGiB } from '$lib/mcserver/version';

	let { actionBusy, onCreate }: { actionBusy: boolean; onCreate: () => void } = $props();

	let confirmDelete = $state<LocalServerConfig | null>(null);
	let deleteName = $state('');
	let deleteBackups = $state(false);
	let deleting = $state(false);
	let busyId = $state('');

	const ORDER_KEY = 'luxmc.localServerOrder';
	const COLORS_KEY = 'luxmc.localServerColors';

	const COLOR_OPTIONS = [
		{ id: 'brass', css: 'rgb(var(--brass-400))' },
		{ id: 'amethyst', css: 'rgb(var(--amethyst-400))' },
		{ id: 'blue', css: 'rgb(var(--brand-400))' },
		{ id: 'green', css: 'rgb(var(--success))' },
		{ id: 'yellow', css: 'rgb(var(--warning))' },
		{ id: 'red', css: 'rgb(var(--danger))' },
		{ id: 'sky', css: 'rgb(var(--info))' },
		{ id: 'copper', css: 'rgb(var(--copper-400))' }
	];

	let order = $state<string[]>([]);
	let colors = $state<Record<string, string>>({});
	let dragId = $state<string | null>(null);
	let dropTargetId = $state<string | null>(null);
	let dropAxis = $state<'x' | 'y'>('x');
	let dropBefore = $state(true);
	let dragRect: DOMRect | null = null;

	const PHASE_LABEL: Record<string, string> = {
		running: 'Online',
		starting: 'Iniciando',
		preparing: 'Preparando',
		downloading: 'Baixando',
		java: 'Preparando o Java',
		stopping: 'Encerrando',
		unreachable: 'Sem resposta',
		failed: 'Falhou',
		stopped: 'Offline'
	};

	const servers = $derived.by(() => {
		const list = platform.servers;
		const index = new Map(order.map((id, position) => [id, position]));
		return [...list].sort((a, b) => (index.get(a.id) ?? Number.MAX_SAFE_INTEGER) - (index.get(b.id) ?? Number.MAX_SAFE_INTEGER));
	});

	function accentOf(id: string): string {
		return COLOR_OPTIONS.find((option) => option.id === colors[id])?.css ?? 'rgb(var(--brass-500))';
	}

	function phaseOf(server: LocalServerConfig): string {
		const states = platform.overview?.states ?? (platform.active ? [platform.active] : []);
		return states.find((state) => state.id === server.id)?.phase ?? 'stopped';
	}

	function select(server: LocalServerConfig) {
		platform.select(server.id);
		void platform.loadServerData(server.id);
	}

	function persistOrder() {
		try {
			localStorage.setItem(ORDER_KEY, JSON.stringify(order));
		} catch {}
	}

	function persistColors() {
		try {
			localStorage.setItem(COLORS_KEY, JSON.stringify(colors));
		} catch {}
	}

	function setColor(id: string, colorId: string) {
		colors = { ...colors, [id]: colorId };
		persistColors();
	}

	function clearColor(id: string) {
		const next = { ...colors };
		delete next[id];
		colors = next;
		persistColors();
	}

	function reorder(sourceId: string, targetId: string, side: 'before' | 'after') {
		if (sourceId === targetId) return;
		const ids = servers.map((server) => server.id);
		const from = ids.indexOf(sourceId);
		if (from < 0) return;
		ids.splice(from, 1);
		let to = ids.indexOf(targetId);
		if (to < 0) return;
		if (side === 'after') to += 1;
		ids.splice(to, 0, sourceId);
		order = ids;
		persistOrder();
	}

	function onCardDragStart(event: DragEvent, id: string) {
		if (event.target instanceof Element && event.target.closest('button')) {
			event.preventDefault();
			return;
		}
		if (!event.dataTransfer) return;
		event.dataTransfer.setData('text/plain', id);
		event.dataTransfer.effectAllowed = 'move';
		dragId = id;
		dragRect = (event.currentTarget as HTMLElement).getBoundingClientRect();
	}

	function onCardDragOver(event: DragEvent, id: string) {
		if (!dragId || !event.dataTransfer) return;
		event.preventDefault();
		event.dataTransfer.dropEffect = 'move';
		if (dragId === id) {
			dropTargetId = null;
			return;
		}
		const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const sameRow = dragRect ? Math.abs(dragRect.top - rect.top) < rect.height * 0.6 : true;
		dropAxis = sameRow ? 'x' : 'y';
		dropBefore = sameRow ? event.clientX < rect.left + rect.width / 2 : event.clientY < rect.top + rect.height / 2;
		dropTargetId = id;
	}

	function onCardDrop(event: DragEvent, targetId: string) {
		event.preventDefault();
		if (dragId && dragId !== targetId) reorder(dragId, targetId, dropBefore ? 'before' : 'after');
		clearDrag();
	}

	function clearDrag() {
		dragId = null;
		dropTargetId = null;
		dragRect = null;
	}

	function onCardDragLeave(event: DragEvent) {
		const related = event.relatedTarget;
		if (related instanceof Node && (event.currentTarget as HTMLElement).contains(related)) return;
		if (dropTargetId === (event.currentTarget as HTMLElement).dataset.serverId) dropTargetId = null;
	}

	async function startOne(id: string) {
		if (busyId) return;
		busyId = id;
		try {
			await startLocalServer(id);
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyId = '';
		}
	}

	async function stopOne(id: string) {
		if (busyId) return;
		busyId = id;
		try {
			await stopLocalServer(id);
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			busyId = '';
		}
	}

	async function confirmDeletion() {
		const target = confirmDelete;
		if (!target || deleting) return;
		if (deleteName !== target.name) {
			toast('Digite o nome exato do servidor para confirmar.', 'warning');
			return;
		}
		deleting = true;
		try {
			await serverManage({ action: 'delete_server', id: target.id, name: target.name, deleteBackups });
			toast(`Servidor “${target.name}” excluído.`, 'success');
			confirmDelete = null;
			deleteName = '';
			await platform.refreshServers();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			deleting = false;
		}
	}

	onMount(() => {
		try {
			const storedOrder: unknown = JSON.parse(localStorage.getItem(ORDER_KEY) ?? '[]');
			if (Array.isArray(storedOrder)) order = storedOrder.filter((id): id is string => typeof id === 'string');
		} catch {}
		try {
			const storedColors: unknown = JSON.parse(localStorage.getItem(COLORS_KEY) ?? '{}');
			if (storedColors && typeof storedColors === 'object' && !Array.isArray(storedColors)) colors = storedColors as Record<string, string>;
		} catch {}
	});
</script>

<div class="grid gap-5">
	<header class="head">
		<div>
			<h1 class="title">Servidores</h1>
			<p class="sub">Cada servidor tem a própria pasta, versão, mundo, porta e console. Vários servidores podem rodar ao mesmo tempo, de forma totalmente independente.</p>
		</div>
		<button type="button" class="create" onclick={onCreate}><Plus class="h-4 w-4" /> Criar servidor</button>
	</header>

	{#if servers.length === 0}
		<div class="empty">
			<Server class="h-8 w-8 text-fg-subtle" />
			<p>Nenhum servidor criado ainda.</p>
			<button type="button" class="create" onclick={onCreate}><Plus class="h-4 w-4" /> Criar o primeiro servidor</button>
		</div>
	{:else}
		<p class="hint">Arraste os blocos para reorganizar e toque nas cores para pintar cada um. A ordem e as cores ficam salvas neste computador.</p>
		<div class="cards">
			{#each servers as server (server.id)}
				{@const phase = phaseOf(server)}
				{@const isActive = (platform.overview?.states ?? []).some((state) => state.id === server.id)}
				{@const selected = server.id === platform.selectedId}
				{@const accent = accentOf(server.id)}
				{@const dragging = dragId === server.id}
				{@const targeted = dropTargetId === server.id}
				<div
					class="card"
					class:selected
					class:dragging
					class:targeted
					class:drop-before={targeted && dropBefore}
					class:drop-after={targeted && !dropBefore}
					class:drop-x={targeted && dropAxis === 'x'}
					class:drop-y={targeted && dropAxis === 'y'}
					style="--card-accent: {accent}"
					data-server-id={server.id}
					role="button"
					tabindex="0"
					aria-pressed={selected}
					draggable="true"
					ondragstart={(event) => onCardDragStart(event, server.id)}
					ondragover={(event) => onCardDragOver(event, server.id)}
					ondrop={(event) => onCardDrop(event, server.id)}
					ondragleave={onCardDragLeave}
					ondragend={clearDrag}
					onclick={() => select(server)}
					onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ') select(server); }}
				>
					<div class="card-top">
						<span class="grip" title="Arraste para reorganizar"><GripVertical class="h-4 w-4" /></span>
						<span class="name">{server.name}</span>
						{#if selected}<span class="tag tag-brand">Selecionado</span>{/if}
						<span class={phase === 'running' ? 'phase ok' : phase === 'failed' ? 'phase bad' : phase === 'stopped' ? 'phase' : 'phase busy'}>{PHASE_LABEL[phase] ?? phase}</span>
					</div>
					<div class="chips">
						<span class="chip"><Box class="h-3.5 w-3.5" /> Minecraft {server.mcVersion}</span>
						<span class="chip"><Users class="h-3.5 w-3.5" /> {server.maxPlayers}</span>
						<span class="chip"><MemoryStick class="h-3.5 w-3.5" /> {formatGiB(server.ramMb)}</span>
					</div>
					{#if selected && platform.loader.kind !== 'vanilla'}
						<p class="loader">{softwareByKind(platform.loader.kind).name} {platform.loader.version}{isActive && phase === 'running' ? ' · Online' : ''}</p>
					{:else if isActive && phase === 'running'}
						<p class="loader">Servidor em execução · porta {platform.overview?.states?.find((state) => state.id === server.id)?.port ?? '—'}</p>
					{/if}
					<div class="card-actions">
						{#if isActive && !['stopped', 'failed'].includes(phase)}
							<button type="button" class="icon-btn stop" title="Parar servidor" disabled={actionBusy || busyId !== ''} onclick={(event) => { event.stopPropagation(); void stopOne(server.id); }}>
								{#if busyId === server.id}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Square class="h-4 w-4" />{/if}
								<span>Parar</span>
							</button>
						{:else}
							<button type="button" class="icon-btn play" title="Iniciar servidor" disabled={actionBusy || busyId !== ''} onclick={(event) => { event.stopPropagation(); void startOne(server.id); }}>
								{#if busyId === server.id}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Play class="h-4 w-4" />{/if}
								<span>Iniciar</span>
							</button>
						{/if}
						<button type="button" class="icon-btn danger" title="Excluir servidor" onclick={(event) => { event.stopPropagation(); confirmDelete = server; deleteName = ''; deleteBackups = false; }}>
							<Trash2 class="h-4 w-4" />
						</button>
					</div>
					<div class="swatches" role="group" aria-label="Cor do bloco">
						<span class="swatch-icon" title="Cor do bloco"><Palette class="h-3.5 w-3.5" /></span>
						{#each COLOR_OPTIONS as option (option.id)}
							<button
								type="button"
								class="dot"
								class:picked={colors[server.id] === option.id || (!colors[server.id] && option.id === 'brass')}
								style="--dot: {option.css}"
								title={option.id}
								aria-label={`Cor ${option.id}`}
								onclick={(event) => { event.stopPropagation(); setColor(server.id, option.id); }}
							></button>
						{/each}
						{#if colors[server.id]}
							<button type="button" class="reset" onclick={(event) => { event.stopPropagation(); clearColor(server.id); }}>Padrão</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if confirmDelete}
	<dialog class="confirm text-fg" open>
		<div class="confirm-card">
			<header class="confirm-head">
				<AlertTriangle class="h-5 w-5 text-warning" />
				<h2>Excluir “{confirmDelete.name}”?</h2>
			</header>
			<p class="confirm-text">
				Esta ação apaga permanentemente a pasta do servidor, incluindo o mundo e as configurações.
				Jogadores conectados serão desconectados. Esta ação não pode ser desfeita.
			</p>
			<label class="confirm-check">
				<input type="checkbox" bind:checked={deleteBackups} />
				<span>Também excluir os backups deste servidor</span>
			</label>
			<label class="confirm-field">
				<span>Digite <strong class="font-mono">{confirmDelete.name}</strong> para confirmar</span>
				<input type="text" bind:value={deleteName} />
			</label>
			<footer class="confirm-footer">
				<button type="button" class="ghost" onclick={() => (confirmDelete = null)}>Cancelar</button>
				<button type="button" class="danger" disabled={deleting || deleteName !== confirmDelete.name} onclick={() => void confirmDeletion()}>
					{#if deleting}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}
					Excluir para sempre
				</button>
			</footer>
		</div>
	</dialog>
{/if}

<style>
	.grid { display: grid; gap: 18px; }
	.head { display: flex; flex-wrap: wrap; align-items: flex-end; justify-content: space-between; gap: 14px; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 26px; font-weight: 700; letter-spacing: 0.01em; }
	.sub { margin-top: 6px; font-size: 13.5px; line-height: 1.6; color: rgb(var(--fg-muted)); max-width: 560px; }
	.hint { display: flex; align-items: center; gap: 8px; font-size: 12.5px; color: rgb(var(--fg-subtle)); }
	.hint::before { content: ''; width: 8px; height: 8px; border-radius: 3px; background: rgb(var(--brass-400)); box-shadow: 0 0 10px rgb(var(--brass-400) / 0.6); }
	.create { display: inline-flex; align-items: center; gap: 8px; padding: 11px 20px; border: 1px solid rgb(var(--brass-500) / 0.55); border-radius: 999px; background: rgb(var(--brass-500) / 0.14); color: rgb(var(--brass-300)); font-size: 12.5px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; }
	.create:hover { background: rgb(var(--brass-500) / 0.24); }
	.create:focus-visible { outline: 2px solid rgb(var(--brass-400)); outline-offset: 2px; }
	.empty { display: flex; flex-direction: column; align-items: center; gap: 14px; padding: 64px 0; color: rgb(var(--fg-muted)); }

	.cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(300px, 100%), 1fr)); gap: 18px; }
	.card {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 18px 20px 16px;
		border: 1px solid color-mix(in srgb, var(--card-accent) 45%, rgb(var(--border-strong)));
		border-radius: 22px;
		background:
			linear-gradient(165deg, color-mix(in srgb, var(--card-accent) 24%, rgb(var(--bg-elevated))) 0%, rgb(var(--bg-elevated)) 62%),
			rgb(var(--bg-elevated));
		box-shadow: inset 0 1px 0 color-mix(in srgb, var(--card-accent) 30%, transparent);
		text-align: left;
		cursor: grab;
		overflow: hidden;
		transition: border-color 160ms ease, background-color 160ms ease, transform 160ms ease, box-shadow 160ms ease, opacity 160ms ease;
	}
	.card:hover { transform: translateY(-3px); border-color: color-mix(in srgb, var(--card-accent) 80%, transparent); box-shadow: 0 16px 34px rgb(var(--shadow-color) / 0.32), inset 0 1px 0 color-mix(in srgb, var(--card-accent) 40%, transparent); }
	.card:focus-visible { outline: 2px solid var(--card-accent); outline-offset: 2px; }
	.card.selected {
		border-color: var(--card-accent);
		background:
			linear-gradient(165deg, color-mix(in srgb, var(--card-accent) 34%, rgb(var(--bg-elevated))) 0%, rgb(var(--bg-elevated)) 68%),
			rgb(var(--bg-elevated));
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--card-accent) 30%, transparent), 0 18px 38px rgb(var(--shadow-color) / 0.34);
	}
	.card.dragging { opacity: 0.4; transform: scale(0.97) rotate(1.2deg); }
	.card.targeted { border-color: var(--card-accent); }
	.card.targeted.drop-x.drop-before::after,
	.card.targeted.drop-x.drop-after::after,
	.card.targeted.drop-y.drop-before::after,
	.card.targeted.drop-y.drop-after::after {
		content: '';
		position: absolute;
		background: var(--card-accent);
		border-radius: 999px;
		z-index: 2;
		box-shadow: 0 0 12px color-mix(in srgb, var(--card-accent) 70%, transparent);
	}
	.card.targeted.drop-x.drop-before::after { left: 5px; top: 14px; bottom: 14px; width: 4px; }
	.card.targeted.drop-x.drop-after::after { right: 5px; top: 14px; bottom: 14px; width: 4px; }
	.card.targeted.drop-y.drop-before::after { top: 5px; left: 14px; right: 14px; height: 4px; }
	.card.targeted.drop-y.drop-after::after { bottom: 5px; left: 14px; right: 14px; height: 4px; }

	.card-top { display: flex; align-items: center; gap: 10px; }
	.grip { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 10px; background: rgb(0 0 0 / 0.22); color: rgb(var(--fg-subtle)); cursor: grab; flex: none; }
	.card:hover .grip { color: var(--card-accent); background: color-mix(in srgb, var(--card-accent) 18%, rgb(0 0 0 / 0.22)); }
	.name { flex: 1; min-width: 0; font-size: 17px; font-weight: 800; letter-spacing: 0.01em; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.tag { flex: none; padding: 3px 9px; border-radius: 999px; font-size: 10px; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; }
	.tag-brand { background: rgb(var(--amethyst-500) / 0.2); color: rgb(var(--amethyst-300)); border: 1px solid rgb(var(--amethyst-400) / 0.4); }
	.phase { display: inline-flex; align-items: center; gap: 7px; flex: none; padding: 4px 11px; border-radius: 999px; font-size: 10.5px; font-weight: 800; letter-spacing: 0.07em; text-transform: uppercase; background: rgb(0 0 0 / 0.3); border: 1px solid rgb(var(--border-strong)); color: rgb(var(--fg-muted)); }
	.phase.ok { color: rgb(var(--success)); border-color: rgb(var(--success) / 0.5); background: rgb(var(--success) / 0.12); }
	.phase.ok::before { content: ''; width: 7px; height: 7px; border-radius: 50%; background: rgb(var(--success)); box-shadow: 0 0 8px rgb(var(--success) / 0.8); animation: pulse 1.8s ease-in-out infinite; }
	.phase.bad { color: rgb(var(--danger)); border-color: rgb(var(--danger) / 0.5); background: rgb(var(--danger) / 0.12); }
	.phase.busy { color: rgb(var(--brass-300)); border-color: rgb(var(--brass-400) / 0.5); background: rgb(var(--brass-500) / 0.12); }
	@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.35; } }

	.chips { display: flex; flex-wrap: wrap; gap: 7px; }
	.chip { display: inline-flex; align-items: center; gap: 6px; padding: 5px 11px; border-radius: 999px; font-size: 11.5px; font-weight: 600; color: rgb(var(--fg-muted)); background: rgb(0 0 0 / 0.28); border: 1px solid rgb(var(--border-strong) / 0.7); }
	.chip :global(svg) { color: var(--card-accent); }
	.loader { font-size: 11.5px; color: rgb(var(--fg-subtle)); }

	.card-actions { display: flex; gap: 9px; }
	.card .icon-btn { display: inline-flex; align-items: center; justify-content: center; gap: 7px; min-width: 42px; height: 42px; padding: 0 13px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; color: rgb(var(--fg-muted)); background-color: rgb(0 0 0 / 0.25) !important; font-size: 12.5px; font-weight: 700; transition: border-color 140ms ease, color 140ms ease, background-color 140ms ease, transform 140ms ease; }
	.card .icon-btn:hover:not(:disabled) { color: rgb(var(--fg)); border-color: color-mix(in srgb, var(--card-accent) 70%, transparent); transform: translateY(-1px); }
	.card .icon-btn.play { color: rgb(var(--success)); border-color: rgb(var(--success) / 0.45); background-color: rgb(var(--success) / 0.16) !important; }
	.card .icon-btn.play:hover:not(:disabled) { background-color: rgb(var(--success) / 0.26) !important; border-color: rgb(var(--success) / 0.7); }
	.card .icon-btn.stop { color: rgb(var(--warning)); border-color: rgb(var(--warning) / 0.45); background-color: rgb(var(--warning) / 0.14) !important; }
	.card .icon-btn.stop:hover:not(:disabled) { background-color: rgb(var(--warning) / 0.24) !important; border-color: rgb(var(--warning) / 0.7); }
	.card .icon-btn.danger { margin-left: auto; }
	.card .icon-btn.danger:hover:not(:disabled) { color: rgb(var(--danger)); border-color: rgb(var(--danger) / 0.55); background-color: rgb(var(--danger) / 0.16) !important; }
	.card .icon-btn:disabled { opacity: 0.5; cursor: not-allowed; }
	.card .icon-btn:focus-visible { outline: 2px solid var(--card-accent); outline-offset: 2px; }

	.swatches { display: flex; align-items: center; gap: 8px; padding-top: 12px; border-top: 1px dashed color-mix(in srgb, var(--card-accent) 35%, rgb(var(--border-strong))); }
	.swatch-icon { display: grid; place-items: center; color: var(--card-accent); margin-right: 2px; }
	.card .dot { width: 24px; height: 24px; border-radius: 9px; background: var(--dot) !important; border: 2px solid rgb(0 0 0 / 0.45); box-shadow: inset 0 -3px 0 rgb(0 0 0 / 0.25), inset 0 2px 0 rgb(255 255 255 / 0.25); transition: transform 130ms ease, border-color 130ms ease, box-shadow 130ms ease; }
	.card .dot:hover { transform: translateY(-2px) scale(1.1); }
	.card .dot.picked { border-color: rgb(var(--fg)); transform: scale(1.12); box-shadow: inset 0 -3px 0 rgb(0 0 0 / 0.25), inset 0 2px 0 rgb(255 255 255 / 0.25), 0 0 0 2px color-mix(in srgb, var(--dot) 55%, transparent), 0 0 12px color-mix(in srgb, var(--dot) 45%, transparent); }
	.dot:focus-visible { outline: 2px solid var(--card-accent); outline-offset: 2px; }
	.card .reset { margin-left: auto; padding: 4px 10px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; background-color: rgb(0 0 0 / 0.3) !important; color: rgb(var(--fg-muted)); font-size: 10.5px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; }
	.card .reset:hover { color: rgb(var(--fg)); border-color: color-mix(in srgb, var(--card-accent) 60%, transparent); }

	.confirm { margin: auto; padding: 0; border: none; background: transparent; color: rgb(var(--fg)); }
	.confirm::backdrop { background: rgb(var(--bg-overlay) / 0.8); }
	.confirm-card { width: min(480px, calc(100vw - 40px)); padding: 22px 24px; border: 1px solid rgb(var(--danger) / 0.4); border-radius: 20px; background: rgb(var(--bg-elevated)); box-shadow: 0 24px 64px rgb(0 0 0 / 0.5); }
	.confirm-head { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
	.confirm-head h2 { font-size: 17px; font-weight: 700; }
	.confirm-text { font-size: 13px; line-height: 1.7; color: rgb(var(--fg-muted)); }
	.confirm-check { display: flex; align-items: center; gap: 9px; margin-top: 12px; font-size: 13px; cursor: pointer; }
	.confirm-check input { accent-color: rgb(var(--danger)); }
	.confirm-field { display: flex; flex-direction: column; gap: 6px; margin-top: 14px; font-size: 12px; color: rgb(var(--fg-muted)); }
	.confirm-field input { min-height: 40px; padding: 8px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: rgb(var(--bg-overlay) / 0.7); color: rgb(var(--fg)); font-size: 13.5px; }
	.confirm-footer { display: flex; justify-content: flex-end; gap: 10px; margin-top: 18px; }
	.confirm-footer .ghost { padding: 9px 16px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; color: rgb(var(--fg-muted)); font-size: 13px; font-weight: 600; }
	.confirm-footer .ghost:hover { color: rgb(var(--fg)); }
	.confirm-footer .danger { display: inline-flex; align-items: center; gap: 8px; padding: 9px 16px; border: 1px solid rgb(var(--danger) / 0.5); border-radius: 999px; background: rgb(var(--danger) / 0.14); color: rgb(var(--danger)); font-size: 13px; font-weight: 700; }
	.confirm-footer .danger:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
