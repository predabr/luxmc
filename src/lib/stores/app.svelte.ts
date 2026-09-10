let devMode = $state(false);
let performanceMode = $state(false);
let showCutscene = $state(false);
let isGameRunning = $state(false);
let activeGameDetails = $state<{ name: string; version: string; loader: string } | null>(null);

export const appState = {
	get performanceMode() { return performanceMode; },
	set performanceMode(v: boolean) { performanceMode = v; },
	get devMode() { return devMode; },
	set devMode(v: boolean) { devMode = v; },
	get showCutscene() { return showCutscene; },
	set showCutscene(v: boolean) { showCutscene = v; },
	get isGameRunning() { return isGameRunning; },
	set isGameRunning(v: boolean) { isGameRunning = v; },
	get activeGameDetails() { return activeGameDetails; },
	set activeGameDetails(v: { name: string; version: string; loader: string } | null) { activeGameDetails = v; },
	playCutscene() { showCutscene = true; },
};

export interface LogEntry {
	id: number;
	timestamp: Date;
	stream: "stdout" | "stderr" | "system";
	message: string;
}

let logEntries = $state<LogEntry[]>([]);
let logIdCounter = 0;

export const gameLogs = {
	get entries() { return logEntries; },
	clear() { logEntries = []; },
	add(stream: "stdout" | "stderr" | "system", message: string) {
		const newEntry: LogEntry = {
			id: ++logIdCounter,
			timestamp: new Date(),
			stream,
			message,
		};
		if (logEntries.length >= 800) {
			logEntries = [...logEntries.slice(150), newEntry];
		} else {
			logEntries = [...logEntries, newEntry];
		}
	},
};
