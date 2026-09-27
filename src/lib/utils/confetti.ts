import confetti from "canvas-confetti";

export function fireModpackSuccessConfetti(): void {
	try {
		confetti({
			particleCount: 45,
			spread: 60,
			origin: { y: 0.8 },
			colors: ["#10b981", "#06b6d4", "#8b5cf6", "#f59e0b"],
			disableForReducedMotion: true,
			zIndex: 9999
		});
	} catch {}
}

export function fireAchievementConfetti(): void {
	try {
		confetti({
			particleCount: 55,
			spread: 80,
			origin: { y: 0.65 },
			colors: ["#fbbf24", "#f59e0b", "#d97706", "#ffffff"],
			disableForReducedMotion: true,
			zIndex: 9999
		});
	} catch {}
}
