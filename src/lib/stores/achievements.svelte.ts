import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { fireAchievementConfetti } from "$lib/utils/confetti";
import { toast } from "$lib/stores/toasts.svelte";
import { playSound } from "$lib/utils/sound";

export interface Achievement {
	id: string;
	title: string;
	description: string;
	icon: string;
	unlockedAt?: string;
}

export const ACHIEVEMENTS_LIST: Achievement[] = [
	{
		id: "primeira_noite",
		title: "Primeira Noite",
		description: uiText("ui.9b65f3504b87cddf"),
		icon: "🌙"
	},
	{
		id: "mestre_mods",
		title: "Mestre dos Modpacks",
		description: uiText("ui.f81e40c517c74537"),
		icon: "📦"
	},
	{
		id: "lux_boost",
		title: "Lux Boost Turbo",
		description: uiText("ui.43d6ffcb579b6ecb"),
		icon: "⚡"
	},
	{
		id: "veterano",
		title: "Veterano do Bloco",
		description: uiText("ui.a907c19158e26b21"),
		icon: "👑"
	},
	{
		id: "share_code",
		title: uiText("ui.d06e660f2e9b5369"),
		description: uiText("ui.2cee57c0e7c40b18"),
		icon: "🚀"
	}
];

function createAchievementsStore() {
	let unlockedIds = $state<string[]>([]);

	if (typeof window !== "undefined") {
		try {
			const saved = localStorage.getItem("luxmc_unlocked_achievements");
			if (saved) {
				unlockedIds = JSON.parse(saved);
			}
		} catch {
		}
	}

	function unlock(id: string) {
		if (unlockedIds.includes(id)) return;
		const ach = ACHIEVEMENTS_LIST.find((a) => a.id === id);
		if (!ach) return;

		unlockedIds = [...unlockedIds, id];
		if (typeof window !== "undefined") {
			try {
				localStorage.setItem("luxmc_unlocked_achievements", JSON.stringify(unlockedIds));
			} catch {
			}
		}

		playSound("achievement");
		fireAchievementConfetti();

		toast(`🏆 Conquista Desbloqueada: ${ach.title}!`, "success");
	}

	return {
		get unlocked() {
			return unlockedIds;
		},
		isUnlocked(id: string) {
			return unlockedIds.includes(id);
		},
		unlock
	};
}

export const achievements = createAchievementsStore();
