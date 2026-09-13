import confetti from "canvas-confetti";
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
		description: "Iniciou sua primeira sessão de Minecraft com o Luxmc!",
		icon: "🌙"
	},
	{
		id: "mestre_mods",
		title: "Mestre dos Modpacks",
		description: "Configurou uma instância com mais de 15 mods ativos.",
		icon: "📦"
	},
	{
		id: "lux_boost",
		title: "Lux Boost Turbo",
		description: "Instalou o pacote de performance máximo do Luxmc.",
		icon: "⚡"
	},
	{
		id: "veterano",
		title: "Veterano do Bloco",
		description: "Acumulou 10 ou mais sessões de jogo.",
		icon: "👑"
	},
	{
		id: "share_code",
		title: "Mundo Compartilhado",
		description: "Gerou ou importou uma instância via código rápido LUX-XXXX.",
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
		try {
			confetti({
				particleCount: 80,
				spread: 70,
				origin: { y: 0.7 }
			});
		} catch {
		}

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
