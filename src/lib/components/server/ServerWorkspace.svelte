<script lang="ts">
	import { onMount } from 'svelte';
	import { Plus, Minus, Square, X, LoaderCircle, AlertTriangle } from 'lucide-svelte';
	import { platform } from '$lib/mcserver/state.svelte';
	import ServerCreationWizard from './ServerCreationWizard.svelte';
	import ServerNodeCanvas from './ServerNodeCanvas.svelte';
	import ModpackServerDialog from './ModpackServerDialog.svelte';

	let showWizard = $state(false);
	let showPackDialog = $state(false);
	let actionBusy = $state(false);
	let wizardKey = $state(0);

	const config = $derived(platform.config);
	const phase = $derived(platform.phase);

	const PHASE_META: Record<string, { label: string; tone: 'success' | 'warning' | 'danger' | 'info' | 'neutral' }> = {
		running: { label: 'Online', tone: 'success' },
		starting: { label: 'Iniciando', tone: 'info' },
		preparing: { label: 'Preparando', tone: 'info' },
		downloading: { label: 'Baixando', tone: 'info' },
		java: { label: 'Preparando o Java', tone: 'info' },
		stopping: { label: 'Encerrando', tone: 'warning' },
		unreachable: { label: 'Sem resposta', tone: 'warning' },
		failed: { label: 'Falhou', tone: 'danger' },
		stopped: { label: 'Offline', tone: 'neutral' }
	};

	const phaseMeta = $derived(PHASE_META[phase] ?? { label: 'Offline', tone: 'neutral' as const });
	const currentLabel = $derived(config ? config.name : 'Nenhum servidor selecionado');

	function windowAction(action: 'minimize' | 'maximize' | 'close') {
		void (async () => {
			try {
				if (window.electronAPI?.window) {
					if (action === 'minimize') await window.electronAPI.window.minimize();
					else if (action === 'maximize') await window.electronAPI.window.maximize();
					else await window.electronAPI.window.close();
					return;
				}
				const { getCurrentWindow } = await import('@tauri-apps/api/window');
				const win = getCurrentWindow();
				if (action === 'minimize') await win.minimize();
				else if (action === 'maximize') await win.toggleMaximize();
				else await win.close();
			} catch {
				// Controles de janela indisponíveis neste ambiente.
			}
		})();
	}

	function openWizard() {
		wizardKey += 1;
		showWizard = true;
	}

	onMount(() => {
		platform.startPolling();
		return () => platform.stopPolling();
	});

	$effect(() => {
		const id = platform.selectedId;
		void platform.loadServerData(id);
	});
</script>

<div class="workspace">
	<header class="topbar">
		<div class="brand">
			<img src="/branding/luxmc-emblem-128.png" alt="LUXMC" class="emblem" />
			<div class="brand-text">
				<span class="brand-name">LUXMC</span>
				<span class="brand-sub">Servidores dedicados</span>
			</div>
		</div>

		<div class="current">
			<span class="dot dot-{phaseMeta.tone}" aria-hidden="true"></span>
			<span class="current-name">{currentLabel}</span>
			<span class="phase-badge tone-{phaseMeta.tone}">
				{#if platform.transition}<LoaderCircle class="h-3 w-3 animate-spin" />{/if}
				{phaseMeta.label}
			</span>
		</div>

		<div class="actions">
			<button type="button" class="create-btn" onclick={openWizard}>
				<Plus class="h-4 w-4" /> Criar servidor
			</button>
			<span class="rule" aria-hidden="true"></span>
			<div class="window-controls">
				<button type="button" class="win-btn" aria-label="Minimizar" onclick={() => windowAction('minimize')}>
					<Minus class="h-3.5 w-3.5" />
				</button>
				<button type="button" class="win-btn" aria-label="Maximizar" onclick={() => windowAction('maximize')}>
					<Square class="h-3 w-3" />
				</button>
				<button type="button" class="win-btn win-close" aria-label="Fechar" onclick={() => windowAction('close')}>
					<X class="h-3.5 w-3.5" />
				</button>
			</div>
		</div>
	</header>

	{#if platform.warnings.length > 0}
		<div class="warn-strip" role="status">
			<AlertTriangle class="h-4 w-4 shrink-0 text-warning" />
			{platform.warnings.join(' ')}
		</div>
	{/if}

	<div class="canvas-host">
		<ServerNodeCanvas {actionBusy} onCreate={openWizard} onFromPack={() => (showPackDialog = true)} />
	</div>
</div>

{#if showPackDialog}
	<ModpackServerDialog
		onclose={() => (showPackDialog = false)}
		oncreated={(server) => {
			showPackDialog = false;
			platform.select(server.id);
		}}
	/>
{/if}

{#if showWizard}
	{#key wizardKey}
		<ServerCreationWizard
			onclose={() => (showWizard = false)}
			oncreated={(server) => {
				showWizard = false;
				platform.select(server.id);
			}}
		/>
	{/key}
{/if}

<style>
	.workspace {
		position: relative;
		display: flex; height: 100%; min-height: 0; flex-direction: column;
		background: rgb(var(--bg));
		border-radius: 18px;
		box-shadow: inset 0 1px 0 rgb(var(--fg) / 0.04), 0 18px 44px rgb(0 0 0 / 0.38);
		overflow: hidden;
	}
	.workspace::after {
		content: "";
		position: absolute; inset: 0;
		border-radius: inherit; padding: 1px;
		background: linear-gradient(150deg,
			rgb(var(--brass-500) / 0.55),
			rgb(var(--brass-500) / 0.18) 30%,
			rgb(var(--amethyst-500) / 0.32) 62%,
			rgb(var(--border-strong) / 0.5));
		-webkit-mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0);
		-webkit-mask-composite: xor;
		mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0);
		mask-composite: exclude;
		pointer-events: none;
	}
	.topbar {
		position: relative;
		display: flex; align-items: center; gap: 18px;
		padding: 10px 18px;
		background: linear-gradient(180deg, rgb(var(--bg-subtle) / 0.94), rgb(var(--bg) / 0.92));
		box-shadow: inset 0 1px 0 rgb(var(--fg) / 0.05);
	}
	.topbar::after {
		content: "";
		position: absolute; left: 0; right: 0; bottom: 0; height: 1px;
		pointer-events: none;
		background: linear-gradient(90deg,
			rgb(var(--brass-500) / 0.8),
			rgb(var(--brass-400) / 0.35) 28%,
			rgb(var(--amethyst-500) / 0.42) 58%,
			rgb(var(--amethyst-500) / 0) 86%);
	}
	.brand { display: flex; align-items: center; gap: 12px; flex: none; }
	.emblem { width: 34px; height: 34px; object-fit: contain; flex: none; filter: drop-shadow(0 2px 8px rgb(var(--brass-700) / 0.5)); }
	.brand-text { display: flex; flex-direction: column; line-height: 1.15; }
	.brand-name { font-family: Georgia, 'Times New Roman', serif; font-size: 16px; font-weight: 700; letter-spacing: 0.22em; color: rgb(var(--brass-300)); }
	.brand-sub { font-size: 9.5px; font-weight: 600; letter-spacing: 0.16em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.current { display: flex; align-items: center; gap: 10px; min-width: 0; flex: 1; }
	.dot { width: 8px; height: 8px; flex: none; border-radius: 999px; }
	.dot-success { background: rgb(var(--success)); box-shadow: 0 0 8px rgb(var(--success) / 0.6); }
	.dot-warning { background: rgb(var(--warning)); }
	.dot-danger { background: rgb(var(--danger)); }
	.dot-info { background: rgb(var(--brand-400)); }
	.dot-neutral { background: rgb(var(--fg-subtle)); }
	.current-name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 14px; font-weight: 600; }
	.phase-badge { display: inline-flex; align-items: center; gap: 6px; padding: 3px 10px; border-radius: 999px; font-size: 11px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; flex: none; }
	.tone-success { background: rgb(var(--success) / 0.14); color: rgb(var(--success)); }
	.tone-warning { background: rgb(var(--warning) / 0.14); color: rgb(var(--warning)); }
	.tone-danger { background: rgb(var(--danger) / 0.14); color: rgb(var(--danger)); }
	.tone-info { background: rgb(var(--brand-500) / 0.14); color: rgb(var(--brand-300)); }
	.tone-neutral { background: rgb(var(--fg) / 0.08); color: rgb(var(--fg-muted)); }
	.actions { display: flex; align-items: center; gap: 12px; flex: none; }
	.create-btn { display: inline-flex; align-items: center; gap: 8px; padding: 9px 16px; border: 1px solid rgb(var(--brass-500) / 0.55); border-radius: 14px; background: rgb(var(--brass-500) / 0.14); color: rgb(var(--brass-300)); font-size: 12.5px; font-weight: 700; letter-spacing: 0.06em; text-transform: uppercase; transition: background-color 150ms ease, border-color 150ms ease; }
	.create-btn:hover { background: rgb(var(--brass-500) / 0.24); border-color: rgb(var(--brass-400) / 0.8); }
	.rule { width: 1px; height: 24px; background: rgb(var(--border-strong)); }
	.window-controls { display: flex; gap: 3px; padding: 4px; border: 1px solid rgb(var(--border)); border-radius: 13px; background: rgb(var(--bg-elevated) / 0.72); }
	.win-btn { display: grid; place-items: center; width: 30px; height: 26px; border: 0; border-radius: 9px; background: transparent; color: rgb(var(--fg-muted)); transition: color 140ms ease, background-color 140ms ease; }
	.win-btn:hover { color: rgb(var(--fg)); background: rgb(var(--fg) / 0.09); }
	.win-btn:focus-visible { outline: 2px solid rgb(var(--amethyst-400) / 0.7); outline-offset: 1px; }
	.win-close:hover { background: rgb(var(--danger) / 0.16); color: rgb(var(--danger)); }
	.warn-strip { display: flex; align-items: center; gap: 10px; padding: 8px 18px; border-bottom: 1px solid rgb(var(--warning) / 0.3); background: rgb(var(--warning) / 0.08); font-size: 12.5px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.canvas-host { position: relative; flex: 1; min-height: 0; }
	@media (prefers-reduced-motion: reduce) {
		.create-btn, .win-btn { transition: none; }
	}
</style>
