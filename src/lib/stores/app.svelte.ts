let devMode = $state(false);
let performanceMode = $state(false);
let showCutscene = $state(false);

export const appState = {
	get performanceMode() { return performanceMode; },
	set performanceMode(v: boolean) { performanceMode = v; },
	get devMode() { return devMode; },
	set devMode(v: boolean) { devMode = v; },
	get showCutscene() { return showCutscene; },
	set showCutscene(v: boolean) { showCutscene = v; },
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
		logEntries = [...logEntries, {
			id: ++logIdCounter,
			timestamp: new Date(),
			stream,
			message,
		}].slice(-2000);
	},
};
