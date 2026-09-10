import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const api = {
	invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		return invoke<T>(cmd, args);
	}
};

export interface DownloadSpeed {
	bytesPerSecond: number;
	totalDownloaded: number;
	elapsedMs: number;
}

export interface DownloadProgress {
	phase: string;
	total: number;
	completed: number;
	currentFile: string;
	bytesDownloaded: number;
	totalBytes: number;
	speed: DownloadSpeed | null;
}

export async function listenDownloadProgress(
	callback: (progress: DownloadProgress) => void,
): Promise<UnlistenFn> {
	return listen<DownloadProgress>("download-progress", (event) => {
		callback(event.payload);
	});
}

export async function ping(): Promise<string> {
	return api.invoke<string>("ping");
}

export async function authBegin(): Promise<{ state: string; verifier: string; url: string }> {
	return api.invoke("auth_begin");
}

export async function authLogin(): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
}> {
	return api.invoke("auth_login");
}

export async function authComplete(code: string, state: string, verifier: string): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
}> {
	return api.invoke("auth_complete", { code, stateToken: state, verifier });
}

export async function authRefresh(refreshToken: string): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
}> {
	return api.invoke("auth_refresh", { refreshToken });
}

export async function authAccounts(): Promise<Array<{
	id: string;
	username: string;
	uuid: string;
	refreshToken: string;
	accessToken: string | null;
	expiresAt: string | null;
	createdAt: string;
	updatedAt: string;
}>> {
	return api.invoke("auth_accounts");
}

export async function authRemove(uuid: string): Promise<void> {
	return api.invoke("auth_remove", { uuid });
}

export async function authDevLogin(): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
}> {
	return api.invoke("auth_dev_login");
}

export async function authOfflineLogin(username: string): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
}> {
	return api.invoke("auth_offline_login", { username });
}

export async function profilesList(): Promise<Array<{
	id: string;
	name: string;
	icon: string;
	mcVersion: string;
	loader: string;
	loaderVersion: string | null;
	javaPath: string | null;
	jvmArgs: string | null;
	resolutionW: number | null;
	resolutionH: number | null;
	fullscreen: boolean;
	gameDir: string;
	createdAt: string;
	updatedAt: string;
}>> {
	return api.invoke("profiles_list");
}

export async function profilesGet(id: string): Promise<{
	id: string;
	name: string;
	icon: string;
	mcVersion: string;
	loader: string;
	loaderVersion: string | null;
	javaPath: string | null;
	jvmArgs: string | null;
	resolutionW: number | null;
	resolutionH: number | null;
	fullscreen: boolean;
	gameDir: string;
	createdAt: string;
	updatedAt: string;
}> {
	return api.invoke("profiles_get", { id });
}

export async function profilesCreate(input: {
	name: string;
	icon?: string;
	mcVersion: string;
	loader: string;
	loaderVersion?: string;
	javaPath?: string;
	jvmArgs?: string;
	resolutionW?: number;
	resolutionH?: number;
	fullscreen?: boolean;
	gameDir?: string;
}): Promise<{
	id: string;
	name: string;
	icon: string;
	mcVersion: string;
	loader: string;
	loaderVersion: string | null;
	javaPath: string | null;
	jvmArgs: string | null;
	resolutionW: number | null;
	resolutionH: number | null;
	fullscreen: boolean;
	gameDir: string;
	createdAt: string;
	updatedAt: string;
}> {
	return api.invoke("profiles_create", { input });
}

export async function profilesUpdate(input: {
	id: string;
	name?: string;
	icon?: string;
	mcVersion?: string;
	loader?: string;
	loaderVersion?: string | null;
	javaPath?: string | null;
	jvmArgs?: string | null;
	resolutionW?: number | null;
	resolutionH?: number | null;
	fullscreen?: boolean;
	gameDir?: string;
	ramMb?: number;
}): Promise<{
	id: string;
	name: string;
	icon: string;
	mcVersion: string;
	loader: string;
	loaderVersion: string | null;
	javaPath: string | null;
	jvmArgs: string | null;
	resolutionW: number | null;
	resolutionH: number | null;
	fullscreen: boolean;
	gameDir: string;
	createdAt: string;
	updatedAt: string;
}> {
	return api.invoke("profiles_update", { input });
}

export async function profilesDelete(id: string): Promise<void> {
	return api.invoke("profiles_delete", { id });
}

export async function versionsList(): Promise<{
	versions: Array<{
		id: string;
		versionType: string;
		url: string;
		releaseTime: string;
	}>;
	latestRelease: string;
	latestSnapshot: string;
}> {
	return api.invoke("versions_list");
}

export async function fetchVersionsDirect(): Promise<{
	versions: Array<{ id: string; versionType: string; releaseTime: string }>;
	latestRelease: string;
	latestSnapshot: string;
}> {
	const resp = await fetch("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json");
	const data = await resp.json();
	return {
		versions: data.versions.map((v: any) => ({
			id: v.id,
			versionType: v.type,
			releaseTime: v.releaseTime,
		})),
		latestRelease: data.latest.release,
		latestSnapshot: data.latest.snapshot,
	};
}

export async function versionsDetail(id: string): Promise<{
	id: string;
	versionType: string;
	mainClass: string;
	assetIndex: { id: string; url: string; size: number };
}> {
	return api.invoke("versions_detail", { id });
}

export async function versionsDownload(id: string): Promise<void> {
	return api.invoke("versions_download", { id });
}

export async function versionsCheckInstalled(id: string): Promise<boolean> {
	return api.invoke("versions_check_installed", { id });
}

export async function launchGame(request: {
	versionId: string;
	accountId: string;
	profileId: string;
	enableVulkan?: boolean;
}): Promise<{ pid: number }> {
	return api.invoke("launch_game", { request });
}

export async function loadersVersions(loader: string, mcVersion: string): Promise<{
	versions: Array<{
		id: string;
		stable: boolean;
	}>;
}> {
	return api.invoke("loaders_versions", { loader, mcVersion });
}

export async function modsSearch(
	query: string,
	mcVersion: string,
	limit?: number,
	offset?: number,
	contentType?: string
): Promise<Array<{
	slug: string;
	title: string;
	description: string;
	downloads: number;
	iconUrl: string | null;
	categories: string[];
	versions: string[];
	source: string;
	sourceId: string;
}>> {
	return api.invoke("mods_search", { query, mcVersion, limit, offset, contentType });
}

export async function modsSearchTyped(
	query: string,
	mcVersion: string,
	contentType: string,
	limit?: number,
	offset?: number
): Promise<Array<{
	slug: string;
	title: string;
	description: string;
	downloads: number;
	iconUrl: string | null;
	categories: string[];
	versions: string[];
	source: string;
	sourceId: string;
}>> {
	return api.invoke("mods_search_typed", { query, mcVersion, contentType, limit, offset });
}

export interface ModFile {
	url: string;
	filename: string;
	size: number;
	sha1: string;
}

export interface ModVersion {
	id: string;
	name: string;
	versionNumber: string;
	files: ModFile[];
}

export async function modsVersions(projectId: string, mcVersion: string, source?: string): Promise<ModVersion[]> {
	return api.invoke("mods_versions", { projectId, mcVersion, source });
}

export interface ModGalleryImage {
	url: string;
	title?: string | null;
	description?: string | null;
}

export interface ModAuthor {
	name: string;
	avatarUrl?: string | null;
	role?: string | null;
}

export interface ModProjectDetails {
	id: string;
	slug: string;
	title: string;
	description: string;
	body: string;
	bodyType: "markdown" | "html";
	iconUrl: string | null;
	downloads: number;
	categories: string[];
	loaders: string[];
	gameVersions: string[];
	latestVersion?: string | null;
	updatedAt?: string | null;
	createdAt?: string | null;
	source: string;
	sourceUrl?: string | null;
	issuesUrl?: string | null;
	discordUrl?: string | null;
	wikiUrl?: string | null;
	donationUrl?: string | null;
	author?: ModAuthor | null;
	gallery: ModGalleryImage[];
}

export async function modsProjectDetails(projectId: string, source?: string): Promise<ModProjectDetails> {
	return api.invoke("mods_project_details", { projectId, source });
}

export async function modsList(profileId: string): Promise<Array<{
	profileId: string;
	projectId: string;
	versionId: string;
	fileName: string;
	sha1: string;
	source: string;
	installedAt: string;
}>> {
	return api.invoke("mods_list", { profileId });
}

export async function modsInstall(request: {
	profileId: string;
	projectId: string;
	versionId: string;
	source: string;
}): Promise<void> {
	return api.invoke("mods_install", { request });
}

export async function modsRemove(profileId: string, projectId: string): Promise<void> {
	return api.invoke("mods_remove", { profileId, projectId });
}

export async function modsCheckUpdates(profileId: string): Promise<Array<{
	projectId: string;
	projectTitle: string;
	currentVersionId: string;
	currentVersionNumber: string;
	latestVersionId: string;
	latestVersionNumber: string;
}>> {
	return api.invoke("mods_check_updates", { profileId });
}

export async function modsUpdate(projectId: string, versionId: string, profileId: string): Promise<void> {
	return api.invoke("mods_update", { projectId, versionId, profileId });
}

export async function instanceImportModpack(filePath: string, profileName: string, mcVersion: string, loader: string): Promise<{
	id: string;
	name: string;
}> {
	return api.invoke("instance_import_modpack", { filePath, profileName, mcVersion, loader });
}

export async function instanceImportMrpack(filePath: string, profileName: string): Promise<{
	id: string;
	name: string;
	mcVersion: string;
	loader: string;
	gameDir: string;
}> {
	return api.invoke("instance_import_mrpack", { filePath, profileName });
}

export async function instancesList(): Promise<Array<{
	id: string;
	name: string;
	icon: string;
	mcVersion: string;
	loader: string;
	loaderVersion: string | null;
	javaPath: string | null;
	jvmArgs: string | null;
	resolutionW: number | null;
	resolutionH: number | null;
	fullscreen: boolean;
	gameDir: string;
	createdAt: string;
	updatedAt: string;
}>> {
	return api.invoke("instances_list");
}

export async function instancesDuplicate(id: string): Promise<{
	id: string;
	name: string;
	icon: string;
	mcVersion: string;
	loader: string;
	loaderVersion: string | null;
	gameDir: string;
	createdAt: string;
	updatedAt: string;
}> {
	return api.invoke("instances_duplicate", { id });
}

export async function instancesOpenFolder(id: string): Promise<void> {
	return api.invoke("instances_open_folder", { id });
}

export async function instancesScreenshots(id: string): Promise<Array<{
	name: string;
	path: string;
	modified: string;
	dataUrl?: string | null;
}>> {
	return api.invoke("instances_screenshots", { id });
}

export async function screenshotDelete(path: string): Promise<void> {
	return api.invoke("screenshot_delete", { path });
}

export async function screenshotsOpenFolder(profileId: string): Promise<void> {
	return api.invoke("screenshots_open_folder", { profileId });
}

export async function authSwitchAccount(uuid: string): Promise<void> {
	return api.invoke("auth_switch_account", { uuid });
}

export interface ServerStatus {
	online: boolean;
	version: string;
	playersMax: number;
	playersOnline: number;
	motd: string;
	favicon?: string;
	latencyMs?: number;
}

export async function serverPing(host: string, port: number): Promise<ServerStatus> {
	return api.invoke<ServerStatus>("server_ping", { host, port });
}

export async function serverAdd(host: string, port: number, name: string) {
	return api.invoke("server_add", { host, port, name });
}

export async function serverList() {
	return api.invoke("server_list");
}

export async function serverRemove(id: string) {
	return api.invoke("server_remove", { id });
}

export async function serverFavorite(id: string, favorite: boolean) {
	return api.invoke("server_favorite", { id, favorite });
}

export async function settingsGet(): Promise<unknown> {
	return api.invoke("settings_get");
}

export async function settingsSet(value: unknown): Promise<void> {
	return api.invoke("settings_set", { value });
}

export async function appInit(): Promise<{
	devMode: boolean;
	account: {
		id: string;
		username: string;
		uuid: string;
		refreshToken: string;
		accessToken: string | null;
		expiresAt: string | null;
		createdAt: string;
		updatedAt: string;
	} | null;
	profiles: Array<{
		id: string;
		name: string;
		icon: string;
		mcVersion: string;
		loader: string;
		loaderVersion: string | null;
		javaPath: string | null;
		jvmArgs: string | null;
		resolutionW: number | null;
		resolutionH: number | null;
		fullscreen: boolean;
		gameDir: string;
		createdAt: string;
		updatedAt: string;
		favorite: boolean;
		notes: string | null;
		lastPlayed: string | null;
		launchCount: number;
		modCount: number;
		diskUsage: number;
		ramMb: number | null;
		instanceGroup: string | null;
	}>;
	activeProfileId: string | null;
}> {
	return api.invoke("app_init");
}

export interface EnvIssue {
	code: string;
	message: string;
	fix: string;
}

export interface EnvCheckResult {
	ok: boolean;
	issues: EnvIssue[];
}

export async function envCheck(): Promise<EnvCheckResult> {
	return api.invoke("env_check");
}

export interface GameLogEntry {
	stream: string;
	message: string;
}

export interface GameExitEvent {
	versionId: string;
	code: number;
	success: boolean;
}

export async function listenGameLog(
	callback: (entry: GameLogEntry) => void,
): Promise<UnlistenFn> {
	return listen<GameLogEntry>("game-log", (event) => {
		callback(event.payload);
	});
}

export async function listenGameExit(
	callback: (event: GameExitEvent) => void,
): Promise<UnlistenFn> {
	return listen<GameExitEvent>("game-exit", (event) => {
		callback(event.payload);
	});
}

export async function listenLauncherLog(
	callback: (message: string) => void,
): Promise<UnlistenFn> {
	return listen<string>("launcher-log", (event) => {
		callback(event.payload);
	});
}

export interface HealthCheckResult {
	clientJar: boolean;
	natives: boolean;
	modsOk: boolean;
	issues: string[];
}

export async function instanceHealthCheck(profileId: string): Promise<HealthCheckResult> {
	return api.invoke("instance_health_check", { profileId });
}

export interface FileTreeEntry {
	name: string;
	path: string;
	isDir: boolean;
	size: number;
}

export async function instanceFileTree(profileId: string, subPath?: string): Promise<FileTreeEntry[]> {
	return api.invoke("instance_file_tree", { profileId, subPath });
}

export async function instanceExport(profileId: string, destPath: string): Promise<string> {
	return api.invoke("instance_export", { profileId, destPath });
}

export async function instanceSetNotes(profileId: string, notes: string | null): Promise<void> {
	return api.invoke("instance_set_notes", { profileId, notes });
}

export async function instanceSetFavorite(profileId: string, favorite: boolean): Promise<void> {
	return api.invoke("instance_set_favorite", { profileId, favorite });
}

export async function instanceModToggle(profileId: string, fileName: string, enabled: boolean): Promise<string> {
	return api.invoke<string>("instance_mod_toggle", { profileId, fileName, enabled });
}

export async function instanceModDelete(profileId: string, fileName: string): Promise<void> {
	return api.invoke("instance_mod_delete", { profileId, fileName });
}

export async function instanceModAdd(profileId: string, sourcePath: string): Promise<string> {
	return api.invoke<string>("instance_mod_add", { profileId, sourcePath });
}

export async function instanceModsOpenFolder(profileId: string): Promise<void> {
	return api.invoke("instance_mods_open_folder", { profileId });
}

export async function instancePackAdd(profileId: string, packType: string, sourcePath: string): Promise<string> {
	return api.invoke<string>("instance_pack_add", { profileId, packType, sourcePath });
}

export async function instancePackDelete(profileId: string, packType: string, fileName: string): Promise<void> {
	return api.invoke("instance_pack_delete", { profileId, packType, fileName });
}

export async function instancePackOpenFolder(profileId: string, packType: string): Promise<void> {
	return api.invoke("instance_pack_open_folder", { profileId, packType });
}

export async function shareLogMclogs(content: string): Promise<string> {
	return api.invoke<string>("share_log_mclogs", { content });
}

// --- Launch logs / persistent session log ---

export interface LaunchLogSummary {
	id: number;
	profileId: string | null;
	versionId: string;
	startedAt: string;
	endedAt: string | null;
	exitCode: number | null;
	summary: string | null;
	errorClassification: string | null;
	lineCount: number;
}

export interface LaunchLogLine {
	id: number;
	logId: number;
	stream: string;
	level: string;
	message: string;
	ts: string;
}

export async function launchLogsList(limit = 100): Promise<LaunchLogSummary[]> {
	return api.invoke<LaunchLogSummary[]>("launch_logs_list", { limit });
}

export async function launchLogsGet(id: number): Promise<LaunchLogLine[]> {
	return api.invoke<LaunchLogLine[]>("launch_logs_get", { id });
}

export async function launchLogsSearch(query: string, minLevel: string, limit = 200): Promise<LaunchLogLine[]> {
	return api.invoke<LaunchLogLine[]>("launch_logs_search", { query, minLevel, limit });
}

export async function launchLogsClear(): Promise<void> {
	return api.invoke("launch_logs_clear");
}

export async function launchLogOpen(profileId: string | null, versionId: string): Promise<number> {
	return api.invoke<number>("launch_log_open", { profileId, versionId });
}

export async function launchLogAppend(logId: number, stream: string, level: string, message: string): Promise<void> {
	return api.invoke("launch_log_append", { logId, stream, level, message });
}

export async function launchLogClose(logId: number, exitCode: number | null, summary: string | null, errorClassification: string | null): Promise<void> {
	return api.invoke("launch_log_close", { logId, exitCode, summary, errorClassification });
}

// --- Instance tools: JVM validation, Java runtime, crash summary, backup/restore ---

export interface JvmValidationResult {
	valid: boolean;
	rejected: string[];
	normalized: string;
	suggestions: string[];
}

export async function jvmArgsValidate(input: string): Promise<JvmValidationResult> {
	return api.invoke<JvmValidationResult>("jvm_args_validate", { input });
}

export interface JavaRuntimeInfo {
	available: boolean;
	major: number;
	path: string | null;
	version: string | null;
}

export async function javaRuntimeStatus(major: number): Promise<JavaRuntimeInfo> {
	return api.invoke<JavaRuntimeInfo>("java_runtime_status", { major });
}

export interface CrashSummary {
	category: string;
	probableCause: string;
	hint: string;
}

export async function crashSummary(lines: string[]): Promise<CrashSummary> {
	return api.invoke<CrashSummary>("crash_summary", { lines });
}

export async function instanceExportZip(profileId: string, outputPath: string): Promise<string> {
	return api.invoke<string>("instance_export_zip", { profileId, outputPath });
}

export async function instanceBackupSaves(profileId: string, outputPath: string): Promise<string> {
	return api.invoke<string>("instance_backup_saves", { profileId, outputPath });
}

export async function instanceRestoreSaves(profileId: string, zipPath: string): Promise<string> {
	return api.invoke<string>("instance_restore_saves", { profileId, zipPath });
}

export async function instanceRepair(profileId: string): Promise<void> {
	return api.invoke("instance_repair", { profileId });
}

export async function instanceDiskUsage(profileId: string): Promise<number> {
	return api.invoke<number>("instance_disk_usage", { profileId });
}

export async function versionRepair(versionId: string): Promise<void> {
	return api.invoke("version_repair", { versionId });
}

// --- Storage breakdown ---

export interface StorageBreakdown {
	category: string;
	bytes: number;
	path: string;
}

export async function storageBreakdown(): Promise<StorageBreakdown[]> {
	return api.invoke<StorageBreakdown[]>("storage_breakdown");
}

// --- Changelog ---

export interface ChangelogEntry {
	version: string;
	date: string;
	title: string;
	highlights: string[];
}

export async function changelogGet(): Promise<ChangelogEntry[]> {
	return api.invoke<ChangelogEntry[]>("changelog_get");
}

// --- File system helpers ---

export async function storageTotal(paths: string[]): Promise<number> {
	return api.invoke<number>("storage_total", { paths });
}

export async function directoryExists(path: string): Promise<boolean> {
	return api.invoke<boolean>("directory_exists", { path });
}

export async function ensureDirectory(path: string): Promise<void> {
	return api.invoke("ensure_directory", { path });
}

export async function readTextFile(path: string): Promise<string> {
	return api.invoke<string>("read_text_file", { path });
}

export async function writeTextFile(path: string, contents: string): Promise<void> {
	return api.invoke("write_text_file", { path, contents });
}

export async function deleteFileOrDir(path: string): Promise<void> {
	return api.invoke("delete_file_or_dir", { path });
}


// --- P2P Direct LAN/PC Messaging ---

export interface P2PConnectionInfo {
	ip: string;
	port: number;
}

export interface P2PMessagePayload {
	sender: string;
	text: string;
	timestamp: string;
}

export async function p2pGetLocalInfo(): Promise<P2PConnectionInfo> {
	return api.invoke<P2PConnectionInfo>("p2p_get_local_info");
}

export interface HostLinkInfo {
	localIp: string;
	port: number;
	shareLink: string;
	directAddress: string;
}

export async function p2pGetHostLink(port?: number): Promise<HostLinkInfo> {
	return api.invoke<HostLinkInfo>("p2p_get_host_link", { port });
}

export async function p2pStartListener(): Promise<boolean> {
	return api.invoke<boolean>("p2p_start_listener");
}

export async function p2pSendMessage(targetAddress: string, sender: string, text: string): Promise<boolean> {
	return api.invoke<boolean>("p2p_send_message", { targetAddress, sender, text });
}

export async function listenP2PMessage(
	callback: (payload: P2PMessagePayload) => void
): Promise<UnlistenFn> {
	return listen<P2PMessagePayload>("p2p-chat-message", (event) => {
		callback(event.payload);
	});
}

// --- Real World Scanner ---

export interface WorldDetail {
	name: string;
	folderName: string;
	iconBase64: string | null;
	lastPlayed: number | null;
	gameMode: string | null;
	sizeBytes: number;
}

export async function instanceWorldsList(profileId: string): Promise<WorldDetail[]> {
	return api.invoke<WorldDetail[]>("instance_worlds_list", { profileId });
}

export async function instanceWorldDelete(profileId: string, folderName: string): Promise<void> {
	return api.invoke<void>("instance_world_delete", { profileId, folderName });
}


export interface DiscordActivityOptions {
	details?: string;
	state?: string;
	largeText?: string;
	largeImage?: string;
	smallText?: string;
	smallImage?: string;
	startTime?: number;
	inGame?: boolean;
	clientId?: string;
}

export async function discordSetActivity(
	detailsOrOptions?: string | DiscordActivityOptions,
	state?: string,
	largeText?: string,
	largeImage?: string
): Promise<boolean> {
	if (typeof detailsOrOptions === "object" && detailsOrOptions !== null) {
		return api.invoke<boolean>("discord_set_activity", {
			details: detailsOrOptions.details,
			state: detailsOrOptions.state,
			largeText: detailsOrOptions.largeText,
			largeImage: detailsOrOptions.largeImage,
			smallText: detailsOrOptions.smallText,
			smallImage: detailsOrOptions.smallImage,
			startTime: detailsOrOptions.startTime,
			inGame: detailsOrOptions.inGame,
			clientId: detailsOrOptions.clientId
		});
	}
	return api.invoke<boolean>("discord_set_activity", { details: detailsOrOptions, state, largeText, largeImage });
}

export async function discordClearActivity(): Promise<void> {
	return api.invoke("discord_clear_activity");
}

export async function getSystemSpecs(): Promise<{
	osDistro: string;
	kernelVersion: String;
	arch: string;
	totalRamMb: number;
	launcherVersion: string;
}> {
	return api.invoke("get_system_specs");
}
