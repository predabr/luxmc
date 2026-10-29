import { api } from './client';
import { roomIdentity, type TunnelStatus } from './tunnel';
import { account } from '$lib/stores/account.svelte';
import { ensureRoomAvailable } from './roomSession';

export interface LocalServerConfig { id: string; name: string; mcVersion: string; ramMb: number; maxPlayers: number; eulaAccepted: boolean }
export interface CreateLocalServer { name: string; mcVersion: string; ramMb: number; maxPlayers: number; eulaAccepted: boolean }
export interface ServerConsoleLine { sequence: number; stream: string; text: string }
export type ServerPhase = 'preparing' | 'downloading' | 'java' | 'starting' | 'running' | 'unreachable' | 'stopping' | 'stopped' | 'failed';
export interface ServerPlayer { uuid: string; name: string; firstJoinedAt: string; lastJoinedAt: string; lastLeftAt: string | null; visits: number; online: boolean }
export interface LocalServerState { id: string; generation: string; phase: ServerPhase; port: number | null; pid: number | null; maxPlayers: number; playersOnline: number; error: string | null; exitCode: number | null; console: ServerConsoleLine[]; cursor: number; lastSavedAt: string | null; forcedStop: boolean; players: ServerPlayer[]; historyComplete: boolean }
export interface ServerBackup { id: string; createdAt: string; bytes: number; files: number }
export interface ServerContent { name: string; kind: string; bytes: number; staged: boolean; remove: boolean }
export interface ServerLoader { kind: string; version: string }
export interface LocalServerOverview { servers: LocalServerConfig[]; active: LocalServerState | null; warnings: string[]; icons?: Record<string, string> }
export interface ServerSettings { motd: string; difficulty: 'peaceful' | 'easy' | 'normal' | 'hard'; gamemode: 'survival' | 'creative' | 'adventure' | 'spectator'; forceGamemode?: boolean; pvp: boolean; whitelist: boolean; viewDistance: number; simulationDistance: number; port: number; autosaveMinutes: number; resourcePackUrl: string; resourcePackFile?: string; resourcePackSha1: string; requireResourcePack: boolean; seed?: string | null; hardcore?: boolean; spawnProtection?: number; allowFlight?: boolean; commandBlocks?: boolean; idleTimeout?: number; economyMode?: boolean; generateStructures?: boolean; networkCompressionThreshold?: number; entityBroadcastRangePercentage?: number; maxTickTime?: number; opPermissionLevel?: number; hideOnlinePlayers?: boolean }
export interface ServerCatalogVersion { id: string; name: string; gameVersions: string[]; loaders: string[] }
export const serverManage = <T>(request: Record<string, unknown>): Promise<T> => api.invoke('local_server_manage', { request: JSON.parse(JSON.stringify(request)), accountId: account.value?.id });
export interface ServerConnectionDiagnostic { minecraftResponding: boolean; address: string | null; phase: string; tcpReachable: boolean; port?: number; publicAddress?: string | null }
export const serverConnectionDiagnostic = (id: string): Promise<ServerConnectionDiagnostic> => serverManage({ action: 'connection_diagnostic', id });
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
export const updateLocalServer = (id: string, name: string, ramMb: number, maxPlayers: number): Promise<LocalServerConfig> => api.invoke('local_server_update', { id, name, ramMb, maxPlayers, accountId: account.value?.id });
export async function shareLocalServer(id: string): Promise<TunnelStatus> { await ensureRoomAvailable(); return api.invoke('local_server_share', { id, identity: await roomIdentity(), accountId: account.value?.id }); }
export async function shareExistingServer(port: number): Promise<TunnelStatus> { await ensureRoomAvailable(); return api.invoke('local_server_share_existing', { port, identity: await roomIdentity(), accountId: account.value?.id }); }


export interface ManagedSkinSupport {name:string;required:boolean;valid:boolean;clientComponentRequired:boolean}
export const serverManagedComponents = (id:string):Promise<ManagedSkinSupport|null> => serverManage({action:'managed_components',id});
