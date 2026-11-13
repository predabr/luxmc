import { api } from './client';
import { roomIdentity, type TunnelStatus } from './tunnel';
import { account } from '$lib/stores/account.svelte';
import { ensureRoomAvailable } from './roomSession';

export interface LocalServerConfig { id: string; name: string; mcVersion: string; ramMb: number; maxPlayers: number; eulaAccepted: boolean }
export interface CreateLocalServer { name: string; mcVersion: string; ramMb: number; maxPlayers: number; eulaAccepted: boolean }
export interface ServerConsoleLine { sequence: number; stream: string; text: string }
export type ServerPhase = 'preparing' | 'downloading' | 'java' | 'starting' | 'running' | 'unreachable' | 'stopping' | 'stopped' | 'failed';
export interface ServerPlayer { uuid: string; name: string; firstJoinedAt: string; lastJoinedAt: string; lastLeftAt: string | null; visits: number; online: boolean }
export interface LocalServerState { id: string; generation: string; phase: ServerPhase; port: number | null; pid: number | null; maxPlayers: number; playersOnline: number; error: string | null; exitCode: number | null; console: ServerConsoleLine[]; cursor: number; lastSavedAt: string | null; forcedStop: boolean; players: ServerPlayer[]; historyComplete: boolean; lastStatus?: ServerStatusInfo | null }
export interface ServerStatusInfo { online: boolean; version: string; playersMax: number; playersOnline: number; motd: string; favicon: string | null; latencyMs: number | null }
export interface ServerBackup { id: string; createdAt: string; bytes: number; files: number }
export interface ServerContent { name: string; kind: string; bytes: number; staged: boolean; remove: boolean }
export interface ServerLoader { kind: string; version: string }
export interface LocalServerOverview { servers: LocalServerConfig[]; active: LocalServerState | null; states?: LocalServerState[]; warnings: string[]; icons?: Record<string, string> }
export interface ServerSettings { motd: string; difficulty: 'peaceful' | 'easy' | 'normal' | 'hard'; gamemode: 'survival' | 'creative' | 'adventure' | 'spectator'; forceGamemode?: boolean; pvp: boolean; whitelist: boolean; viewDistance: number; simulationDistance: number; port: number; autosaveMinutes: number; resourcePackUrl: string; resourcePackFile?: string; resourcePackSha1: string; requireResourcePack: boolean; seed?: string | null; hardcore?: boolean; spawnProtection?: number; allowFlight?: boolean; commandBlocks?: boolean; idleTimeout?: number; economyMode?: boolean; generateStructures?: boolean; networkCompressionThreshold?: number; entityBroadcastRangePercentage?: number; maxTickTime?: number; opPermissionLevel?: number; hideOnlinePlayers?: boolean; listenOnNetwork?: boolean; onlineMode?: boolean; keepInventory?: boolean; daylightCycle?: boolean; naturalRegeneration?: boolean; mobSpawning?: boolean; monsterSpawning?: boolean }
export interface ServerCatalogVersion { id: string; name: string; gameVersions: string[]; loaders: string[] }
export const serverManage = <T>(request: Record<string, unknown>): Promise<T> => api.invoke('local_server_manage', { request: JSON.parse(JSON.stringify(request)), accountId: account.value?.id });
export type DiagnosticStatus = 'approved' | 'attention' | 'failed' | 'unverified';
export interface ServerConnectionCheck { id: string; label: string; status: DiagnosticStatus; statusLabel: string; detail: string }
export interface ServerPingResult { online: boolean; version: string; playersOnline: number; playersMax: number; motd: string; latencyMs: number | null }
export interface ServerConnectionDiagnostic { minecraftResponding: boolean; address: string | null; phase: string; tcpReachable: boolean; port?: number; publicAddress?: string | null; lanAddresses?: string[]; checks?: ServerConnectionCheck[]; ping?: ServerPingResult | null; generatedAt?: string }
export const serverConnectionDiagnostic = (id: string): Promise<ServerConnectionDiagnostic> => serverManage({ action: 'connection_diagnostic', id });
export interface ServerTelemetry {
	serverId: string; phase: string; online: boolean; pid: number | null; uptimeSeconds: number | null; port: number | null;
	configuredRamMb: number; playersOnline: number; maxPlayers: number;
	processMemoryMb: number | null; processCpuPercent: number | null;
	heapUsedMb: number | null; heapCommittedMb: number | null; heapMaxMb: number | null; heapSource: string | null;
	tps: number | null; mspt: number | null;
	systemCpuPercent: number; systemUsedMemoryMb: number; systemTotalMemoryMb: number; systemAvailableMemoryMb: number;
	detail: string;
}
export const serverTelemetry = (id: string): Promise<ServerTelemetry> => api.invoke('local_server_telemetry', { id });
export const serverConsoleText = (id: string): Promise<{text: string; truncated: boolean}> => serverManage({ action: 'console_text', id });
export const serverDeletionPreview = (id: string): Promise<{files: number; bytes: number}> => serverManage({ action: 'deletion_preview', id });
export const serverSettings = (id: string): Promise<ServerSettings> => serverManage({ action: 'settings', id });
export const saveServerSettings = (id: string, settings: ServerSettings): Promise<ServerSettings> => serverManage({ action: 'save_settings', id, settings });
export const moderateServer = (id: string, command: string, name: string, reason = '', uuid?: string): Promise<void> => serverManage({ action: 'moderate', id, command, name, reason, uuid });

export const localServerList = (after = 0): Promise<LocalServerOverview> => api.invoke('local_server_list', { after });
export const createLocalServer = (input: CreateLocalServer): Promise<LocalServerConfig> => api.invoke('local_server_create', { input, accountId: account.value?.id });
export const startLocalServer = (id: string): Promise<LocalServerState> => api.invoke('local_server_start', { id, accountId: account.value?.id });
export const stopLocalServer = (id: string): Promise<void> => api.invoke('local_server_stop', { id });
export const sendServerCommand = (id: string, text: string): Promise<void> => api.invoke('local_server_console', { id, text, accountId: account.value?.id });
export const openLocalServerFolder = (id: string): Promise<void> => api.invoke('local_server_open_folder', { id });
export const updateLocalServer = (id: string, name: string, ramMb: number, maxPlayers: number, mcVersion?: string): Promise<LocalServerConfig> => api.invoke('local_server_update', { id, name, ramMb, maxPlayers, mcVersion, accountId: account.value?.id });
export async function shareLocalServer(id: string): Promise<TunnelStatus> { await ensureRoomAvailable(); return api.invoke('local_server_share', { id, identity: await roomIdentity(), accountId: account.value?.id }); }
export async function shareExistingServer(port: number): Promise<TunnelStatus> { await ensureRoomAvailable(); return api.invoke('local_server_share_existing', { port, identity: await roomIdentity(), accountId: account.value?.id }); }


export interface ServerPublicAccess {
	enabled: boolean;
	configured: boolean;
	whitelist: boolean;
	port: number;
	externalIp: string | null;
	address: string | null;
	likelyBehindCgnat: boolean;
	renewalError: string | null;
}
export type PublicAccessOperation = 'status' | 'enable' | 'disable';
export const serverPublicAccess = (id: string, operation: PublicAccessOperation): Promise<ServerPublicAccess> =>
	serverManage({ action: 'public_access', id, operation, accountId: account.value?.id });

export interface ManagedSkinSupport {name:string;required:boolean;valid:boolean;clientComponentRequired:boolean}
export const serverManagedComponents = (id:string):Promise<ManagedSkinSupport|null> => serverManage({action:'managed_components',id});
