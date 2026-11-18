<script lang="ts">
	import {
		Play,
		Square,
		FolderOpen,
		Copy,
		Network,
		Users,
		MemoryStick,
		Clock,
		TerminalSquare,
		Activity,
		CheckCircle2,
		LoaderCircle,
		Cpu,
		Globe2,
		MapPin,
		ShieldCheck,
		Router,
		Radar
	} from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { openLocalServerFolder } from '$lib/api/localServers';
	import { platform } from '$lib/mcserver/state.svelte';
	import { softwareByKind } from '$lib/mcserver/software';
	import { formatGiB, formatUptime, formatRate } from '$lib/mcserver/version';

	let { busy, diagnostic, onStart, onStop, onDiagnostic, onCreate }: {
		busy: boolean;
		diagnostic: string | null;
		onStart: () => Promise<void> | void;
		onStop: () => Promise<void> | void;
		onDiagnostic: () => Promise<void> | void;
		onCreate: () => void;
	} = $props();

	const config = $derived(platform.config);
	const running = $derived(platform.running);
	const transitioning = $derived(platform.transition);
	const software = $derived(softwareByKind(platform.loader.kind));
	const port = $derived(platform.port);
	const localAddress = $derived(port ? `127.0.0.1:${port}` : '—');
	const lanAddresses = $derived(platform.diagnostic?.lanAddresses ?? []);
	const publicAddress = $derived(
		platform.diagnostic?.publicAddress && port ? `${platform.diagnostic.publicAddress}:${port}` : null
	);
	const checks = $derived(platform.diagnostic?.checks ?? []);
	const memUsedPercent = $derived(platform.totalRamMb > 0 ? (platform.usedRamMb / platform.totalRamMb) * 100 : 0);

	const telemetry = $derived(platform.telemetry);
	const procOnline = $derived(telemetry?.online ?? false);
	const heapUsedMb = $derived(procOnline ? telemetry?.heapUsedMb ?? null : null);
	const heapMaxMb = $derived(telemetry?.heapMaxMb ?? config?.ramMb ?? null);
	const processMemoryMb = $derived(procOnline ? telemetry?.processMemoryMb ?? null : null);
	const processCpuPercent = $derived(procOnline ? telemetry?.processCpuPercent ?? null : null);
	const heapPercent = $derived(heapUsedMb !== null && heapMaxMb ? Math.min(100, (heapUsedMb / heapMaxMb) * 100) : 0);
	const telemetryNote = $derived(
		!running
			? 'Servidor offline — sem métricas de processo.'
			: (telemetry?.detail ?? 'Coletando métricas reais do processo Java…')
	);

	async function copyAddress() {
		if (!port) {
			toast('O servidor está offline; ligue-o para obter o endereço.', 'warning');
			return;
		}
		const target = lanAddresses[0] ?? localAddress;
		try {
			await navigator.clipboard.writeText(target);
		} catch {
			toast('Não foi possível copiar automaticamente.', 'error');
		}
	}

	async function openFolder() {
		if (!config) return;
		try {
			await openLocalServerFolder(config.id);
		} catch (cause) {
			toast(serverMessage(cause), 'error');
		}
	}

	let clock = $state(Date.now());
	$effect(() => {
		const timer = setInterval(() => (clock = Date.now()), 1000);
		return () => clearInterval(timer);
	});
	const uptime = $derived(platform.runningSince ? formatUptime(clock - platform.runningSince) : '—');
</script>

{#if config}
	<div class="grid gap-5">
		<section class="card head-card">
			<div class="head-main">
				<h1 class="head-title">{config.name}</h1>
				<p class="head-sub">
					Minecraft {config.mcVersion} · {software.name}{platform.loader.version ? ` ${platform.loader.version}` : ''}
				</p>
				<div class="head-meta">
					<span><TerminalSquare class="h-3.5 w-3.5" /> {localAddress}</span>
					<span><Users class="h-3.5 w-3.5" /> {platform.playersOnline} / {config.maxPlayers} jogadores</span>
					<span><Clock class="h-3.5 w-3.5" /> {uptime}</span>
					<span><MemoryStick class="h-3.5 w-3.5" /> RAM configurada: {formatGiB(config.ramMb)}</span>
				</div>
			</div>
			<div class="head-actions">
				{#if running}
					<button type="button" class="btn stop" disabled={busy || transitioning} onclick={onStop}>
						{#if transitioning}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Square class="h-4 w-4" />{/if}
						Parar servidor
					</button>
				{:else}
					<button type="button" class="btn start" disabled={busy || transitioning} onclick={onStart}>
						{#if transitioning}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Play class="h-4 w-4" />{/if}
						Iniciar servidor
					</button>
				{/if}
				<div class="head-secondary">
					<button type="button" class="btn ghost" onclick={copyAddress}><Copy class="h-4 w-4" /> Copiar endereço</button>
					<button type="button" class="btn ghost" onclick={openFolder}><FolderOpen class="h-4 w-4" /> Abrir pasta</button>
					<button type="button" class="btn ghost" onclick={() => void onDiagnostic()}><Network class="h-4 w-4" /> Diagnosticar conexão</button>
				</div>
			</div>
		</section>

		{#if diagnostic}
			<div class="diag" role="status"><CheckCircle2 class="h-4 w-4 shrink-0 text-success" /> {diagnostic}</div>
		{/if}

		<div class="grid gap-5 lg:grid-cols-3">
			<section class="card">
				<h2 class="card-title"><Cpu class="h-4 w-4 text-amethyst-400" /> Processo Java agora</h2>
				{#if procOnline}
					<div class="metric">
						<div class="metric-head"><span>CPU do processo Java</span><span class="font-mono">{processCpuPercent !== null ? `${processCpuPercent.toFixed(1)}%` : 'indisponível'}</span></div>
						<div class="bar"><span class="bar-fill" style="width:{processCpuPercent !== null ? Math.min(100, processCpuPercent) : 0}%"></span></div>
					</div>
					<div class="metric">
						<div class="metric-head"><span>RAM utilizada (heap JVM)</span><span class="font-mono">{heapUsedMb !== null ? formatGiB(heapUsedMb) : 'indisponível'} / {heapMaxMb ? formatGiB(heapMaxMb) : '—'}</span></div>
						<div class="bar"><span class="bar-fill" style="width:{heapPercent}%"></span></div>
					</div>
					<div class="metric">
						<div class="metric-head"><span>Memória do processo Java</span><span class="font-mono">{processMemoryMb !== null ? formatGiB(processMemoryMb) : 'indisponível'}</span></div>
						<div class="bar"><span class="bar-fill" style="width:{heapMaxMb ? Math.min(100, (processMemoryMb ?? 0) / heapMaxMb * 100) : 0}%"></span></div>
					</div>
					<div class="metric">
						<div class="metric-head"><span>CPU do computador</span><span class="font-mono">{platform.cpuPercent.toFixed(0)}%</span></div>
						<div class="bar"><span class="bar-fill" style="width:{Math.min(100, platform.cpuPercent)}%"></span></div>
				</div>
					<div class="metric">
						<div class="metric-head"><span>Memória do computador</span><span class="font-mono">{platform.usedRamMb > 0 ? formatGiB(platform.usedRamMb) : '—'} / {platform.totalRamMb ? formatGiB(platform.totalRamMb) : '—'}</span></div>
						<div class="bar"><span class="bar-fill" style="width:{Math.min(100, memUsedPercent)}%"></span></div>
					</div>
					<div class="metric">
						<div class="metric-head"><span>Rede</span><span class="font-mono">{formatRate(platform.netDownBps + platform.netUpBps)}</span></div>
						<div class="bar"><span class="bar-fill" style="width:{Math.min(100, ((platform.netDownBps + platform.netUpBps) / (2 * 1024 * 1024)) * 100)}%"></span></div>
					</div>
				{:else}
					<p class="big">—</p>
				{/if}
				<p class="note">{telemetryNote}</p>
			</section>

			<section class="card">
				<h2 class="card-title"><Users class="h-4 w-4 text-amethyst-400" /> Jogadores</h2>
				{#if running}
					<p class="big">{platform.playersOnline}<span class="big-sub"> / {config.maxPlayers}</span></p>
					<p class="note">Contagem obtida por ping direto ao servidor.</p>
				{:else}
					<p class="big">—</p>
					<p class="note">O servidor está offline. Inicie-o para ver os jogadores conectados.</p>
				{/if}
				<dl class="kv">
					<div><dt>Software</dt><dd>{software.name}</dd></div>
					<div><dt>Versão</dt><dd class="font-mono">{config.mcVersion}</dd></div>
					<div><dt>Modo de jogo</dt><dd>{platform.settings?.gamemode ?? '—'}</dd></div>
					<div><dt>Dificuldade</dt><dd>{platform.settings?.difficulty ?? '—'}</dd></div>
				</dl>
			</section>

			<section class="card">
				<h2 class="card-title"><TerminalSquare class="h-4 w-4 text-amethyst-400" /> Atividade recente do console</h2>
				{#if platform.activeForSelected && platform.selectedState?.console.length}
					<div class="console-tail custom-scrollbar">
						{#each platform.selectedState.console.slice(-9) as line (line.sequence)}
							<p class="log-line" class:log-warn={line.stream === 'stderr'}>{line.text}</p>
						{/each}
					</div>
					<a class="more" href="/hosting/server?section=console">Abrir o console completo →</a>
				{:else}
					<p class="note">Sem saída de console. As mensagens aparecem aqui quando o servidor inicia.</p>
				{/if}
			</section>
		</div>

		<div class="grid gap-5 lg:grid-cols-2">
			<section class="card">
				<h2 class="card-title"><Network class="h-4 w-4 text-amethyst-400" /> Endereços de conexão</h2>
				<dl class="addr-list">
					<div class="addr-row">
						<dt><TerminalSquare class="h-3.5 w-3.5" /> Local (este computador)</dt>
						<dd class="font-mono">{localAddress}</dd>
					</div>
					<div class="addr-row">
						<dt><MapPin class="h-3.5 w-3.5" /> Rede local (LAN)</dt>
						<dd class="font-mono">
							{#if lanAddresses.length > 0}
								{lanAddresses.join(', ')}
							{:else if running}
								Execute o diagnóstico para listar os endereços da rede.
							{:else}
								Servidor offline — sem endereço de rede.
							{/if}
						</dd>
					</div>
					<div class="addr-row">
						<dt><Globe2 class="h-3.5 w-3.5" /> Internet (outra rede)</dt>
						<dd class="font-mono">
							{#if publicAddress}
								{publicAddress}
							{:else}
								Não verificado — requer encaminhamento de porta e um teste fora da rede.
							{/if}
						</dd>
					</div>
				</dl>
				<p class="note">O servidor escuta em todas as interfaces (server-ip vazio) na porta {port ?? 'configurada'}. Endereços públicos só aparecem após consulta real; o LUXMC não afirma acesso externo sem teste.</p>
			</section>

			<section class="card">
				<h2 class="card-title"><Activity class="h-4 w-4 text-amethyst-400" /> Recursos do computador</h2>
				<div class="metric">
					<div class="metric-head"><span>CPU do computador</span><span class="font-mono">{platform.cpuPercent.toFixed(0)}%</span></div>
					<div class="bar"><span class="bar-fill" style="width:{Math.min(100, platform.cpuPercent)}%"></span></div>
				</div>
				<div class="metric">
					<div class="metric-head"><span>Memória do computador</span><span class="font-mono">{platform.usedRamMb > 0 ? formatGiB(platform.usedRamMb) : '—'} / {platform.totalRamMb ? formatGiB(platform.totalRamMb) : '—'}</span></div>
					<div class="bar"><span class="bar-fill" style="width:{Math.min(100, memUsedPercent)}%"></span></div>
				</div>
				<div class="metric">
					<div class="metric-head"><span>Rede</span><span class="font-mono">{formatRate(platform.netDownBps + platform.netUpBps)}</span></div>
					<div class="bar"><span class="bar-fill" style="width:{Math.min(100, ((platform.netDownBps + platform.netUpBps) / (2 * 1024 * 1024)) * 100)}%"></span></div>
				</div>
				<p class="note">Medições reais do computador, atualizadas a cada poucos segundos.</p>
			</section>
		</div>

		{#if checks.length > 0}
			<section class="card">
				<h2 class="card-title"><ShieldCheck class="h-4 w-4 text-amethyst-400" /> Diagnóstico de conexão — 12 verificações</h2>
				<div class="checks">
					{#each checks as check (check.id)}
						<div class="check check-{check.status}">
							<div class="check-head">
								<span class="check-label">
									{#if check.status === 'approved'}<CheckCircle2 class="h-3.5 w-3.5" />{:else}<Radar class="h-3.5 w-3.5" />{/if}
									{check.label}
								</span>
								<span class="badge badge-{check.status}">{check.statusLabel}</span>
							</div>
							<p class="check-detail">{check.detail}</p>
						</div>
					{/each}
				</div>
				{#if platform.diagnostic?.generatedAt}
					<p class="note">Gerado em {new Date(platform.diagnostic.generatedAt).toLocaleString('pt-BR')}.</p>
				{/if}
				{#if platform.diagnostic && !platform.diagnostic.minecraftResponding && running}
					<p class="note">O ping de status (Server List Ping) ainda não respondeu; os itens abaixo mostram exatamente onde está o bloqueio.</p>
				{/if}
			</section>
		{/if}
	</div>
{:else}
	<div class="empty-inline">
		<p>Nenhum servidor selecionado.</p>
		<button type="button" class="btn start" onclick={onCreate}>Criar servidor</button>
	</div>
{/if}

<style>
	.grid { display: grid; gap: 20px; }
	.card { padding: 18px 20px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: linear-gradient(165deg, rgb(var(--bg-elevated) / 0.9), rgb(var(--bg-subtle) / 0.7)); }
	.head-card { display: flex; flex-wrap: wrap; gap: 20px; justify-content: space-between; }
	.head-main { min-width: 0; }
	.head-title { font-family: Georgia, 'Times New Roman', serif; font-size: 26px; font-weight: 700; }
	.head-sub { margin-top: 4px; font-size: 13.5px; color: rgb(var(--brass-300)); }
	.head-meta { display: flex; flex-wrap: wrap; gap: 14px; margin-top: 12px; font-size: 12.5px; color: rgb(var(--fg-muted)); }
	.head-meta span { display: inline-flex; align-items: center; gap: 6px; }
	.head-actions { display: flex; flex-direction: column; gap: 10px; }
	.head-secondary { display: flex; flex-wrap: wrap; gap: 8px; }
	.btn { display: inline-flex; align-items: center; justify-content: center; gap: 8px; padding: 10px 16px; border-radius: 14px; font-size: 13px; font-weight: 700; border: 1px solid rgb(var(--border-strong)); color: rgb(var(--fg)); background: rgb(var(--fg) / 0.05); transition: background-color 140ms ease, border-color 140ms ease; }
	.btn:hover:not(:disabled) { border-color: rgb(var(--brass-500) / 0.5); }
	.btn:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 2px; }
	.btn:disabled { opacity: 0.5; cursor: not-allowed; }
	.btn.start { border-color: rgb(var(--success) / 0.45); background: rgb(var(--success) / 0.12); color: rgb(var(--success)); }
	.btn.start:hover:not(:disabled) { background: rgb(var(--success) / 0.2); }
	.btn.stop { border-color: rgb(var(--danger) / 0.4); background: rgb(var(--danger) / 0.1); color: rgb(var(--danger)); }
	.btn.stop:hover:not(:disabled) { background: rgb(var(--danger) / 0.18); }
	.btn.ghost { padding: 8px 12px; font-size: 12px; color: rgb(var(--fg-muted)); }
	.diag { display: flex; align-items: center; gap: 10px; padding: 10px 14px; border: 1px solid rgb(var(--success) / 0.3); border-radius: 16px; background: rgb(var(--success) / 0.06); font-size: 13px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.card-title { display: flex; align-items: center; gap: 8px; margin-bottom: 14px; font-size: 12px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.metric { margin-bottom: 12px; }
	.metric-head { display: flex; justify-content: space-between; margin-bottom: 6px; font-size: 12px; color: rgb(var(--fg-muted)); }
	.bar { height: 6px; border-radius: 999px; background: rgb(var(--fg) / 0.07); overflow: hidden; }
	.bar-fill { display: block; height: 100%; border-radius: 999px; background: linear-gradient(90deg, rgb(var(--amethyst-500)), rgb(var(--amethyst-300))); transition: width 400ms ease; }
	.note { margin-top: 10px; font-size: 11.5px; line-height: 1.6; color: rgb(var(--fg-subtle)); }
	.big { font-size: 34px; font-weight: 700; color: rgb(var(--fg)); }
	.big-sub { font-size: 16px; font-weight: 500; color: rgb(var(--fg-muted)); }
	.kv { display: grid; gap: 6px; margin-top: 14px; }
	.kv > div { display: flex; justify-content: space-between; gap: 12px; font-size: 12.5px; }
	.kv dt { color: rgb(var(--fg-subtle)); }
	.kv dd { color: rgb(var(--fg)); }
	.console-tail { display: flex; flex-direction: column; gap: 2px; max-height: 180px; overflow-y: auto; }
	.log-line { font-family: 'JetBrains Mono', ui-monospace, monospace; font-size: 10.5px; line-height: 1.65; color: rgb(var(--fg-muted)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.log-warn { color: rgb(var(--warning)); }
	.more { display: inline-block; margin-top: 10px; font-size: 12.5px; font-weight: 600; color: rgb(var(--brand-300)); }
	.more:hover { text-decoration: underline; }
	.empty-inline { display: flex; flex-direction: column; align-items: center; gap: 14px; padding: 80px 0; }
	.addr-list { display: grid; gap: 10px; }
	.addr-row { display: flex; flex-direction: column; gap: 4px; padding: 10px 12px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.4); }
	.addr-row dt { display: inline-flex; align-items: center; gap: 6px; font-size: 10.5px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.addr-row dd { font-size: 12.5px; word-break: break-all; color: rgb(var(--fg)); }
	.checks { display: grid; gap: 10px; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); }
	.check { padding: 10px 12px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.4); }
	.check-approved { border-color: rgb(var(--success) / 0.35); }
	.check-attention { border-color: rgb(var(--warning) / 0.35); }
	.check-failed { border-color: rgb(var(--danger) / 0.35); }
	.check-unverified { border-color: rgb(var(--border-strong)); }
	.check-head { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
	.check-label { display: inline-flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 700; color: rgb(var(--fg)); }
	.check-detail { margin-top: 6px; font-size: 11.5px; line-height: 1.6; color: rgb(var(--fg-muted)); }
	.badge { padding: 3px 8px; border-radius: 999px; font-size: 10.5px; font-weight: 700; letter-spacing: 0.04em; white-space: nowrap; }
	.badge-approved { background: rgb(var(--success) / 0.14); color: rgb(var(--success)); }
	.badge-attention { background: rgb(var(--warning) / 0.14); color: rgb(var(--warning)); }
	.badge-failed { background: rgb(var(--danger) / 0.14); color: rgb(var(--danger)); }
	.badge-unverified { background: rgb(var(--fg) / 0.08); color: rgb(var(--fg-muted)); }
	@media (prefers-reduced-motion: reduce) {
		.bar-fill, .btn { transition: none; }
	}
</style>
