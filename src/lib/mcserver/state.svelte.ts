import {
	localServerList,
	serverConnectionDiagnostic,
	serverManage,
	serverPublicAccess,
	serverSettings,
	serverTelemetry,
	type LocalServerConfig,
	type LocalServerOverview,
	type LocalServerState,
	type ServerConnectionDiagnostic,
	type ServerPublicAccess,
	type ServerLoader,
	type ServerSettings,
	type ServerTelemetry
} from '$lib/api/localServers';
import { getSystemSpecs, networkThroughputSample, systemResourceSample } from '$lib/api/system';
import { supportsMods } from './software';

const HISTORY = 60;

function push(list: number[], value: number): number[] {
	return [...list, Number.isFinite(value) ? value : 0].slice(-HISTORY);
}

export type PlatformSection =
	| 'inicio'
	| 'servidores'
	| 'console'
	| 'jogadores'
	| 'mundo'
	| 'mods'
	| 'plugins'
	| 'backups'
	| 'desempenho'
	| 'configuracoes'
	| 'sobre';

class ServerPlatform {
	overview = $state<LocalServerOverview | null>(null);
	selectedId = $state('');
	settings = $state<ServerSettings | null>(null);
	loader = $state<ServerLoader>({ kind: 'vanilla', version: '' });
	busy = $state(false);

	totalRamMb = $state(0);
	usedRamMb = $state(0);
	cpuPercent = $state(0);
	netDownBps = $state(0);
	netUpBps = $state(0);
	cpuHistory = $state<number[]>([]);
	memHistory = $state<number[]>([]);
	netHistory = $state<number[]>([]);

	telemetry = $state<ServerTelemetry | null>(null);
	telemetryError = $state<string | null>(null);
	heapHistory = $state<number[]>([]);
	procMemHistory = $state<number[]>([]);
	procCpuHistory = $state<number[]>([]);
	diagnostic = $state<ServerConnectionDiagnostic | null>(null);
	publicAccess = $state<ServerPublicAccess | null>(null);

	runningSince = $state<number | null>(null);

	#timers: ReturnType<typeof setInterval>[] = [];
	#specsLoaded = false;
	#telemetryBusy = false;

	get servers(): LocalServerConfig[] {
		return this.overview?.servers ?? [];
	}

	get active(): LocalServerState | null {
		return this.overview?.active ?? null;
	}

	get selectedState(): LocalServerState | null {
		if (!this.selectedId) return null;
		const states = this.overview?.states ?? (this.active ? [this.active] : []);
		return states.find((state) => state.id === this.selectedId) ?? null;
	}

	get config(): LocalServerConfig | null {
		return this.servers.find((server) => server.id === this.selectedId) ?? null;
	}

	get phase(): string {
		return this.selectedState?.phase ?? 'stopped';
	}

	get running(): boolean {
		return this.phase === 'running';
	}

	get transition(): boolean {
		return ['preparing', 'downloading', 'java', 'starting', 'stopping'].includes(this.phase);
	}

	get activeForSelected(): boolean {
		return this.selectedState !== null;
	}

	get playersOnline(): number {
		return this.selectedState?.playersOnline ?? 0;
	}

	get port(): number | null {
		return this.selectedState?.port ?? null;
	}

	get modLoader(): boolean {
		return supportsMods(this.loader.kind);
	}

	get allocatedRamMb(): number {
		return this.config?.ramMb ?? 2048;
	}

	get warnings(): string[] {
		return this.overview?.warnings ?? [];
	}

	select(id: string) {
		if (id === this.selectedId) return;
		this.selectedId = id;
		this.runningSince = null;
		this.telemetry = null;
		this.telemetryError = null;
		this.heapHistory = [];
		this.procMemHistory = [];
		this.procCpuHistory = [];
		this.diagnostic = null;
		this.publicAccess = null;
	}

	async refreshServers(): Promise<void> {
		try {
			const next = await localServerList();
			this.overview = next;
			if (!this.selectedId) this.selectedId = next.active?.id ?? next.servers[0]?.id ?? '';
			else if (!next.servers.some((server) => server.id === this.selectedId)) this.selectedId = next.servers[0]?.id ?? '';
			const states = next.states ?? (next.active ? [next.active] : []);
			const selected = states.find((state) => state.id === this.selectedId) ?? null;
			const nowRunning = selected?.phase === 'running';
			if (nowRunning && this.runningSince === null) this.runningSince = Date.now();
			if (!nowRunning && this.runningSince !== null) this.runningSince = null;
		} catch {
			// A falha temporária de leitura não deve derrubar a interface; a próxima varredura corrige.
		}
	}

	async refreshDiagnostic(): Promise<void> {
		const id = this.selectedId;
		if (!id) return;
		try {
			const result = await serverConnectionDiagnostic(id);
			if (this.selectedId === id) this.diagnostic = result;
		} catch {
			if (this.selectedId === id) this.diagnostic = null;
		}
	}

	async refreshPublicAccess(): Promise<void> {
		const id = this.selectedId;
		if (!id) return;
		try {
			const status = await serverPublicAccess(id, 'status');
			if (this.selectedId === id) this.publicAccess = status;
		} catch {
			if (this.selectedId === id) this.publicAccess = null;
		}
	}

	async refreshServerTelemetry(): Promise<void> {
		const id = this.selectedId;
		if (!id || this.#telemetryBusy) return;
		this.#telemetryBusy = true;
		try {
			const sample = await serverTelemetry(id);
			if (this.selectedId !== id) return;
			this.telemetry = sample;
			this.telemetryError = null;
			if (sample.online) {
				this.heapHistory = push(this.heapHistory, sample.heapUsedMb ?? 0);
				this.procMemHistory = push(this.procMemHistory, sample.processMemoryMb ?? 0);
				this.procCpuHistory = push(this.procCpuHistory, sample.processCpuPercent ?? 0);
			} else {
				this.heapHistory = [];
				this.procMemHistory = [];
				this.procCpuHistory = [];
			}
		} catch (cause) {
			if (this.selectedId !== id) return;
			this.telemetry = null;
			this.telemetryError = cause instanceof Error ? cause.message : 'Telemetria indisponível.';
		} finally {
			this.#telemetryBusy = false;
		}
	}

	async refreshTelemetry(): Promise<void> {
		try {
			const sample = await systemResourceSample();
			this.cpuPercent = sample.cpuPercent;
			this.usedRamMb = sample.usedMemoryMb;
			if (sample.totalMemoryMb > 0) this.totalRamMb = sample.totalMemoryMb;
			const total = this.totalRamMb || sample.totalMemoryMb || 1;
			this.cpuHistory = push(this.cpuHistory, sample.cpuPercent);
			this.memHistory = push(this.memHistory, (sample.usedMemoryMb / total) * 100);
		} catch {
			// Telemetria indisponível: os gráficos apenas pausam.
		}
	}

	async refreshNetwork(): Promise<void> {
		try {
			const sample = await networkThroughputSample();
			this.netDownBps = sample.receivedBytesPerSecond;
			this.netUpBps = sample.transmittedBytesPerSecond;
			this.netHistory = push(this.netHistory, sample.receivedBytesPerSecond + sample.transmittedBytesPerSecond);
		} catch {
			// Telemetria indisponível: os gráficos apenas pausam.
		}
	}

	async loadSystemSpecs(): Promise<void> {
		if (this.#specsLoaded || this.totalRamMb > 0) return;
		try {
			const specs = await getSystemSpecs();
			if (specs.totalRamMb > 0) {
				this.totalRamMb = specs.totalRamMb;
				this.#specsLoaded = true;
			}
		} catch {
			// Sem especificações, o controle de RAM usa limites conservadores.
		}
	}

	async loadServerData(id: string): Promise<void> {
		if (!id) {
			this.settings = null;
			this.loader = { kind: 'vanilla', version: '' };
			return;
		}
		try {
			const [nextSettings, nextLoader] = await Promise.all([
				serverSettings(id).catch(() => null),
				serverManage<ServerLoader>({ action: 'loader', id }).catch(() => null)
			]);
			if (nextSettings) {
				this.settings = {
					networkCompressionThreshold: 256,
					entityBroadcastRangePercentage: 100,
					maxTickTime: 60000,
					opPermissionLevel: 4,
					...nextSettings
				} satisfies ServerSettings;
			}
			if (nextLoader) this.loader = nextLoader;
		} catch {
			// Os dados chegam na próxima varredura.
		}
	}

	startPolling(): void {
		if (this.#timers.length > 0) return;
		void this.refreshServers();
		void this.refreshServerTelemetry();
		void this.refreshTelemetry();
		void this.refreshNetwork();
		void this.loadSystemSpecs();
		this.#timers = [
			setInterval(() => void this.refreshServers(), 1500),
			setInterval(() => void this.refreshServerTelemetry(), 2000),
			setInterval(() => void this.refreshTelemetry(), 2500),
			setInterval(() => void this.refreshNetwork(), 2000)
		];
	}

	stopPolling(): void {
		this.#timers.forEach((timer) => clearInterval(timer));
		this.#timers = [];
	}
}

export const platform = new ServerPlatform();
