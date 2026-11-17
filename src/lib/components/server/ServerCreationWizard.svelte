<script lang="ts">
	import { onMount } from 'svelte';
	import { fade, fly } from 'svelte/transition';
	import { quintOut } from 'svelte/easing';
	import {
		X,
		ArrowRight,
		ArrowLeft,
		LoaderCircle,
		ShieldCheck,
		AlertTriangle,
		Boxes,
		Coffee,
		Cpu,
		FlaskConical,
		Blocks,
		Puzzle,
		Globe2,
		Box,
		Package,
		RefreshCw,
		Dice5,
		Check,
		ExternalLink
	} from 'lucide-svelte';
	import { serverMessage } from '$lib/utils/serverMessages';
	import { versionsList, loadersVersions } from '$lib/api/launch';
import { javaScan, javaInstall } from '$lib/api/java';
	import type { JavaInstallStatus } from '$lib/api/types';
	import {
		createLocalServer,
		serverManage,
		serverSettings,
		saveServerSettings,
		startLocalServer,
		localServerList,
		type LocalServerConfig,
		type ServerSettings
	} from '$lib/api/localServers';
	import { SERVER_SOFTWARE, softwareByKind, type SoftwareKind } from '$lib/mcserver/software';
	import GlassSelect from './GlassSelect.svelte';
	import { requiredJavaMajor, formatGiB } from '$lib/mcserver/version';
	import { gamerulesForVersion, GAMERULE_CATEGORIES, type ResolvedGamerule } from '$lib/mcserver/gamerules';
	import InstallTimeline, { type TimelineStep } from './InstallTimeline.svelte';

	let { onclose, oncreated }: { onclose: () => void; oncreated: (server: LocalServerConfig) => void } = $props();

	const TOTAL_STEPS = 5;
	const STEP_TITLES = ['Identidade do servidor', 'Software do servidor', 'Java', 'Mundo e rede', 'Regras e confirmação'];

	let step = $state(0);
	let busy = $state(false);
	let error = $state('');
	let installing = $state(false);

	let name = $state('');
	let description = $state('');
	let showSnapshots = $state(false);
	let mcVersion = $state('');
	let versions = $state<Array<{ id: string; versionType: string }>>([]);
	let latestRelease = $state('');
	let latestSnapshot = $state('');

	let kind = $state<SoftwareKind>('vanilla');
	let loaderVersion = $state('');
	let loaderVersions = $state<Array<{ id: string; stable: boolean }>>([]);
	let loadingLoaders = $state(false);

	let runtimes = $state<JavaInstallStatus[]>([]);
	let javaScanning = $state(true);
	let javaInstalling = $state(false);

	let seed = $state('');
	let gamemode = $state<ServerSettings['gamemode']>('survival');
	let difficulty = $state<ServerSettings['difficulty']>('normal');

	const GAMEMODE_OPTIONS: Array<{ value: ServerSettings['gamemode']; label: string }> = [
		{ value: 'survival', label: 'Sobrevivência' },
		{ value: 'creative', label: 'Criativo' },
		{ value: 'adventure', label: 'Aventura' },
		{ value: 'spectator', label: 'Espectador' }
	];
	const DIFFICULTY_OPTIONS: Array<{ value: ServerSettings['difficulty']; label: string }> = [
		{ value: 'peaceful', label: 'Pacífico' },
		{ value: 'easy', label: 'Fácil' },
		{ value: 'normal', label: 'Normal' },
		{ value: 'hard', label: 'Difícil' }
	];
	let hardcore = $state(false);
	let structures = $state(true);
	let pvp = $state(true);
	let commandBlocks = $state(false);
	let allowFlight = $state(false);
	let onlineMode = $state(true);
	let whitelist = $state(false);
	let maxPlayers = $state(10);
	let spawnProtection = $state(16);
	let viewDistance = $state(6);
	let simulationDistance = $state(4);
	let ramMb = $state(2048);
	let totalRamMb = $state(0);
	let keepInventory = $state(false);
	let daylightCycle = $state(true);
	let naturalRegeneration = $state(true);
	let mobSpawning = $state(true);
	let monsterSpawning = $state(true);
	let gameruleSearch = $state('');
	let gameruleCategory = $state<string>('all');

	let eula = $state(false);
	let dialog = $state<HTMLDialogElement | null>(null);
	let timeline = $state<InstallTimeline | null>(null);
	let overviewPoll: ReturnType<typeof setInterval> | null = null;

	const selectedSoftware = $derived(softwareByKind(kind));
	const javaMajor = $derived(mcVersion ? requiredJavaMajor(mcVersion) : 0);
	const compatibleRuntimes = $derived(runtimes.filter((runtime) => runtime.installed && runtime.major >= javaMajor));
	const releases = $derived(versions.filter((version) => version.versionType === 'release'));
	const snapshots = $derived(versions.filter((version) => version.versionType === 'snapshot'));
	const versionChoices = $derived(showSnapshots ? [...snapshots, ...releases] : releases);
	const catalog = $derived(mcVersion ? gamerulesForVersion(mcVersion) : []);
	const filteredRules = $derived.by(() => {
		const query = gameruleSearch.trim().toLowerCase();
		return catalog.filter(
			(rule) =>
				(gameruleCategory === 'all' || rule.category === gameruleCategory) &&
				(!query || rule.id.toLowerCase().includes(query) || rule.label.toLowerCase().includes(query))
		);
	});
	const categoryLabel = $derived.by(() => {
		const counts = new Map<string, number>();
		for (const rule of catalog) counts.set(rule.category, (counts.get(rule.category) ?? 0) + 1);
		return counts;
	});

	const recommendedRam = $derived.by(() => {
		if (!totalRamMb) return 2048;
		const base = kind === 'vanilla' ? 2048 : 4096;
		const capped = Math.min(Math.floor((totalRamMb * 0.5) / 512) * 512, 8192);
		return Math.max(1024, Math.min(base, capped));
	});

	const ramTooHigh = $derived(totalRamMb > 0 && ramMb > totalRamMb * 0.75);
	const ramTooLow = $derived(kind !== 'vanilla' && ramMb < 3072);
	const ramInvalid = $derived(ramMb < 512 || ramMb > 16384);

	function setGamerule(id: string, value: boolean) {
		if (id === 'keepInventory') keepInventory = value;
		else if (id === 'doDaylightCycle') daylightCycle = value;
		else if (id === 'naturalRegeneration') naturalRegeneration = value;
		else if (id === 'doMobSpawning') mobSpawning = value;
		else if (id === 'spawnMonsters') monsterSpawning = value;
	}

	const gameruleValues = $derived<Record<string, boolean>>({
		keepInventory,
		doDaylightCycle: daylightCycle,
		naturalRegeneration,
		doMobSpawning: mobSpawning,
		spawnMonsters: monsterSpawning
	});

	const identityValid = $derived(Boolean(name.trim()) && mcVersion);
	const softwareValid = $derived(kind === 'vanilla' || Boolean(loaderVersion));
	const worldValid = $derived(
		maxPlayers >= 1 && maxPlayers <= 999 && !ramInvalid && viewDistance >= 3 && viewDistance <= 32 && simulationDistance >= 3 && simulationDistance <= 32
	);

	const canAdvance = $derived(
		(step === 0 && identityValid) || (step === 1 && softwareValid) || step === 2 || (step === 3 && worldValid) || step === 4
	);

	const INSTALL_STEPS: TimelineStep[] = [
		{ id: 'validate', label: 'Validando configuração', detail: 'Conferindo nome, versão e compatibilidade.' },
		{ id: 'java', label: 'Conferindo o Java', detail: 'Verificando runtimes instalados no computador.' },
		{ id: 'folder', label: 'Preparando a pasta do servidor', detail: 'Criando a estrutura de arquivos.' },
		{ id: 'eula', label: 'Aplicando a EULA aceita', detail: 'Gravando a aceitação do contrato de licença.' },
		{ id: 'loader', label: 'Instalando o software selecionado', detail: 'Baixando e executando o instalador oficial.' },
		{ id: 'world', label: 'Gerando a configuração do mundo', detail: 'Aplicando semente, modo de jogo e regras.' },
		{ id: 'prepare', label: 'Preparando a inicialização', detail: 'Resolvendo arquivos do Minecraft.' },
		{ id: 'download', label: 'Baixando arquivos do servidor', detail: 'Transferindo componentes oficiais.' },
		{ id: 'start', label: 'Iniciando o servidor', detail: 'Gerando o mundo pela primeira vez.' },
		{ id: 'verify', label: 'Verificando a inicialização', detail: 'Aguardando o servidor aceitar conexões.' },
		{ id: 'done', label: 'Instalação concluída', detail: 'O servidor está pronto para receber jogadores.' }
	];

	onMount(() => {
		void versionsList()
			.then((result) => {
				versions = result.versions;
				latestRelease = result.latestRelease;
				latestSnapshot = result.latestSnapshot;
				mcVersion = result.latestRelease;
			})
			.catch((cause) => (error = serverMessage(cause)));
		void javaScan()
			.then((result) => (runtimes = result.runtimes))
			.catch(() => (runtimes = []))
			.finally(() => (javaScanning = false));
		void import('$lib/api/system').then(({ getSystemSpecs }) =>
			getSystemSpecs()
				.then((specs) => {
					totalRamMb = specs.totalRamMb;
					if (ramMb < recommendedRam) ramMb = recommendedRam;
				})
				.catch(() => {})
		);
		dialog?.showModal();
		return () => {
			if (overviewPoll) clearInterval(overviewPoll);
		};
	});

	let loaderSource = '';
	$effect(() => {
		const current = mcVersion;
		const kindNow = kind;
		if (!current || kindNow === 'vanilla') return;
		const key = `${kindNow}:${current}`;
		if (loaderSource === key) return;
		loaderSource = key;
		void loadLoaderVersions();
	});

	async function loadLoaderVersions() {
		if (kind === 'vanilla' || !mcVersion) {
			loaderVersion = '';
			loaderVersions = [];
			return;
		}
		loadingLoaders = true;
		loaderVersion = '';
		try {
			const result = await loadersVersions(kind, mcVersion);
			loaderVersions = result.versions;
			loaderVersion = result.versions.find((item) => item.stable)?.id ?? result.versions[0]?.id ?? '';
			if (!loaderVersion) error = 'Nenhuma versão deste software é compatível com o Minecraft selecionado.';
		} catch (cause) {
			error = serverMessage(cause);
		} finally {
			loadingLoaders = false;
		}
	}

	function pickSoftware(next: SoftwareKind) {
		const option = softwareByKind(next);
		if (!option.supported || next === kind) return;
		kind = next;
		ramMb = recommendedRam;
	}

	function randomSeed() {
		const value = (Math.floor(Math.random() * 4294967295) + 1).toString();
		seed = value;
	}

	function applyProfile(profile: 'economico' | 'equilibrado' | 'desempenho') {
		if (profile === 'economico') {
			viewDistance = 4;
			simulationDistance = 3;
			ramMb = Math.max(1024, Math.min(2048, recommendedRam));
		} else if (profile === 'equilibrado') {
			viewDistance = 8;
			simulationDistance = 5;
			ramMb = recommendedRam;
		} else {
			viewDistance = 12;
			simulationDistance = 8;
			ramMb = Math.min(Math.max(recommendedRam + 1024, 4096), totalRamMb ? Math.floor((totalRamMb * 0.6) / 512) * 512 : 6144);
		}
	}

	async function next() {
		error = '';
		if (!canAdvance) return;
		if (step === 1 && kind !== 'vanilla' && !loaderVersions.length) await loadLoaderVersions();
		if (step < TOTAL_STEPS - 1) step += 1;
	}

	function back() {
		error = '';
		if (step > 0) step -= 1;
	}

	async function create(): Promise<void> {
		if (!eula || busy) return;
		busy = true;
		error = '';
		installing = true;
		timeline?.begin();
		const trimmedName = name.trim();
		try {
			await timeline?.completeStep('validate', `${trimmedName} · Minecraft ${mcVersion} · ${selectedSoftware.name}`);
			const scan = runtimes.length ? runtimes : (await javaScan().catch(() => ({ runtimes: [] }))).runtimes;
			runtimes = scan;
			const compatible = scan.some((runtime) => runtime.installed && runtime.major >= javaMajor);
			await timeline?.completeStep('java', compatible ? `Java ${javaMajor}+ disponível neste computador.` : `Nenhum Java ${javaMajor}+ detectado; o LUXMC vai preparar um durante a inicialização.`);
			const server = await createLocalServer({ name: trimmedName, mcVersion, ramMb, maxPlayers, eulaAccepted: true });
			await timeline?.completeStep('folder', 'Estrutura criada com sucesso.');
			const defaults = await serverSettings(server.id);
			const nextSettings: ServerSettings = {
				...defaults,
				motd: description.trim() || trimmedName,
				seed: seed.trim() || null,
				gamemode,
				difficulty,
				hardcore,
				generateStructures: structures,
				pvp,
				commandBlocks,
				allowFlight,
				onlineMode,
				whitelist,
				spawnProtection,
				viewDistance,
				simulationDistance,
				keepInventory,
				daylightCycle,
				naturalRegeneration,
				mobSpawning,
				monsterSpawning
			};
			await saveServerSettings(server.id, nextSettings);
			await timeline?.completeStep('eula', 'eula=true gravado com a sua aceitação.');
			if (kind !== 'vanilla') {
				await serverManage({ action: 'set_loader', id: server.id, loader: { kind, version: loaderVersion } });
				await timeline?.completeStep('loader', `${selectedSoftware.name} ${loaderVersion} instalado pelo instalador oficial.`);
			} else {
				await timeline?.completeStep('loader', 'Vanilla selecionado: o arquivo oficial é baixado durante a inicialização.');
			}
			await timeline?.completeStep('world', `Semente ${seed.trim() ? 'personalizada' : 'aleatória'} · ${labelGamemode(gamemode)} · ${labelDifficulty(difficulty)}.`);

			overviewPoll = setInterval(() => {
				void localServerList().then((overview) => {
					const active = overview.active;
					if (active?.id !== server.id) return;
					void timeline?.trackPhase(active.phase, active.console.slice(-40).map((line) => line.text));
				}).catch(() => {});
			}, 1500);

			await startLocalServer(server.id);
			await timeline?.completeStep('prepare', 'Comando de inicialização preparado.');
			const overview = await waitForPhase(server.id, ['running', 'failed', 'stopped'], 240_000);
			if (!overview || overview.phase !== 'running') {
				await timeline?.failStep(
					'verify',
					overview?.error || 'A inicialização não terminou dentro do tempo esperado. Confira o console do servidor.'
				);
				return;
			}
			await timeline?.completeStep('verify', 'O servidor respondeu na porta local.');
			await timeline?.completeStep('done', 'Tudo pronto. Abrindo o painel de gerenciamento.');
			await new Promise((resolve) => setTimeout(resolve, 1800));
			if (overviewPoll) {
				clearInterval(overviewPoll);
				overviewPoll = null;
			}
			oncreated(server);
		} catch (cause) {
			error = serverMessage(cause);
			await timeline?.failStep('verify', 'A instalação não foi concluída. Revise a mensagem e tente novamente.');
		} finally {
			if (overviewPoll) {
				clearInterval(overviewPoll);
				overviewPoll = null;
			}
			busy = false;
		}
	}

	function waitForPhase(id: string, phases: string[], timeoutMs: number): Promise<{ phase: string; error: string | null } | null> {
		const deadline = Date.now() + timeoutMs;
		return new Promise((resolve) => {
			const poll = setInterval(async () => {
				try {
					const overview = await localServerList();
					const active = overview.active;
					if (active?.id !== id) {
						if (Date.now() > deadline) {
							clearInterval(poll);
							resolve(null);
						}
						return;
					}
					if (phases.includes(active.phase) || Date.now() > deadline) {
						clearInterval(poll);
						resolve({ phase: active.phase, error: active.error });
					}
				} catch {
					if (Date.now() > deadline) {
						clearInterval(poll);
						resolve(null);
					}
				}
			}, 1500);
		});
	}

	function labelGamemode(value: string): string {
		return { survival: 'Sobrevivência', creative: 'Criativo', adventure: 'Aventura', spectator: 'Espectador' }[value] ?? value;
	}

	function labelDifficulty(value: string): string {
		return { peaceful: 'Pacífico', easy: 'Fácil', normal: 'Normal', hard: 'Difícil' }[value] ?? value;
	}

	const softIcons: Record<SoftwareKind, typeof Boxes> = {
		vanilla: Package,
		fabric: Puzzle,
		forge: FlaskConical,
		neoforge: Blocks,
		quilt: Boxes,
		paper: Globe2,
		purpur: Globe2,
		spigot: Globe2
	};
</script>

<dialog
	bind:this={dialog}
	class="wizard-dialog text-fg"
	aria-labelledby="wizard-title"
	oncancel={(event) => {
		event.preventDefault();
		if (!busy) onclose();
	}}
>
	<div class="frame">
		<header class="header">
			<div class="brand">
				<img src="/branding/luxmc-emblem-128.png" alt="" aria-hidden="true" class="emblem" />
				<div>
					<p class="eyebrow">Luxmc · Servidor dedicado</p>
					<h2 id="wizard-title" class="title">{installing ? 'Instalando o servidor' : 'Criar servidor'}</h2>
				</div>
			</div>
			{#if !installing}
				<button type="button" class="icon-btn" aria-label="Fechar" onclick={onclose}>
					<X class="h-5 w-5" />
				</button>
			{/if}
		</header>

		{#if !installing}
			<nav class="progress" aria-label="Etapas de criação">
				{#each STEP_TITLES as title, index (title)}
					<div class="progress-step" class:current={index === step} class:past={index < step} aria-current={index === step ? 'step' : undefined}>
						<span class="dot">{#if index < step}<Check class="h-3 w-3" />{:else}{index + 1}{/if}</span>
						<span class="progress-label">{title}</span>
					</div>
					{#if index < STEP_TITLES.length - 1}<span class="bar" class:filled={index < step}></span>{/if}
				{/each}
			</nav>
		{/if}

		<div class="content custom-scrollbar">
			{#if error && !installing}
				<div role="alert" class="alert">{error}</div>
			{/if}

			{#if installing}
				<div class="installing">
					<InstallTimeline bind:this={timeline} steps={INSTALL_STEPS} />
					{#if error}
						<div role="alert" class="alert alert-install">
							<AlertTriangle class="h-4 w-4 shrink-0 text-warning" />
							<div>
								<p class="font-semibold text-fg">Falha na instalação</p>
								<p class="mt-1 text-xs leading-relaxed text-fg-muted">{error}</p>
							</div>
						</div>
					{/if}
				</div>
			{:else if step === 0}
				<div class="step-body" in:fly={{ y: 12, duration: 240, easing: quintOut }}>
					<div class="grid gap-4">
						<label class="field">
							<span>Nome do servidor</span>
							<input type="text" maxlength="64" bind:value={name} placeholder="Ex.: Aventuras do Vale" />
						</label>
						<label class="field">
							<span>Descrição <em class="hint-inline">(opcional)</em></span>
							<input type="text" maxlength="140" bind:value={description} placeholder="Uma frase para lembrar o propósito deste mundo" />
						</label>
						<div class="field">
							<span>Edição do Minecraft</span>
							<div class="edition-row">
								<div class="edition selected" aria-pressed="true">
									<Box class="h-5 w-5 text-brand-300" />
									<div>
										<p class="edition-name">Java Edition</p>
										<p class="edition-note">Edição original, com suporte completo a mods e plugins.</p>
									</div>
								</div>
								<div class="edition disabled" title="O LUXMC ainda não instala servidores Bedrock.">
									<Package class="h-5 w-5 text-fg-subtle" />
									<div>
										<p class="edition-name">Bedrock Edition</p>
										<p class="edition-note">Ainda não disponível nesta versão do LUXMC.</p>
									</div>
								</div>
							</div>
						</div>
						<div class="field">
							<span>Versão do Minecraft</span>
							<div class="release-toggle">
								<button type="button" class={!showSnapshots ? 'chip active' : 'chip'} onclick={() => (showSnapshots = false)}>
									Versões estáveis
								</button>
								<button type="button" class={showSnapshots ? 'chip active' : 'chip'} onclick={() => (showSnapshots = true)}>
									Incluir snapshots
								</button>
							</div>
							<GlassSelect
								ariaLabel="Versão do Minecraft"
								options={versionChoices.map((item) => ({
									value: item.id,
									label: item.id === latestRelease ? `${item.id} · mais recente estável` : item.id === latestSnapshot && showSnapshots ? `${item.id} · snapshot mais recente` : item.id
								}))}
								bind:value={mcVersion}
							/>
							<p class="hint">Lista obtida agora dos servidores oficiais da Mojang. {releases.length} versões estáveis disponíveis.</p>
						</div>
					</div>
				</div>
			{:else if step === 1}
				<div class="step-body" in:fly={{ y: 12, duration: 240, easing: quintOut }}>
					<div class="family-title">Modo original</div>
					<div class="cards">
						{#each SERVER_SOFTWARE.filter((option) => option.family === 'vanilla') as option (option.kind)}
							{@render softwareCard(option)}
						{/each}
					</div>
					<div class="family-title">Mod loaders · servidores de mods</div>
					<div class="cards">
						{#each SERVER_SOFTWARE.filter((option) => option.family === 'mods') as option (option.kind)}
							{@render softwareCard(option)}
						{/each}
					</div>
					<div class="family-title">Servidores de plugins</div>
					<div class="cards">
						{#each SERVER_SOFTWARE.filter((option) => option.family === 'plugins') as option (option.kind)}
							{@render softwareCard(option)}
						{/each}
					</div>
					{#if selectedSoftware.requiresVersion && kind !== 'vanilla'}
						<label class="field mt-2">
							<span>Versão do {selectedSoftware.name} para o Minecraft {mcVersion}</span>
							{#if loadingLoaders}
								<div class="loader-loading"><LoaderCircle class="h-4 w-4 animate-spin" /> Consultando versões oficiais…</div>
							{:else if loaderVersions.length}
								<GlassSelect
									accent="rgb(var(--amethyst-400))"
									ariaLabel={`Versão do ${selectedSoftware.name}`}
									options={loaderVersions.map((item) => ({ value: item.id, label: item.stable ? item.id : `${item.id} · beta` }))}
									bind:value={loaderVersion}
								/>
							{:else}
								<p class="hint error-hint">Nenhuma versão compatível encontrada para esta combinação. Volte e escolha outro software ou versão do Minecraft.</p>
							{/if}
						</label>
					{/if}
				</div>
			{:else if step === 2}
				<div class="step-body" in:fly={{ y: 12, duration: 240, easing: quintOut }}>
					<div class="java-required" class:ok={compatibleRuntimes.length > 0}>
						<Coffee class="h-5 w-5" />
						<p>
							O Minecraft {mcVersion} requer <strong>Java {javaMajor}</strong> para o servidor.
						</p>
					</div>
					{#if javaScanning}
						<div class="java-loading"><LoaderCircle class="h-5 w-5 animate-spin" /> Procurando Javas instalados…</div>
					{:else if compatibleRuntimes.length}
						<ul class="java-list">
							{#each compatibleRuntimes as runtime (runtime.path ?? runtime.major)}
								<li class="java-item ok">
									<Cpu class="h-4 w-4 text-success" />
									<div class="min-w-0">
										<p class="java-name">Java {runtime.major}{runtime.isSystem ? ' · do sistema' : ' · gerenciado pelo Luxmc'}</p>
										<p class="java-path">{runtime.versionString ?? 'Versão não informada'}{runtime.path ? ` · ${runtime.path}` : ''}</p>
									</div>
									<span class="java-badge">Compatível</span>
								</li>
							{/each}
						</ul>
						<p class="hint">O Luxmc usará automaticamente um runtime compatível ao iniciar este servidor.</p>
					{:else}
						<ul class="java-list">
							{#each runtimes.filter((runtime) => runtime.installed) as runtime (runtime.path ?? runtime.major)}
								<li class="java-item old">
									<Cpu class="h-4 w-4 text-warning" />
									<div class="min-w-0">
										<p class="java-name">Java {runtime.major}{runtime.isSystem ? ' · do sistema' : ' · gerenciado pelo Luxmc'}</p>
										<p class="java-path">{runtime.versionString ?? 'Versão não informada'}{runtime.path ? ` · ${runtime.path}` : ''}</p>
									</div>
									<span class="java-badge warn">Incompatível com o Minecraft {mcVersion}</span>
								</li>
							{/each}
						</ul>
						<div class="java-install">
							<p class="text-sm leading-relaxed text-fg-muted">
								Nenhum Java {javaMajor} compatível foi encontrado. O Luxmc pode instalar um runtime oficial gerenciado, usado apenas pelos seus servidores e instâncias.
							</p>
							<button type="button" class="chip" disabled={javaInstalling} onclick={() => { javaInstalling = true; javaInstall(javaMajor).then((status) => { if (status.installed) { runtimes = [...runtimes.filter((r) => r.major !== status.major), status]; } }).catch((cause) => (error = serverMessage(cause))).finally(() => (javaInstalling = false)); }}>
								{#if javaInstalling}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}
								Instalar Java {javaMajor}
							</button>
						</div>
						<p class="hint">A instalação também pode continuar: o Luxmc prepara o runtime automaticamente durante a primeira inicialização.</p>
					{/if}
				</div>
			{:else if step === 3}
				<div class="step-body" in:fly={{ y: 12, duration: 240, easing: quintOut }}>
					<div class="grid gap-5 lg:grid-cols-2">
						<section class="panel">
							<h3 class="panel-title">Mundo</h3>
							<label class="field">
								<span>Semente do mundo <em class="hint-inline">(opcional)</em></span>
								<div class="seed-row">
									<input type="text" maxlength="64" bind:value={seed} placeholder="Deixe vazio para o Minecraft escolher" />
									<button type="button" class="chip" onclick={randomSeed} aria-label="Gerar semente aleatória">
										<Dice5 class="h-4 w-4" /> Aleatória
									</button>
								</div>
							</label>
							<div class="grid grid-cols-2 gap-3">
								<label class="field">
									<span>Modo de jogo</span>
									<GlassSelect accent="rgb(var(--amethyst-400))" options={GAMEMODE_OPTIONS} bind:value={gamemode} ariaLabel="Modo de jogo" />
								</label>
								<label class="field">
									<span>Dificuldade</span>
									<GlassSelect accent="rgb(var(--amethyst-400))" options={DIFFICULTY_OPTIONS} bind:value={difficulty} ariaLabel="Dificuldade" />
								</label>
							</div>
							<div class="toggles">
								<label class="toggle-row">
									<input type="checkbox" bind:checked={hardcore} />
									<span><strong>Hardcore</strong><small>Dificuldade travada no nível mais difícil; quem morre é banido do mundo.</small></span>
								</label>
								<label class="toggle-row">
									<input type="checkbox" bind:checked={structures} />
									<span><strong>Gerar estruturas</strong><small>Aldeias, fortalezas, templos e minas surgem no mundo.</small></span>
								</label>
							</div>
							<p class="hint">O tipo de terreno usa o padrão normal do Minecraft. Presets como Superflat e Amplified podem ser trocados depois regenerando o mundo.</p>
						</section>
						<section class="panel">
							<h3 class="panel-title">Jogadores e rede</h3>
							<div class="grid grid-cols-2 gap-3">
								<label class="field">
									<span>Máximo de jogadores</span>
									<input type="number" min="1" max="999" bind:value={maxPlayers} />
								</label>
								<label class="field">
									<span>Proteção do spawn (blocos)</span>
									<input type="number" min="0" max="64" bind:value={spawnProtection} />
								</label>
								<label class="field">
									<span>Distância de visão</span>
									<input type="number" min="3" max="32" bind:value={viewDistance} />
								</label>
								<label class="field">
									<span>Distância de simulação</span>
									<input type="number" min="3" max="32" bind:value={simulationDistance} />
								</label>
							</div>
							<div class="toggles">
								<label class="toggle-row">
									<input type="checkbox" bind:checked={pvp} />
									<span><strong>PvP</strong><small>Jogadores podem se atacar entre si.</small></span>
								</label>
								<label class="toggle-row">
									<input type="checkbox" bind:checked={whitelist} />
									<span><strong>Lista de permissões</strong><small>Somente jogadores autorizados entram no servidor.</small></span>
								</label>
								<label class="toggle-row">
									<input type="checkbox" bind:checked={commandBlocks} />
									<span><strong>Blocos de comando</strong><small>Ativa o uso de blocos de comando no mundo.</small></span>
								</label>
								<label class="toggle-row">
									<input type="checkbox" bind:checked={allowFlight} />
									<span><strong>Voo livre</strong><small>Tolera voo no modo sobrevivência sem kick antitrapaça.</small></span>
								</label>
								<label class="toggle-row">
									<input type="checkbox" bind:checked={onlineMode} />
									<span><strong>Exigir contas originais</strong><small>Desmarque para aceitar também contas pirata/offline (modo offline).</small></span>
								</label>
							</div>
						</section>
					</div>

					<section class="panel">
						<h3 class="panel-title">Desempenho</h3>
						<div class="profile-row" role="radiogroup" aria-label="Perfil de recursos">
							<button type="button" class="chip" onclick={() => applyProfile('economico')}>Econômico</button>
							<button type="button" class="chip" onclick={() => applyProfile('equilibrado')}>Equilibrado</button>
							<button type="button" class="chip" onclick={() => applyProfile('desempenho')}>Desempenho</button>
						</div>
						<p class="hint">Econômico usa menos memória e distâncias curtas. Desempenho aumenta visão e simulação para mundos maiores. Os valores podem ser ajustados abaixo.</p>
						<div class="ram-block">
							<div class="ram-head">
								<span class="field-label">Memória do servidor</span>
								<span class="ram-value font-mono">{formatGiB(ramMb)}</span>
							</div>
							<input type="range" min="512" max={totalRamMb ? Math.min(16384, Math.floor(totalRamMb / 512) * 512) : 8192} step="512" bind:value={ramMb} aria-label="Memória alocada ao servidor" />
							<div class="ram-scale font-mono"><span>512 MiB</span><span>{totalRamMb ? formatGiB(totalRamMb) : '8 GiB'} do computador</span></div>
							{#if ramTooHigh}
								<p class="warn-line"><AlertTriangle class="h-3.5 w-3.5 text-warning" /> Essa alocação usa mais de 75% da memória do computador e pode travar o sistema.</p>
							{/if}
							{#if ramTooLow}
								<p class="warn-line"><AlertTriangle class="h-3.5 w-3.5 text-warning" /> Servidores com mods precisam de pelo menos 3 GiB para carregar com estabilidade.</p>
							{/if}
							<p class="hint">Recomendado para este computador e este software: {formatGiB(recommendedRam)}.</p>
						</div>
					</section>
				</div>
			{:else}
				<div class="step-body" in:fly={{ y: 12, duration: 240, easing: quintOut }}>
					<section class="panel">
						<h3 class="panel-title">Regras do jogo</h3>
						<p class="hint mb-3">
							O catálogo completo abaixo reflete o Minecraft {mcVersion}. As regras marcadas como “aplicadas na criação” entram na configuração do servidor; as demais podem ser ajustadas a qualquer momento na seção Mundo, com o servidor online.
						</p>
						<div class="rule-applied">
							{#each Object.entries(gameruleValues).filter(([id]) => catalog.some((rule) => rule.id === id)) as [id, value] (id)}
								<label class="toggle-row compact">
									<input type="checkbox" checked={gameruleValues[id] ?? false} onchange={(event) => setGamerule(id, event.currentTarget.checked)} />
									<span>
										<strong>{catalog.find((rule) => rule.id === id)?.label ?? id}</strong>
										<small class="font-mono">{catalog.find((rule) => rule.id === id)?.serverName ?? id} · {value ? 'ativado' : 'desativado'}</small>
									</span>
								</label>
							{/each}
						</div>
						<div class="catalog-tools">
							<input type="search" class="search" placeholder="Buscar regra…" bind:value={gameruleSearch} aria-label="Buscar regra do jogo" />
							<div class="catalog-filters">
								<button type="button" class={gameruleCategory === 'all' ? 'chip active' : 'chip'} onclick={() => (gameruleCategory = 'all')}>
									Todas ({catalog.length})
								</button>
								{#each GAMERULE_CATEGORIES as category (category.id)}
									{#if categoryLabel.get(category.id)}
										<button type="button" class={gameruleCategory === category.id ? 'chip active' : 'chip'} onclick={() => (gameruleCategory = category.id)}>
											{category.label} ({categoryLabel.get(category.id)})
										</button>
									{/if}
								{/each}
							</div>
						</div>
						<div class="catalog custom-scrollbar">
							{#each filteredRules as rule (rule.id)}
								<div class="rule">
									<div class="min-w-0 flex-1">
										<p class="rule-label">{rule.label}</p>
										<p class="rule-id font-mono">{rule.serverName}</p>
										<p class="rule-desc">{rule.description}</p>
									</div>
									<span class="rule-default font-mono">{rule.type === 'bool' ? (gameruleValues[rule.id] !== undefined ? (gameruleValues[rule.id] ? 'true' : 'false') : String(rule.defaultValue)) : String(rule.defaultValue)}</span>
								</div>
							{:else}
								<p class="hint">Nenhuma regra corresponde à busca.</p>
							{/each}
						</div>
					</section>

					<section class="panel">
						<h3 class="panel-title">Resumo</h3>
						<dl class="summary">
							<div><dt>Nome</dt><dd>{name.trim()}</dd></div>
							<div><dt>Versão</dt><dd class="font-mono">Minecraft {mcVersion}</dd></div>
							<div><dt>Software</dt><dd>{selectedSoftware.name}{loaderVersion ? ` ${loaderVersion}` : ''}</dd></div>
							<div><dt>Java</dt><dd>Java {javaMajor}{compatibleRuntimes.length ? ' · disponível' : ' · preparado na inicialização'}</dd></div>
							<div><dt>Mundo</dt><dd>{labelGamemode(gamemode)} · {labelDifficulty(difficulty)}{hardcore ? ' · hardcore' : ''}</dd></div>
							<div><dt>Memória</dt><dd class="font-mono">{formatGiB(ramMb)}</dd></div>
							<div><dt>Jogadores</dt><dd>até {maxPlayers}</dd></div>
						</dl>
					</section>

					<section class="panel eula-panel">
						<label class="eula-row" class:eula-off={!eula}>
							<input type="checkbox" bind:checked={eula} />
							<span class="eula-box" aria-hidden="true"><Check class="h-3.5 w-3.5" /></span>
							<span class="eula-text">
								Li e concordo com o contrato de licença de usuário final do Minecraft (EULA).
								<a href="https://www.minecraft.net/en-us/eula" target="_blank" rel="noreferrer" class="eula-link">Ler a EULA oficial <ExternalLink class="h-3 w-3" /></a>
							</span>
						</label>
						<p class="hint"><ShieldCheck class="h-3.5 w-3.5 inline text-brass-400" /> A caixa começa desmarcada: o servidor só recebe eula=true depois da sua aceitação explícita.</p>
					</section>
				</div>
			{/if}
		</div>

		<footer class="footer">
			{#if !installing}
				<button type="button" class="ghost-btn" disabled={step === 0} onclick={back}>
					<ArrowLeft class="h-4 w-4" /> Voltar
				</button>
				{#if step < TOTAL_STEPS - 1}
					<button type="button" class="primary-btn" disabled={!canAdvance} onclick={next}>
						Continuar <ArrowRight class="h-4 w-4" />
					</button>
				{:else}
					<button type="button" class="primary-btn" disabled={!eula || busy} onclick={() => void create()}>
						{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}
						Criar servidor
					</button>
				{/if}
			{:else}
				<span class="installing-note"><RefreshCw class="h-4 w-4 animate-spin text-amethyst-400" /> Executando a instalação real. Não feche esta janela.</span>
			{/if}
		</footer>
	</div>
</dialog>

{#snippet softwareCard(option: (typeof SERVER_SOFTWARE)[number])}
	{@const Icon = softIcons[option.kind]}
	{@const selected = kind === option.kind}
	<button
		type="button"
		class={selected ? 'card selected' : option.supported ? 'card' : 'card disabled'}
		onclick={() => pickSoftware(option.kind)}
		disabled={!option.supported}
		title={option.supported ? undefined : option.unavailableReason}
		aria-pressed={selected}
	>
		<span class="card-icon"><Icon class="h-5 w-5" /></span>
		<span class="card-body">
			<span class="card-name">{option.name}</span>
			<span class="card-tag">{option.tagline}</span>
			<span class="card-desc">{option.supported ? option.description : option.unavailableReason}</span>
		</span>
	</button>
{/snippet}

<style>
	.wizard-dialog {
		margin: auto;
		width: min(860px, calc(100vw - 32px));
		height: min(760px, calc(100dvh - 80px));
		padding: 0;
		border: none;
		background: transparent;
		color: rgb(var(--fg));
	}
	.wizard-dialog:not([open]) { display: none; }
	.wizard-dialog::backdrop { background: rgb(var(--bg-overlay) / 0.85); }
	.frame {
		display: flex; flex-direction: column; height: 100%; overflow: hidden;
		border: 1px solid rgb(var(--border-strong)); border-radius: 18px;
		background: linear-gradient(170deg, rgb(var(--bg-elevated)), rgb(var(--bg-subtle)));
		box-shadow: 0 24px 64px rgb(0 0 0 / 0.5);
	}
	.header { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 20px 24px 14px; }
	.brand { display: flex; align-items: center; gap: 14px; min-width: 0; }
	.emblem { width: 44px; height: 44px; object-fit: contain; }
	.eyebrow { font-size: 10px; font-weight: 700; letter-spacing: 0.22em; text-transform: uppercase; color: rgb(var(--brass-400)); }
	.title { margin-top: 2px; font-family: Georgia, 'Times New Roman', serif; font-size: 22px; font-weight: 700; letter-spacing: 0.01em; }
	.icon-btn { display: grid; place-items: center; width: 36px; height: 36px; border: 1px solid rgb(var(--border-strong)); border-radius: 14px; color: rgb(var(--fg-muted)); background: rgb(var(--bg-subtle)); }
	.icon-btn:hover { color: rgb(var(--fg)); border-color: rgb(var(--brass-500) / 0.6); }
	.progress { display: flex; align-items: center; gap: 8px; padding: 2px 24px 14px; overflow-x: auto; }
	.progress-step { display: flex; align-items: center; gap: 8px; flex: none; }
	.dot { display: grid; place-items: center; width: 22px; height: 22px; flex: none; border-radius: 999px; border: 1px solid rgb(var(--border-strong)); font-size: 10.5px; font-weight: 700; color: rgb(var(--fg-muted)); background: rgb(var(--bg-overlay)); }
	.current .dot { border-color: rgb(var(--amethyst-400)); color: rgb(var(--amethyst-300)); box-shadow: 0 0 0 3px rgb(var(--amethyst-500) / 0.18); }
	.past .dot { border-color: rgb(var(--success) / 0.5); color: rgb(var(--success)); }
	.progress-label { font-size: 11.5px; font-weight: 600; color: rgb(var(--fg-subtle)); white-space: nowrap; }
	.current .progress-label { color: rgb(var(--fg)); }
	.bar { height: 1px; flex: 1 1 24px; min-width: 16px; background: rgb(var(--border-strong)); }
	.bar.filled { background: rgb(var(--success) / 0.55); }
	.content { flex: 1; min-height: 0; overflow-y: auto; padding: 4px 24px 20px; }
	.step-body { display: flex; flex-direction: column; gap: 14px; }
	.alert { display: flex; align-items: flex-start; gap: 10px; margin-bottom: 12px; padding: 12px 14px; border: 1px solid rgb(var(--danger) / 0.4); border-radius: 16px; background: rgb(var(--danger) / 0.08); font-size: 13px; line-height: 1.6; }
	.alert-install { margin-top: 12px; margin-bottom: 0; }
	.field { display: flex; flex-direction: column; gap: 7px; }
	.field > span, .field-label { font-size: 11px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: rgb(var(--fg-muted)); }
	.hint-inline { font-style: normal; font-weight: 500; letter-spacing: 0; text-transform: none; color: rgb(var(--fg-subtle)); }
	.field input {
		min-height: 42px; padding: 9px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 14px;
		background: rgb(var(--bg-overlay) / 0.7); color: rgb(var(--fg)); font-size: 14px;
	}
	.field input:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 1px; }
	.hint { font-size: 12px; line-height: 1.6; color: rgb(var(--fg-muted)); }
	.error-hint { color: rgb(var(--danger)); }
	.edition-row { display: grid; gap: 10px; grid-template-columns: 1fr; }
	.edition { display: flex; gap: 12px; padding: 14px 16px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: rgb(var(--bg-overlay) / 0.5); }
	.edition.selected { border-color: rgb(var(--amethyst-400) / 0.6); background: rgb(var(--amethyst-500) / 0.08); }
	.edition.disabled { opacity: 0.55; }
	.edition-name { font-size: 14px; font-weight: 600; }
	.edition-note { margin-top: 2px; font-size: 12px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.release-toggle { display: flex; gap: 8px; }
	.chip { display: inline-flex; align-items: center; justify-content: center; gap: 8px; padding: 9px 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 999px; background: rgb(var(--bg-overlay) / 0.6); color: rgb(var(--fg-muted)); font-size: 12.5px; font-weight: 600; transition: color 140ms ease, border-color 140ms ease, background-color 140ms ease; }
	.chip:hover:not(:disabled) { color: rgb(var(--fg)); border-color: rgb(var(--brass-500) / 0.55); }
	.chip.active { border-color: rgb(var(--amethyst-400)); background: rgb(var(--amethyst-500) / 0.14); color: rgb(var(--fg)); }
	.chip:disabled { opacity: 0.5; cursor: not-allowed; }
	.family-title { margin-top: 4px; font-size: 11px; font-weight: 700; letter-spacing: 0.14em; text-transform: uppercase; color: rgb(var(--brass-400)); }
	.cards { display: grid; gap: 10px; grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); }
	.card { display: flex; gap: 12px; padding: 14px; text-align: left; border: 1px solid rgb(var(--border-strong)); border-radius: 18px; background: rgb(var(--bg-overlay) / 0.5); transition: border-color 150ms ease, background-color 150ms ease; }
	.card:hover:not(:disabled) { border-color: rgb(var(--brass-500) / 0.5); }
	.card.selected { border-color: rgb(var(--amethyst-400)); background: rgb(var(--amethyst-500) / 0.1); box-shadow: 0 0 0 1px rgb(var(--amethyst-400) / 0.4); }
	.card.disabled { opacity: 0.5; cursor: not-allowed; }
	.card-icon { display: grid; place-items: center; width: 38px; height: 38px; flex: none; border: 1px solid rgb(var(--border-strong)); border-radius: 14px; color: rgb(var(--brass-300)); background: rgb(var(--bg-subtle)); }
	.selected .card-icon { color: rgb(var(--amethyst-300)); border-color: rgb(var(--amethyst-400) / 0.5); }
	.card-body { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
	.card-name { font-size: 14px; font-weight: 700; }
	.card-tag { font-size: 11.5px; font-weight: 600; color: rgb(var(--brass-300)); }
	.card-desc { font-size: 11.5px; line-height: 1.55; color: rgb(var(--fg-muted)); }
	.loader-loading { display: flex; align-items: center; gap: 10px; min-height: 42px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.java-required { display: flex; align-items: center; gap: 12px; padding: 12px 14px; border: 1px solid rgb(var(--warning) / 0.35); border-radius: 16px; background: rgb(var(--warning) / 0.07); font-size: 13.5px; color: rgb(var(--fg)); }
	.java-required.ok { border-color: rgb(var(--success) / 0.35); background: rgb(var(--success) / 0.06); }
	.java-loading { display: flex; align-items: center; gap: 10px; font-size: 13px; color: rgb(var(--fg-muted)); }
	.java-list { display: flex; flex-direction: column; gap: 8px; }
	.java-item { display: flex; align-items: center; gap: 12px; padding: 12px 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: rgb(var(--bg-overlay) / 0.5); }
	.java-name { font-size: 13.5px; font-weight: 600; }
	.java-path { font-family: 'JetBrains Mono', ui-monospace, monospace; font-size: 10.5px; line-height: 1.5; color: rgb(var(--fg-subtle)); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.java-badge { margin-left: auto; flex: none; padding: 4px 10px; border-radius: 999px; font-size: 11px; font-weight: 700; background: rgb(var(--success) / 0.14); color: rgb(var(--success)); }
	.java-badge.warn { background: rgb(var(--warning) / 0.14); color: rgb(var(--warning)); }
	.java-install { display: flex; flex-wrap: wrap; align-items: center; gap: 14px; padding: 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: rgb(var(--bg-overlay) / 0.5); }
	.panel { padding: 16px; border: 1px solid rgb(var(--border-strong)); border-radius: 18px; background: rgb(var(--bg-overlay) / 0.45); }
	.panel-title { margin-bottom: 12px; font-size: 12px; font-weight: 700; letter-spacing: 0.14em; text-transform: uppercase; color: rgb(var(--brass-300)); }
	.seed-row { display: flex; gap: 8px; }
	.seed-row input { flex: 1; min-width: 0; }
	.toggles { display: flex; flex-direction: column; gap: 8px; margin-top: 10px; }
	.toggle-row { display: flex; align-items: flex-start; gap: 10px; padding: 9px 10px; border: 1px solid rgb(var(--border-strong)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.4); cursor: pointer; }
	.toggle-row input { margin-top: 3px; accent-color: rgb(var(--amethyst-400)); }
	.toggle-row input:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 2px; }
	.toggle-row span { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
	.toggle-row strong { font-size: 13px; font-weight: 600; }
	.toggle-row small { font-size: 11.5px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.toggle-row.compact { padding: 7px 10px; }
	.profile-row { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 8px; }
	.ram-block { margin-top: 12px; padding: 14px; border: 1px solid rgb(var(--border-strong)); border-radius: 16px; background: rgb(var(--bg-overlay) / 0.5); }
	.ram-head { display: flex; align-items: baseline; justify-content: space-between; gap: 12px; margin-bottom: 10px; }
	.ram-value { font-size: 20px; font-weight: 700; color: rgb(var(--copper-300, var(--copper-400))); }
	.ram-scale { display: flex; justify-content: space-between; margin-top: 6px; font-size: 10.5px; color: rgb(var(--fg-subtle)); }
	.warn-line { display: flex; align-items: center; gap: 8px; margin-top: 8px; font-size: 12.5px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	input[type='range'] { width: 100%; accent-color: rgb(var(--amethyst-400)); }
	input[type='range']:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 4px; }
	.rule-applied { display: grid; gap: 8px; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); margin-bottom: 12px; }
	.catalog-tools { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; margin-bottom: 10px; }
	.search { min-height: 38px; flex: 1 1 220px; padding: 8px 12px; border: 1px solid rgb(var(--border-strong)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.7); color: rgb(var(--fg)); font-size: 13px; }
	.search:focus-visible { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 1px; }
	.catalog-filters { display: flex; flex-wrap: wrap; gap: 6px; }
	.catalog { display: flex; flex-direction: column; gap: 6px; max-height: 300px; overflow-y: auto; padding-right: 4px; }
	.rule { display: flex; align-items: flex-start; gap: 12px; padding: 10px 12px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.35); }
	.rule-label { font-size: 13px; font-weight: 600; }
	.rule-id { font-size: 10.5px; color: rgb(var(--brass-300)); }
	.rule-desc { margin-top: 2px; font-size: 11.5px; line-height: 1.5; color: rgb(var(--fg-muted)); }
	.rule-default { flex: none; margin-top: 2px; font-size: 11px; color: rgb(var(--fg-subtle)); }
	.summary { display: grid; gap: 8px; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); }
	.summary > div { display: flex; flex-direction: column; gap: 2px; padding: 9px 12px; border: 1px solid rgb(var(--border)); border-radius: 14px; background: rgb(var(--bg-overlay) / 0.35); }
	.summary dt { font-size: 10.5px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: rgb(var(--fg-subtle)); }
	.summary dd { font-size: 13px; color: rgb(var(--fg)); }
	.eula-row { display: flex; align-items: flex-start; gap: 12px; cursor: pointer; }
	.eula-row input { position: absolute; width: 1px; height: 1px; opacity: 0; }
	.eula-box { display: grid; place-items: center; width: 20px; height: 20px; flex: none; margin-top: 2px; border: 1.5px solid rgb(var(--border-strong)); border-radius: 6px; color: transparent; background: rgb(var(--bg-overlay)); transition: border-color 140ms ease, background-color 140ms ease; }
	.eula-box :global(svg) { opacity: 0; }
	.eula-row input:checked + .eula-box { border-color: rgb(var(--amethyst-400)); background: rgb(var(--amethyst-500)); color: white; }
	.eula-row input:checked + .eula-box :global(svg) { opacity: 1; }
	.eula-row input:focus-visible + .eula-box { outline: 2px solid rgb(var(--amethyst-400)); outline-offset: 2px; }
	.eula-off .eula-text { color: rgb(var(--fg-muted)); }
	.eula-text { font-size: 13px; line-height: 1.6; }
	.eula-link { display: inline-flex; align-items: center; gap: 4px; margin-left: 6px; font-weight: 600; color: rgb(var(--brand-300)); text-decoration: underline; text-underline-offset: 3px; }
	.footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 14px 24px; border-top: 1px solid rgb(var(--border-strong)); background: rgb(var(--bg-overlay) / 0.6); }
	.ghost-btn { display: inline-flex; align-items: center; gap: 8px; padding: 10px 16px; border: 1px solid rgb(var(--border-strong)); border-radius: 14px; background: transparent; color: rgb(var(--fg-muted)); font-size: 13px; font-weight: 600; }
	.ghost-btn:hover:not(:disabled) { color: rgb(var(--fg)); border-color: rgb(var(--brass-500) / 0.5); }
	.ghost-btn:disabled { opacity: 0.4; cursor: not-allowed; }
	.primary-btn { display: inline-flex; align-items: center; gap: 8px; padding: 11px 22px; border: 1px solid rgb(var(--amethyst-300) / 0.6); border-radius: 14px; background: rgb(var(--amethyst-500) / 0.85); color: white; font-size: 13px; font-weight: 700; letter-spacing: 0.04em; transition: background-color 150ms ease; }
	.primary-btn:hover:not(:disabled) { background: rgb(var(--amethyst-400) / 0.95); }
	.primary-btn:disabled { opacity: 0.45; cursor: not-allowed; }
	.installing { display: flex; flex-direction: column; gap: 4px; }
	.installing-note { display: flex; align-items: center; gap: 10px; font-size: 13px; color: rgb(var(--fg-muted)); }
	@media (prefers-reduced-motion: reduce) {
		.wizard-dialog, .step-body { animation: none; }
	}
	@media (min-width: 1024px) {
		.edition-row { grid-template-columns: 1fr 1fr; }
	}
</style>
