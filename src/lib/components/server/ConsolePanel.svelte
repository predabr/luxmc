<script lang="ts">
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { Search, Copy, Pause, Play, ChevronDown, History, CornerDownLeft, LoaderCircle } from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { sendServerCommand, serverConsoleText } from '$lib/api/localServers';
	import { platform } from '$lib/mcserver/state.svelte';
	import { formatClock } from '$lib/mcserver/version';
	import GlassSelect from './GlassSelect.svelte';

	interface Line {
		key: string;
		time: string;
		text: string;
		tone: 'error' | 'warn' | 'info';
	}

	let text = $state('');
	let truncated = $state(false);
	let lines = $state<Line[]>([]);
	let query = $state('');
	let levelFilter = $state<'all' | 'error' | 'warn'>('all');
	let autoScroll = $state(true);
	let historyOpen = $state(false);
	let command = $state('');
	let sending = $state(false);
	let history = $state<string[]>([]);
	let historyIndex = -1;
	let scroller: HTMLElement | null = null;
	let poll: ReturnType<typeof setInterval> | null = null;

	const filtered = $derived.by(() => {
		const q = query.trim().toLowerCase();
		return lines.filter((line) => {
			if (levelFilter === 'error' && line.tone !== 'error') return false;
			if (levelFilter === 'warn' && line.tone !== 'error' && line.tone !== 'warn') return false;
			return !q || line.text.toLowerCase().includes(q);
		});
	});

	function classify(raw: string): Line['tone'] {
		const lower = raw.toLowerCase();
		if (lower.includes('error') || lower.includes('exception') || lower.includes('fatal')) return 'error';
		if (lower.includes('warn') || lower.includes('aviso')) return 'warn';
		return 'info';
	}

	function ingest(next: string, wasTruncated: boolean) {
		const incoming = next.length === 0 ? [] : next.split('\n');
		const previousCount = lines.length;
		if (incoming.length < previousCount) lines = [];
		lines = incoming.map((raw, index) => ({
			key: `${index}-${raw.slice(0, 24)}`,
			time: timestampOf(raw),
			text: raw,
			tone: classify(raw)
		}));
		truncated = wasTruncated;
	}

	function timestampOf(raw: string): string {
		const match = /\[(\d{2}:\d{2}:\d{2})\]/.exec(raw);
		return match ? match[1] : '';
	}

	async function refresh() {
		if (!platform.selectedId) return;
		try {
			const result = await serverConsoleText(platform.selectedId);
			ingest(result.text, result.truncated);
		} catch {
			// O console é atualizado na próxima varredura.
		}
	}

	function scrollToEnd() {
		if (autoScroll && scroller) scroller.scrollTop = scroller.scrollHeight;
	}

	function onScroll() {
		if (!scroller) return;
		const distance = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight;
		if (distance > 80 && autoScroll) autoScroll = false;
	}

	$effect(() => {
		filtered.length;
		if (autoScroll) requestAnimationFrame(scrollToEnd);
	});

	async function send() {
		const value = command.trim();
		const id = platform.selectedId;
		if (!value || !id || sending) return;
		if (!platform.running && !platform.activeForSelected) {
			toast('O servidor está offline. Inicie-o antes de enviar comandos.', 'warning');
			return;
		}
		sending = true;
		try {
			await sendServerCommand(id, value);
			history = [value, ...history.filter((item) => item !== value)].slice(0, 60);
			historyIndex = -1;
			command = '';
			historyOpen = false;
			void refresh();
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		} finally {
			sending = false;
		}
	}

	function historyMove(direction: 'up' | 'down') {
		if (history.length === 0) return;
		if (direction === 'up') {
			historyIndex = historyIndex === -1 ? 0 : Math.min(history.length - 1, historyIndex + 1);
		} else {
			historyIndex = historyIndex <= 0 ? -1 : historyIndex - 1;
		}
		command = historyIndex >= 0 ? history[historyIndex] : '';
	}

	function copySelection() {
		const value = filtered.map((line) => line.text).join('\n');
		void navigator.clipboard.writeText(value).then(() => toast('Console copiado.', 'success')).catch(() => toast('Não foi possível copiar.', 'error'));
	}

	onMount(() => {
		void refresh();
		poll = setInterval(() => void refresh(), 1600);
		return () => {
			if (poll) clearInterval(poll);
		};
	});
</script>

<div class="grid">
	<header class="head">
		<div>
			<h1 class="title">Console</h1>
			<p class="sub">
				{#if platform.running}
					Saída ao vivo do processo do servidor. Comandos são entregues ao stdin do Minecraft.
				{:else if platform.activeForSelected && platform.transition}
					O servidor está em transição. A saída aparece aqui assim que o processo responder.
				{:else}
					O servidor está offline. Inicie-o para acompanhar a saída e enviar comandos.
				{/if}
			</p>
		</div>
		<div class="head-actions">
			<div class="filters">
				<div class="search-box">
					<Search class="h-4 w-4" />
					<input type="search" placeholder="Filtrar linhas…" bind:value={query} aria-label="Filtrar linhas do console" />
				</div>
				<div class="level-select">
					<GlassSelect
						ariaLabel="Filtrar por nível"
						options={[
							{ value: 'all' as const, label: 'Tudo' },
							{ value: 'warn' as const, label: 'Avisos e erros' },
							{ value: 'error' as const, label: 'Somente erros' }
						]}
						bind:value={levelFilter}
					/>
				</div>
				<button type="button" class="chip" class:active={autoScroll} onclick={() => (autoScroll = !autoScroll)} title={autoScroll ? 'Pausar rolagem automática' : 'Retomar rolagem automática'}>
					{#if autoScroll}<Pause class="h-3.5 w-3.5" />{:else}<Play class="h-3.5 w-3.5" />{/if}
					Rolar
				</button>
				<button type="button" class="chip" onclick={copySelection} title="Copiar linhas filtradas">
					<Copy class="h-3.5 w-3.5" /> Copiar
				</button>
			</div>
		</div>
	</header>

	<section class="terminal" aria-label="Saída do servidor">
		{#if truncated}
			<p class="truncated">Saídas antigas foram descartadas pelo limite de memória do console.</p>
		{/if}
		<div class="lines custom-scrollbar" bind:this={scroller} onscroll={onScroll}>
			{#each filtered as line, index (line.key + '-' + index)}
				<p class="line" class:warn={line.tone === 'warn'} class:error={line.tone === 'error'}>
					{#if line.time}<span class="time font-mono">{line.time}</span>{/if}
					<span class="text">{line.text}</span>
				</p>
			{:else}
				<p class="line muted">Nenhuma linha{query ? ' corresponde ao filtro' : ' ainda'}.</p>
			{/each}
		</div>
	</section>

	<section class="input-row">
		<div class="history-wrap">
			{#if historyOpen}
				<div class="history" in:fade={{ duration: 140 }}>
					<p class="history-title"><History class="h-3.5 w-3.5" /> Comandos recentes</p>
					{#each history as item (item)}
						<button type="button" class="history-item" onclick={() => { command = item; historyOpen = false; }}>{item}</button>
					{:else}
						<p class="history-item muted">Nenhum comando enviado nesta sessão.</p>
					{/each}
				</div>
			{/if}
		</div>
		<div class="input-line">
			<span class="prompt font-mono">&gt;</span>
			<input
				type="text"
				bind:value={command}
				placeholder={platform.running ? 'Digite um comando, ex.: list, say Olá, time set day' : 'Inicie o servidor para enviar comandos'}
				disabled={!platform.running}
				onkeydown={(event) => {
					if (event.key === 'Enter') void send();
					else if (event.key === 'ArrowUp') { event.preventDefault(); historyMove('up'); }
					else if (event.key === 'ArrowDown') { event.preventDefault(); historyMove('down'); }
					else if (event.key === 'Escape') historyOpen = false;
				}}
				aria-label="Comando do servidor"
			/>
			<button type="button" class="chip" onclick={() => (historyOpen = !historyOpen)} title="Histórico de comandos" aria-expanded={historyOpen}>
				<History class="h-4 w-4" /><ChevronDown class="h-3.5 w-3.5" />
			</button>
			<button type="button" class="send" disabled={!platform.running || sending || !command.trim()} onclick={() => void send()}>
				{#if sending}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<CornerDownLeft class="h-4 w-4" />{/if}
				Enviar
			</button>
		</div>
	</section>
</div>

<style>
	.grid { display: grid; gap: 16px; height: 100%; grid-template-rows: auto 1fr auto; min-height: 0; }
	.head { display: flex; flex-wrap: wrap; align-items: flex-end; justify-content: space-between; gap: 14px; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: 700; }
	.sub { margin-top: 4px; font-size: 13px; color: rgb(var(--fg-muted)); max-width: 620px; }
	.filters { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
	.search-box { display: flex; align-items: center; gap: 8px; padding: 0 14px; min-height: 38px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg-subtle)); }
	.search-box input { width: 180px; border: none; background: transparent; color: rgb(var(--fg)); font-size: 13px; outline: none; }
	.level-select { min-width: 160px; }
	.chip { display: inline-flex; align-items: center; gap: 7px; padding: 8px 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg-muted)); font-size: 12px; font-weight: 600; }
	.chip:hover, .chip.active { color: rgb(var(--fg)); border-color: rgb(var(--amethyst-400) / 0.6); background: rgb(var(--amethyst-500) / 0.1); }
	.chip:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 2px; }
	.terminal { display: flex; flex-direction: column; min-height: 0; border: 1px solid rgb(var(--border-strong)); border-radius: 20px; background: rgb(var(--bg-overlay) / 0.75); overflow: hidden; box-shadow: inset 0 1px 0 rgb(var(--surface-shine) / 0.04); }
	.truncated { padding: 9px 16px; border-bottom: 1px solid rgb(var(--border)); font-size: 11px; color: rgb(var(--fg-subtle)); }
	.lines { flex: 1; min-height: 0; padding: 12px 14px; overflow-y: auto; }
	.line { display: flex; gap: 10px; padding: 3px 8px; border-radius: 14px; font-family: 'JetBrains Mono', ui-monospace, monospace; font-size: 11.5px; line-height: 1.7; color: rgb(var(--fg-muted)); transition: background-color 120ms ease; }
	.line:hover { background: rgb(var(--fg) / 0.04); }
	.line .time { flex: none; padding: 0 6px; border-radius: 6px; background: rgb(var(--fg) / 0.05); color: rgb(var(--fg-subtle)); font-size: 10.5px; line-height: 1.7; }
	.line .text { white-space: pre-wrap; word-break: break-word; min-width: 0; }
	.line.warn { background: rgb(var(--warning) / 0.07); box-shadow: inset 2px 0 0 rgb(var(--warning) / 0.6); }
	.line.warn .text { color: rgb(var(--warning)); }
	.line.error { background: rgb(var(--danger) / 0.08); box-shadow: inset 2px 0 0 rgb(var(--danger) / 0.6); }
	.line.error .text { color: rgb(var(--danger)); }
	.line.muted { color: rgb(var(--fg-subtle)); font-family: inherit; }
	.input-row { display: flex; flex-direction: column; gap: 8px; position: relative; }
	.history-wrap { position: absolute; bottom: 100%; right: 0; width: min(420px, 100%); }
	.history { margin-bottom: 8px; padding: 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: rgb(var(--bg-elevated)); box-shadow: 0 12px 32px rgb(0 0 0 / 0.4); max-height: 240px; overflow-y: auto; }
	.history-title { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; font-size: 10.5px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.history-item { display: block; width: 100%; padding: 8px 12px; border-radius: 999px; text-align: left; font-family: 'JetBrains Mono', ui-monospace, monospace; font-size: 12px; color: rgb(var(--fg-muted)); }
	.history-item:hover { background: rgb(var(--fg) / 0.06); color: rgb(var(--fg)); }
	.history-item.muted { color: rgb(var(--fg-subtle)); font-family: inherit; cursor: default; }
	.input-line { display: flex; align-items: center; gap: 10px; padding: 6px 6px 6px 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; background: rgb(var(--bg-overlay) / 0.7); }
	.input-line:focus-within { border-color: rgb(var(--amethyst-400) / 0.6); box-shadow: 0 0 0 3px rgb(var(--amethyst-500) / 0.14); }
	.prompt { display: grid; place-items: center; width: 26px; height: 26px; flex: none; border-radius: 999px; background: rgb(var(--brass-500) / 0.16); color: rgb(var(--brass-400)); font-weight: 700; font-size: 13px; }
	.input-line input { flex: 1; min-width: 0; min-height: 36px; padding: 0 4px; border: none; background: transparent; color: rgb(var(--fg)); font-family: 'JetBrains Mono', ui-monospace, monospace; font-size: 13px; outline: none; }
	.input-line input:disabled { opacity: 0.5; }
	.send { display: inline-flex; align-items: center; gap: 8px; padding: 10px 18px; border: 1px solid rgb(var(--amethyst-300) / 0.5); border-radius: 999px; background: rgb(var(--amethyst-500) / 0.8); color: white; font-size: 13px; font-weight: 700; }
	.send:hover:not(:disabled) { background: rgb(var(--amethyst-400) / 0.9); }
	.send:disabled { opacity: 0.45; cursor: not-allowed; }
</style>
