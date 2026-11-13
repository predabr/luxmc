<script lang="ts">
	import { Activity, MemoryStick, Cpu, Wifi, Timer, Users, AlertTriangle, LoaderCircle, Gauge } from 'lucide-svelte';
	import { toast } from '$lib/stores/toasts.svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { updateLocalServer } from '$lib/api/localServers';
	import { platform } from '$lib/mcserver/state.svelte';
	import { formatGiB, formatRate, formatUptime } from '$lib/mcserver/version';

	let { actionBusy }: { actionBusy: boolean } = $props();

	let ramDraft = $state<number | null>(null);
	let ramTimer: ReturnType<typeof setTimeout> | undefined;
	let savingRam = $state(false);
	let clock = $state(Date.now());

	const running = $derived(platform.running);
	const allocated = $derived(ramDraft ?? platform.allocatedRamMb);
	const total = $derived(platform.totalRamMb);
	const usedPcPercent = $derived(total > 0 ? (platform.usedRamMb / total) * 100 : 0);
	const serverSharePercent = $derived(total > 0 ? (allocated / total) * 100 : 0);
	const unsafe = $derived(total > 0 && allocated > total * 0.75);
	const tooLittle = $derived(platform.loader.kind !== 'vanilla' && allocated < 3072);
	const uptime = $derived(platform.runningSince ? formatUptime(clock - platform.runningSince) : '—');

	const telemetry = $derived(platform.telemetry);
	const heapUsedMb = $derived(telemetry?.online ? telemetry.heapUsedMb ?? null : null);
	const heapMaxMb = $derived(telemetry?.heapMaxMb ?? platform.allocatedRamMb);
	const heapPercent = $derived(heapUsedMb !== null && heapMaxMb > 0 ? Math.min(100, (heapUsedMb / heapMaxMb) * 100) : 0);
	const processMemoryMb = $derived(telemetry?.online ? telemetry.processMemoryMb ?? null : null);
	const processCpuPercent = $derived(telemetry?.online ? telemetry.processCpuPercent ?? null : null);
	const telemetryNote = $derived(
		!running ? 'Servidor offline — sem métricas de processo.' : (telemetry?.detail ?? 'Coletando métricas reais do processo Java…')
	);

	const profiles = [
		{
			id: 'economico',
			name: 'Econômico',
			description: 'Memória reduzida, distância de visão 4 e simulação 3. Para mundos pequenos em máquinas modestas.',
			apply: () => ({ ram: Math.max(1024, Math.min(2048, Math.floor((total * 0.25) / 512) * 512 || 2048)), view: 4, simulation: 3 })
		},
		{
			id: 'equilibrado',
			name: 'Equilibrado',
			description: 'Memória sugerida pelo LUXMC, distância de visão 8 e simulação 5. Equilíbrio entre mundo visível e estabilidade.',
			apply: () => ({ ram: Math.max(2048, Math.min(total ? Math.floor((total * 0.4) / 512) * 512 : 4096, 6144)), view: 8, simulation: 5 })
		},
		{
			id: 'desempenho',
			name: 'Desempenho',
			description: 'Mais memória, distância de visão 12 e simulação 8. Para máquinas com folga de memória.',
			apply: () => ({ ram: Math.max(4096, Math.min(total ? Math.floor((total * 0.6) / 512) * 512 : 6144, 8192)), view: 12, simulation: 8 })
		}
	];

	async function applyProfile(id: string) {
		const profile = profiles.find((item) => item.id === id);
		if (!profile || !platform.config) return;
		const { ram, view, simulation } = profile.apply();
		ramDraft = ram;
		await commitRam(ram);
		if (platform.settings) {
			try {
				const { saveServerSettings } = await import('$lib/api/localServers');
				const next = { ...platform.settings, viewDistance: view, simulationDistance: simulation };
				platform.settings = next;
				await saveServerSettings(platform.config.id, next);
				toast(`Perfil ${profile.name} aplicado: ${formatGiB(ram)}, visão ${view}, simulação ${simulation}.`, 'success');
			} catch (cause) {
				toast(serverMessage(cause), 'error');
			}
		}
	}

	function onRamInput(value: number) {
		ramDraft = value;
		if (ramTimer) clearTimeout(ramTimer);
		ramTimer = setTimeout(() => void commitRam(value), 500);
	}

	async function commitRam(value: number) {
		const config = platform.config;
		if (!config) return;
		if (value === config.ramMb) {
			ramDraft = null;
			return;
		}
		savingRam = true;
		try {
			await updateLocalServer(config.id, config.name, value, config.maxPlayers);
			ramDraft = null;
			toast(`Memória do servidor ajustada para ${formatGiB(value)}. Vale na próxima inicialização.`, 'success');
			await platform.refreshServers();
		} catch (cause) {
			ramDraft = null;
			toast(serverMessage(cause), 'error');
		} finally {
			savingRam = false;
		}
	}

	function spark(list: number[], max: number): string {
		if (list.length < 2) return '';
		const points = list
			.map((value, index) => `${(index / (list.length - 1)) * 100},${Math.max(2, Math.min(98, 100 - (value / Math.max(max, 1)) * 100))}`)
			.join(' ');
		return points;
	}

	$effect(() => {
		const timer = setInterval(() => (clock = Date.now()), 1000);
		return () => clearInterval(timer);
	});
</script>

<div class="grid">
	<header class="head">
		<div>
			<h1 class="title">Desempenho</h1>
			<p class="sub">Memória do servidor, recursos do computador e telemetria real do processo em execução.</p>
		</div>
	</header>

	<section class="card">
		<h2 class="card-title"><MemoryStick class="h-4 w-4 text-amethyst-400" /> Memória do servidor</h2>
		<div class="ram-grid">
			<div class="ram-stats">
				<div class="stat"><span class="stat-label">Total do computador</span><span class="stat-value font-mono">{total ? formatGiB(total) : 'detectando…'}</span></div>
				<div class="stat"><span class="stat-label">Em uso pelo sistema</span><span class="stat-value font-mono">{platform.usedRamMb ? formatGiB(platform.usedRamMb) : '—'}</span></div>
				<div class="stat"><span class="stat-label">RAM configurada (limite)</span><span class="stat-value font-mono">{formatGiB(platform.allocatedRamMb)}</span></div>
				<div class="stat"><span class="stat-label">RAM utilizada (heap JVM)</span><span class="stat-value font-mono">{heapUsedMb !== null ? formatGiB(heapUsedMb) : '—'}</span></div>
				<div class="stat"><span class="stat-label">Processo Java (memória total)</span><span class="stat-value font-mono">{processMemoryMb !== null ? formatGiB(processMemoryMb) : '—'}</span></div>
				<div class="stat"><span class="stat-label">Mín./máx. da JVM</span><span class="stat-value font-mono">{formatGiB(platform.allocatedRamMb)} fixo (−Xms = −Xmx)</span></div>
			</div>
			<div class="ram-slider">
				<div class="ram-head">
					<span class="stat-label">Alocação</span>
					<span class="ram-value font-mono">{formatGiB(allocated)}{savingRam ? ' · salvando…' : ''}</span>
				</div>
				<input
					type="range"
					min="512"
					max={total ? Math.min(16384, Math.floor(total / 512) * 512) : 8192}
					step="512"
					value={allocated}
					oninput={(event) => onRamInput(Number(event.currentTarget.value))}
					aria-label="Memória alocada ao servidor"
				/>
				<div class="ram-scale font-mono"><span>512 MiB</span><span>{total ? formatGiB(total) : '8 GiB'} do computador</span></div>
				{#if unsafe}
					<p class="warn-line"><AlertTriangle class="h-3.5 w-3.5 text-warning" /> Essa alocação compromete mais de 75% da memória do computador. Reduza para evitar travamentos.</p>
				{/if}
				{#if tooLittle}
					<p class="warn-line"><AlertTriangle class="h-3.5 w-3.5 text-warning" /> Servidores com mods precisam de pelo menos 3 GiB para carregar com estabilidade.</p>
				{/if}
				<div class="share-bar"><span class="share-fill" style="width:{Math.min(100, serverSharePercent)}%"></span></div>
				<p class="note">O servidor usa {serverSharePercent.toFixed(0)}% da memória física. A mudança entra em vigor na próxima inicialização.</p>
			</div>
		</div>
		<div class="profiles" role="group" aria-label="Perfis de recursos">
			{#each profiles as profile (profile.id)}
				<button type="button" class="profile" disabled={actionBusy || !platform.config} onclick={() => void applyProfile(profile.id)}>
					<span class="profile-name">{profile.name}</span>
					<span class="profile-desc">{profile.description}</span>
				</button>
			{/each}
		</div>
	</section>

	<section class="card">
		<h2 class="card-title"><Activity class="h-4 w-4 text-amethyst-400" /> Telemetria</h2>
		{#if running}
			<div class="charts">
				<div class="chart-box">
					<div class="chart-head"><span><MemoryStick class="h-3.5 w-3.5" /> RAM utilizada (heap JVM)</span><span class="font-mono">{heapUsedMb !== null ? `${formatGiB(heapUsedMb)} (${heapPercent.toFixed(0)}%)` : 'indisponível'}</span></div>
					<svg viewBox="0 0 100 40" preserveAspectRatio="none" class="chart" aria-hidden="true">
						<polyline points={spark(platform.heapHistory, Math.max(heapMaxMb, ...platform.heapHistory))} fill="none" stroke="rgb(var(--success))" stroke-width="1.6" vector-effect="non-scaling-stroke" />
					</svg>
					<p class="chart-note">Limite configurado: {formatGiB(heapMaxMb)}</p>
				</div>
				<div class="chart-box">
					<div class="chart-head"><span><Cpu class="h-3.5 w-3.5" /> CPU do processo Java</span><span class="font-mono">{processCpuPercent !== null ? `${processCpuPercent.toFixed(1)}%` : 'indisponível'}</span></div>
					<svg viewBox="0 0 100 40" preserveAspectRatio="none" class="chart" aria-hidden="true">
						<polyline points={spark(platform.procCpuHistory, 100)} fill="none" stroke="rgb(var(--amethyst-400))" stroke-width="1.6" vector-effect="non-scaling-stroke" />
					</svg>
				</div>
				<div class="chart-box">
					<div class="chart-head"><span><MemoryStick class="h-3.5 w-3.5" /> Memória do processo Java</span><span class="font-mono">{processMemoryMb !== null ? formatGiB(processMemoryMb) : 'indisponível'}</span></div>
					<svg viewBox="0 0 100 40" preserveAspectRatio="none" class="chart" aria-hidden="true">
						<polyline points={spark(platform.procMemHistory, Math.max(heapMaxMb, ...platform.procMemHistory))} fill="none" stroke="rgb(var(--brass-400))" stroke-width="1.6" vector-effect="non-scaling-stroke" />
					</svg>
				</div>
				<div class="chart-box">
					<div class="chart-head"><span><Cpu class="h-3.5 w-3.5" /> CPU do computador</span><span class="font-mono">{platform.cpuPercent.toFixed(0)}%</span></div>
					<svg viewBox="0 0 100 40" preserveAspectRatio="none" class="chart" aria-hidden="true">
						<polyline points={spark(platform.cpuHistory, 100)} fill="none" stroke="rgb(var(--amethyst-400))" stroke-width="1.6" vector-effect="non-scaling-stroke" />
					</svg>
				</div>
				<div class="chart-box">
					<div class="chart-head"><span><MemoryStick class="h-3.5 w-3.5" /> Memória do computador</span><span class="font-mono">{usedPcPercent.toFixed(0)}%</span></div>
					<svg viewBox="0 0 100 40" preserveAspectRatio="none" class="chart" aria-hidden="true">
						<polyline points={spark(platform.memHistory, 100)} fill="none" stroke="rgb(var(--brass-400))" stroke-width="1.6" vector-effect="non-scaling-stroke" />
					</svg>
				</div>
				<div class="chart-box">
					<div class="chart-head"><span><Wifi class="h-3.5 w-3.5" /> Rede (entrada + saída)</span><span class="font-mono">{formatRate(platform.netDownBps + platform.netUpBps)}</span></div>
					<svg viewBox="0 0 100 40" preserveAspectRatio="none" class="chart" aria-hidden="true">
						<polyline points={spark(platform.netHistory, Math.max(1024 * 1024, ...platform.netHistory))} fill="none" stroke="rgb(var(--brand-400))" stroke-width="1.6" vector-effect="non-scaling-stroke" />
					</svg>
				</div>
			</div>
			<div class="chips-row">
				<span class="pill"><Timer class="h-3.5 w-3.5" /> Ativo há {uptime}</span>
				<span class="pill"><Users class="h-3.5 w-3.5" /> {platform.playersOnline} / {platform.config?.maxPlayers ?? 0} jogadores</span>
				<span class="pill"><Gauge class="h-3.5 w-3.5" /> {platform.port ? `porta ${platform.port}` : 'sem porta'}</span>
				<span class="pill"><LoaderCircle class="h-3.5 w-3.5 animate-spin" /> amostras coletadas: {platform.cpuHistory.length}</span>
				<span class="pill">TPS: Não disponível</span>
				<span class="pill">MSPT: Não disponível</span>
			</div>
			<p class="note">{telemetryNote}</p>
			<p class="note">Métricas medidas de verdade: heap JVM via jcmd local, memória e CPU do processo Java, uso global do computador e tráfego de rede. TPS e MSPT só existem se o software do servidor os expuser; o LUXMC não inventa valores.</p>
		{:else}
			<div class="offline" role="status">
				<Activity class="h-5 w-5 text-fg-subtle" />
				<p>Servidor offline — sem métricas de processo. As medições reais aparecem aqui quando o servidor estiver em execução — sem dados inventados.</p>
			</div>
		{/if}
	</section>
</div>

<style>
	.grid { display: grid; gap: 18px; }
	.head { display: flex; flex-wrap: wrap; align-items: flex-end; justify-content: space-between; gap: 14px; }
	.title { font-family: Georgia, 'Times New Roman', serif; font-size: 24px; font-weight: 700; }
	.sub { margin-top: 4px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.card { padding: 16px 18px; border: 1px solid rgb(var(--border-strong)); border-radius: 18px; background: rgb(var(--bg-elevated) / 0.8); }
	.card-title { display: flex; align-items: center; gap: 8px; margin-bottom: 12px; font-size: 12px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.ram-grid { display: grid; gap: 20px; grid-template-columns: 1fr; }
	.ram-stats { display: grid; gap: 8px; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); }
	.stat { display: flex; flex-direction: column; gap: 3px; padding: 10px 12px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.4); }
	.stat-label { font-size: 10.5px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.stat-value { font-size: 13.5px; }
	.ram-head { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 8px; }
	.ram-value { font-size: 20px; font-weight: 700; color: rgb(var(--copper-400)); }
	.ram-scale { display: flex; justify-content: space-between; margin-top: 6px; font-size: 10.5px; color: rgb(var(--fg-subtle)); }
	input[type='range'] { width: 100%; accent-color: rgb(var(--amethyst-400)); }
	.share-bar { height: 5px; margin-top: 10px; border-radius: 999px; background: rgb(var(--fg) / 0.07); overflow: hidden; }
	.share-fill { display: block; height: 100%; border-radius: 999px; background: linear-gradient(90deg, rgb(var(--copper-500)), rgb(var(--brass-400))); transition: width 300ms ease; }
	.warn-line { display: flex; align-items: center; gap: 8px; margin-top: 8px; font-size: 12.5px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.note { margin-top: 8px; font-size: 11.5px; line-height: 1.6; color: rgb(var(--fg-subtle)); }
	.profiles { display: grid; gap: 10px; margin-top: 16px; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); }
	.profile { display: flex; flex-direction: column; gap: 4px; padding: 12px 14px; text-align: left; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: rgb(var(--bg-overlay) / 0.5); transition: border-color 140ms ease; }
	.profile:hover:not(:disabled) { border-color: rgb(var(--brass-500) / 0.55); }
	.profile:disabled { opacity: 0.5; cursor: not-allowed; }
	.profile-name { font-size: 13.5px; font-weight: 700; color: rgb(var(--brass-300)); }
	.profile-desc { font-size: 11.5px; line-height: 1.55; color: rgb(var(--fg-muted)); }
	.charts { display: grid; gap: 12px; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); }
	.chart-box { padding: 10px 12px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.4); }
	.chart-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; font-size: 12px; color: rgb(var(--fg-muted)); }
	.chart-head span:first-child { display: inline-flex; align-items: center; gap: 6px; }
	.chart-note { margin-top: 4px; font-size: 10.5px; color: rgb(var(--fg-subtle)); }
	.chart { display: block; width: 100%; height: 40px; }
	.chips-row { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 14px; }
	.pill { display: inline-flex; align-items: center; gap: 7px; padding: 6px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; font-size: 11.5px; font-weight: 600; color: rgb(var(--fg-muted)); }
	.offline { display: flex; align-items: center; gap: 12px; padding: 18px; border: 1px dashed rgb(var(--border-strong)); border-radius: 16px; font-size: 13px; line-height: 1.7; color: rgb(var(--fg-muted)); }
	@media (min-width: 1024px) {
		.ram-grid { grid-template-columns: minmax(260px, 1fr) 1.4fr; }
	}
	@media (prefers-reduced-motion: reduce) {
		.share-fill { transition: none; }
	}
</style>
