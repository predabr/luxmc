const STORAGE_TOTAL_PLAYTIME_MINS = "luxmc_total_playtime_minutes";
const STORAGE_LAST_SESSION_MINS = "luxmc_last_session_minutes";
const STORAGE_TOTAL_LAUNCHES = "luxmc_total_launches";
const STORAGE_DAILY_PLAYTIME = "luxmc_daily_playtime_map";

function getTodayKey(): string {
	const now = new Date();
	const year = now.getFullYear();
	const month = String(now.getMonth() + 1).padStart(2, "0");
	const day = String(now.getDate()).padStart(2, "0");
	return `${year}-${month}-${day}`;
}

let totalMinutes = $state(20);
let lastSessionMinutes = $state(20);
let totalLaunches = $state(4);
let dailyMinutesMap = $state<Record<string, number>>({});
let activeGameStartTime = $state<number | null>(null);
let activeSessionMinutes = $state(0);

let activeInterval: ReturnType<typeof setInterval> | null = null;

function init() {
	if (typeof window === "undefined") return;
	const savedTotal = localStorage.getItem(STORAGE_TOTAL_PLAYTIME_MINS);
	const savedLast = localStorage.getItem(STORAGE_LAST_SESSION_MINS);
	const savedLaunches = localStorage.getItem(STORAGE_TOTAL_LAUNCHES);
	const savedDaily = localStorage.getItem(STORAGE_DAILY_PLAYTIME);

	if (savedTotal !== null) totalMinutes = parseInt(savedTotal, 10) || 0;
	if (savedLast !== null) lastSessionMinutes = parseInt(savedLast, 10) || 0;
	if (savedLaunches !== null) totalLaunches = parseInt(savedLaunches, 10) || 0;
	if (savedDaily) {
		try {
			dailyMinutesMap = JSON.parse(savedDaily);
		} catch {
			dailyMinutesMap = {};
		}
	} else {
		dailyMinutesMap = { [getTodayKey()]: 20 };
	}
}

function saveDaily() {
	if (typeof window === "undefined") return;
	try {
		localStorage.setItem(STORAGE_DAILY_PLAYTIME, JSON.stringify(dailyMinutesMap));
	} catch {}
}

export const gamingStats = {
	get totalMinutes() { return totalMinutes; },
	get lastSessionMinutes() { return lastSessionMinutes; },
	get totalLaunches() { return totalLaunches; },
	get isPlaying() { return activeGameStartTime !== null; },
	get activeSessionMinutes() { return activeSessionMinutes; },

	get todayMinutes() {
		const todayKey = getTodayKey();
		const baseToday = dailyMinutesMap[todayKey] || 0;
		return baseToday + (activeGameStartTime ? activeSessionMinutes : 0);
	},

	get formattedTodayTime() {
		const mins = gamingStats.todayMinutes;
		if (mins < 60) {
			return `${mins} ${mins === 1 ? 'minuto' : 'minutos'}`;
		}
		const h = Math.floor(mins / 60);
		const m = mins % 60;
		if (m === 0) return `${h} ${h === 1 ? 'hora' : 'horas'}`;
		return `${h}h ${m}m`;
	},

	get formattedTotalTime() {
		const currentTotal = totalMinutes + (activeGameStartTime ? activeSessionMinutes : 0);
		const h = Math.floor(currentTotal / 60);
		const m = currentTotal % 60;
		if (h === 0) return `${m}m`;
		return `${h}h ${m}m`;
	},

	get formattedLastSession() {
		if (activeGameStartTime !== null) {
			return `${activeSessionMinutes}m (Em jogo)`;
		}
		const h = Math.floor(lastSessionMinutes / 60);
		const m = lastSessionMinutes % 60;
		if (h === 0) return `${m}m`;
		return `${h}h ${m}m`;
	},

	onGameStart() {
		activeGameStartTime = Date.now();
		activeSessionMinutes = 1;
		totalLaunches += 1;
		if (typeof window !== "undefined") {
			localStorage.setItem(STORAGE_TOTAL_LAUNCHES, String(totalLaunches));
		}

		if (activeInterval) clearInterval(activeInterval);
		activeInterval = setInterval(() => {
			if (activeGameStartTime) {
				const elapsed = Math.max(1, Math.floor((Date.now() - activeGameStartTime) / 60000));
				activeSessionMinutes = elapsed;
			}
		}, 10000);
	},

	onGameExit() {
		if (activeInterval) {
			clearInterval(activeInterval);
			activeInterval = null;
		}

		if (activeGameStartTime) {
			const elapsed = Math.max(1, Math.round((Date.now() - activeGameStartTime) / 60000));
			totalMinutes += elapsed;
			lastSessionMinutes = elapsed;

			const todayKey = getTodayKey();
			dailyMinutesMap[todayKey] = (dailyMinutesMap[todayKey] || 0) + elapsed;
			saveDaily();

			if (typeof window !== "undefined") {
				localStorage.setItem(STORAGE_TOTAL_PLAYTIME_MINS, String(totalMinutes));
				localStorage.setItem(STORAGE_LAST_SESSION_MINS, String(lastSessionMinutes));
			}
			activeGameStartTime = null;
			activeSessionMinutes = 0;
		}
	}
};

init();
