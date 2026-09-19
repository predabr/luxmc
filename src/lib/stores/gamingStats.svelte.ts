import { appState } from "./app.svelte";
import { gamingStatsGet, gamingStatsSave } from "$lib/api/skins";

const STORAGE_TOTAL_PLAYTIME_MINS = "luxmc_total_playtime_minutes";
const STORAGE_LAST_SESSION_MINS = "luxmc_last_session_minutes";
const STORAGE_LONGEST_SESSION_MINS = "luxmc_longest_session_minutes";
const STORAGE_TOTAL_LAUNCHES = "luxmc_total_launches";
const STORAGE_DAILY_PLAYTIME = "luxmc_daily_playtime_map";
const STORAGE_ACTIVE_START_TIME = "luxmc_active_game_start_time";

export interface DayStat {
	dateKey: string;
	dayLabel: string;
	minutes: number;
	isToday: boolean;
	formattedTime: string;
}

function getTodayKey(): string {
	const now = new Date();
	const year = now.getFullYear();
	const month = String(now.getMonth() + 1).padStart(2, "0");
	const day = String(now.getDate()).padStart(2, "0");
	return `${year}-${month}-${day}`;
}

let instanceMinutes = $state<Record<string, number>>({});
let sessionProfileId: string | null = null;
let totalMinutes = $state(0);
let lastSessionMinutes = $state(0);
let longestSessionMinutesState = $state(0);
let totalLaunches = $state(0);
let dailyMinutesMap = $state<Record<string, number>>({});
let activeGameStartTime = $state<number | null>(null);
let activeSessionMinutes = $state(0);

let activeInterval: ReturnType<typeof setInterval> | null = null;

async function init() {
	if (typeof window === "undefined") return;
	try { const saved: unknown = JSON.parse(localStorage.getItem("luxmc_instance_minutes") || "{}"); if (saved && typeof saved === "object" && !Array.isArray(saved)) instanceMinutes = Object.fromEntries(Object.entries(saved).filter((entry): entry is [string, number] => typeof entry[1] === "number" && Number.isFinite(entry[1]) && entry[1] >= 0)); } catch { instanceMinutes = {}; }
	const savedTotal = localStorage.getItem(STORAGE_TOTAL_PLAYTIME_MINS);
	const savedLast = localStorage.getItem(STORAGE_LAST_SESSION_MINS);
	const savedLongest = localStorage.getItem(STORAGE_LONGEST_SESSION_MINS);
	const savedLaunches = localStorage.getItem(STORAGE_TOTAL_LAUNCHES);
	const savedDaily = localStorage.getItem(STORAGE_DAILY_PLAYTIME);
	const savedActiveStart = localStorage.getItem(STORAGE_ACTIVE_START_TIME);

	if (savedTotal !== null) totalMinutes = parseInt(savedTotal, 10) || 0;
	if (savedLast !== null) lastSessionMinutes = parseInt(savedLast, 10) || 0;
	if (savedLongest !== null) longestSessionMinutesState = parseInt(savedLongest, 10) || 0;
	if (savedLaunches !== null) totalLaunches = parseInt(savedLaunches, 10) || 0;
	if (savedDaily) {
		try {
			dailyMinutesMap = JSON.parse(savedDaily);
		} catch {
			dailyMinutesMap = {};
		}
	} else {
		dailyMinutesMap = {};
	}

	if (savedActiveStart !== null) {
		const parsedStart = parseInt(savedActiveStart, 10);
		if (!isNaN(parsedStart) && parsedStart > 0 && Date.now() - parsedStart < 24 * 3600 * 1000) {
			activeGameStartTime = parsedStart;
			activeSessionMinutes = Math.max(1, Math.floor((Date.now() - parsedStart) / 60000));
			if (activeInterval) clearInterval(activeInterval);
			activeInterval = setInterval(() => {
				if (activeGameStartTime) {
					activeSessionMinutes = Math.max(1, Math.floor((Date.now() - activeGameStartTime) / 60000));
				}
			}, 10000);
		} else {
			localStorage.removeItem(STORAGE_ACTIVE_START_TIME);
		}
	}

	try {
		const dbStatsStr = await gamingStatsGet();
		if (dbStatsStr) {
			const parsed = JSON.parse(dbStatsStr);
			if (parsed.totalMinutes && parsed.totalMinutes > totalMinutes) {
				totalMinutes = parsed.totalMinutes;
			}
			if (parsed.longestSessionMinutes && parsed.longestSessionMinutes > longestSessionMinutesState) {
				longestSessionMinutesState = parsed.longestSessionMinutes;
			}
			if (parsed.totalLaunches && parsed.totalLaunches > totalLaunches) {
				totalLaunches = parsed.totalLaunches;
			}
			if (parsed.dailyMinutesMap) {
				dailyMinutesMap = { ...parsed.dailyMinutesMap, ...dailyMinutesMap };
			}
		}
	} catch (e) {
		console.warn("Could not load stats from DB:", e);
	}
}

function saveDaily() {
	if (typeof window === "undefined") return;
	const cutoff = Date.now() - 30 * 24 * 60 * 60 * 1000;
	for (const key of Object.keys(dailyMinutesMap)) {
		const dayMs = new Date(key).getTime();
		if (isNaN(dayMs) || dayMs < cutoff) delete dailyMinutesMap[key];
	}
	try {
		localStorage.setItem(STORAGE_DAILY_PLAYTIME, JSON.stringify(dailyMinutesMap));
	} catch {}
	persistToDb();
}

function persistToDb() {
	gamingStatsSave(JSON.stringify({
		totalMinutes,
		lastSessionMinutes,
		longestSessionMinutes: longestSessionMinutesState,
		totalLaunches,
		dailyMinutesMap,
	})).catch(e => console.warn("Failed to persist gaming stats to db:", e));
}

export const gamingStats = {
	get totalMinutes() { return totalMinutes; },
	get lastSessionMinutes() { return lastSessionMinutes; },
	get longestSessionMinutes() {
		let maxVal = Math.max(lastSessionMinutes, longestSessionMinutesState);
		for (const m of Object.values(dailyMinutesMap)) {
			if (m > maxVal) maxVal = m;
		}
		if (activeGameStartTime !== null && activeSessionMinutes > maxVal) {
			maxVal = activeSessionMinutes;
		}
		return maxVal;
	},
	get totalLaunches() { return totalLaunches; },
	get isPlaying() { return activeGameStartTime !== null; },
	get activeSessionMinutes() { return activeSessionMinutes; },

	get todayMinutes() {
		const todayKey = getTodayKey();
		const baseToday = dailyMinutesMap[todayKey] || 0;
		return baseToday + (activeGameStartTime ? activeSessionMinutes : 0);
	},

	get last7Days(): DayStat[] {
		const days: DayStat[] = [];
		const dayNames = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"];
		const now = new Date();
		for (let i = 6; i >= 0; i--) {
			const d = new Date(now);
			d.setDate(now.getDate() - i);
			const year = d.getFullYear();
			const month = String(d.getMonth() + 1).padStart(2, "0");
			const day = String(d.getDate()).padStart(2, "0");
			const dateKey = `${year}-${month}-${day}`;
			const isToday = i === 0;
			const baseMins = dailyMinutesMap[dateKey] || 0;
			const mins = isToday ? baseMins + (activeGameStartTime ? activeSessionMinutes : 0) : baseMins;
			const dayLabel = isToday ? "Hoje" : dayNames[d.getDay()];
			const formatted = mins < 60 ? `${mins}m` : `${Math.floor(mins / 60)}h ${mins % 60}m`;
			days.push({
				dateKey,
				dayLabel,
				minutes: mins,
				isToday,
				formattedTime: formatted
			});
		}
		return days;
	},

	get daysPlayedInLast7(): number {
		return gamingStats.last7Days.filter(d => d.minutes > 0).length;
	},

	get maxMinutesInLast7(): number {
		return Math.max(1, ...gamingStats.last7Days.map(d => d.minutes));
	},

	get last7DaysTotalMinutes(): number {
		return gamingStats.last7Days.reduce((acc, d) => acc + d.minutes, 0);
	},

	get formattedLast7DaysTime(): string {
		const mins = gamingStats.last7DaysTotalMinutes;
		if (mins < 60) return `${mins}m`;
		const h = Math.floor(mins / 60);
		const m = mins % 60;
		if (m === 0) return `${h}h`;
		return `${h}h ${m}m`;
	},

	get averageSessionMinutes(): number {
		const effectiveLaunches = Math.max(1, totalLaunches);
		const currentTotal = totalMinutes + (activeGameStartTime ? activeSessionMinutes : 0);
		return Math.round(currentTotal / effectiveLaunches);
	},

	get formattedAverageSession(): string {
		const mins = gamingStats.averageSessionMinutes;
		if (mins < 60) return `${mins}m`;
		const h = Math.floor(mins / 60);
		const m = mins % 60;
		if (m === 0) return `${h}h`;
		return `${h}h ${m}m`;
	},

	get formattedLongestSession(): string {
		const mins = gamingStats.longestSessionMinutes;
		if (mins < 60) return `${mins}m`;
		const h = Math.floor(mins / 60);
		const m = mins % 60;
		if (m === 0) return `${h}h`;
		return `${h}h ${m}m`;
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

	profileMinutes(id: string) { return Math.floor((instanceMinutes[id] || 0) + (sessionProfileId === id ? activeSessionMinutes : 0)); },
	onGameStart(profileId?: string) {
		if (activeGameStartTime) return;
		sessionProfileId = profileId || appState.activeGameDetails?.profileId || null;
		activeGameStartTime = Date.now();
		activeSessionMinutes = 1;
		totalLaunches += 1;
		if (typeof window !== "undefined") {
			localStorage.setItem(STORAGE_TOTAL_LAUNCHES, String(totalLaunches));
			localStorage.setItem(STORAGE_ACTIVE_START_TIME, String(activeGameStartTime));
		}
		persistToDb();

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
			if (sessionProfileId) {
				instanceMinutes[sessionProfileId] = (instanceMinutes[sessionProfileId] || 0) + elapsed;
				try { localStorage.setItem("luxmc_instance_minutes", JSON.stringify(instanceMinutes)); } catch {}
			}
			sessionProfileId = null;
			lastSessionMinutes = elapsed;
			longestSessionMinutesState = Math.max(longestSessionMinutesState, elapsed);

			const todayKey = getTodayKey();
			dailyMinutesMap[todayKey] = (dailyMinutesMap[todayKey] || 0) + elapsed;
			saveDaily();

			if (typeof window !== "undefined") {
				localStorage.setItem(STORAGE_TOTAL_PLAYTIME_MINS, String(totalMinutes));
				localStorage.setItem(STORAGE_LAST_SESSION_MINS, String(lastSessionMinutes));
				localStorage.setItem(STORAGE_LONGEST_SESSION_MINS, String(longestSessionMinutesState));
				localStorage.removeItem(STORAGE_ACTIVE_START_TIME);
			}
			activeGameStartTime = null;
			activeSessionMinutes = 0;
			persistToDb();
		}
	},

	destroy() {
		if (activeInterval) {
			clearInterval(activeInterval);
			activeInterval = null;
		}
	}
};

init();
