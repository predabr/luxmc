let devMode = $state(false);
let performanceMode = $state(false);
let isScrolling = $state(false);
let showCutscene = $state(false);
let showProfileModal = $state(false);
let isGameRunning = $state(false);
let isStopping = $state(false);
let isLaunching = $state(false);
let launchStatusText = $state("");
let launchingProfileId = $state<string | null>(null);
let wasManuallyTerminated = $state(false);
let activeGameDetails = $state<{ name: string; version: string; loader: string; profileId?: string } | null>(null);

export const appState = {
    get isScrolling() { return isScrolling; },
    set isScrolling(v: boolean) { isScrolling = v; },
	get performanceMode() { return performanceMode; },
	set performanceMode(v: boolean) { performanceMode = v; },
	get devMode() { return devMode; },
	set devMode(v: boolean) { devMode = v; },
	get showCutscene() { return showCutscene; },
	set showCutscene(v: boolean) { showCutscene = v; },
	get showProfileModal() { return showProfileModal; },
	set showProfileModal(v: boolean) { showProfileModal = v; },
	get isGameRunning() { return isGameRunning; },
	set isGameRunning(v: boolean) { isGameRunning = v; },
	get isStopping() { return isStopping; },
	set isStopping(v: boolean) { isStopping = v; },
	get isLaunching() { return isLaunching; },
	set isLaunching(v: boolean) { isLaunching = v; },
	get launchStatusText() { return launchStatusText; },
	set launchStatusText(v: string) { launchStatusText = v; },
	get launchingProfileId() { return launchingProfileId; },
	set launchingProfileId(v: string | null) { launchingProfileId = v; },
	get wasManuallyTerminated() { return wasManuallyTerminated; },
	set wasManuallyTerminated(v: boolean) { wasManuallyTerminated = v; },
	get activeGameDetails() { return activeGameDetails; },
	set activeGameDetails(v: { name: string; version: string; loader: string; profileId?: string } | null) { activeGameDetails = v; },
	playCutscene() { showCutscene = true; },
};

export interface LogEntry {
	id: number;
	timestamp: Date;
	stream: "stdout" | "stderr" | "system" | "game";
	message: string;
}

let logEntries = $state<LogEntry[]>([]);
let logIdCounter = 0;

export const gameLogs = {
	get entries() { return logEntries; },
	clear() { logEntries = []; },
	add(stream: "stdout" | "stderr" | "system" | "game", message: string) {
		const lines = message.split("\n");
		for (const line of lines) {
			const trimmed = line.length > 500 ? line.slice(0, 500) + "…" : line;
			const newEntry: LogEntry = {
				id: ++logIdCounter,
				timestamp: new Date(),
				stream,
				message: trimmed,
			};
			if (logEntries.length >= 800) {
				logEntries.splice(0, 150);
				logEntries.push(newEntry);
			} else {
				logEntries.push(newEntry);
			}
		}
	},
};
