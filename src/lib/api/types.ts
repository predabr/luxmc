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

export interface ModSearchResultItem {
	slug: string;
	title: string;
	description: string;
	downloads: number;
	iconUrl: string | null;
	bannerUrl: string | null;
	author: string | null;
	categories: string[];
	versions: string[];
	source: string;
	sourceId: string;
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

export interface ServerStatus {
	online: boolean;
	version: string;
	playersMax: number;
	playersOnline: number;
	motd: string;
	favicon?: string;
	latencyMs?: number;
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

export interface GameLogEntry {
	stream: string;
	message: string;
}

export interface GameExitEvent {
	versionId: string;
	code: number;
	success: boolean;
}

export interface HealthCheckResult {
	clientJar: boolean;
	natives: boolean;
	modsOk: boolean;
	issues: string[];
}

export interface FileTreeEntry {
	name: string;
	path: string;
	isDir: boolean;
	size: number;
}

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

export interface JvmValidationResult {
	valid: boolean;
	rejected: string[];
	normalized: string;
	suggestions: string[];
}

export interface JavaRuntimeInfo {
	available: boolean;
	major: number;
	path: string | null;
	version: string | null;
}

export interface CrashSummary {
	category: string;
	probableCause: string;
	hint: string;
}

export interface StorageBreakdown {
	category: string;
	bytes: number;
	path: string;
}

export interface ChangelogEntry {
	version: string;
	date: string;
	title: string;
	highlights: string[];
}

export interface P2PConnectionInfo {
	ip: string;
	port: number;
}

export interface P2PMessagePayload {
	sender: string;
	text: string;
	timestamp: string;
}

export interface HostLinkInfo {
	localIp: string;
	port: number;
	shareLink: string;
	directAddress: string;
}

export interface WorldDetail {
	name: string;
	folderName: string;
	iconBase64: string | null;
	lastPlayed: number | null;
	gameMode: string | null;
	sizeBytes: number;
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

export interface GpuInfo {
	vendor: string;
	renderer: string;
	driver: string;
	supportsZink: boolean;
}

export interface PerformanceModEntry {
	slug: string;
	title: string;
	description: string;
}

export interface PerformancePackInfo {
	available: boolean;
	loader: string;
	mcVersion: string;
	reason?: string | null;
	mods: PerformanceModEntry[];
}
