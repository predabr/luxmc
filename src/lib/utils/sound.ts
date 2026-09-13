import { settings } from "$lib/stores/settings.svelte";

let audioCtx: AudioContext | null = null;

function getAudioContext(): AudioContext | null {
	if (typeof window === "undefined") return null;
	if (!audioCtx) {
		const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
		if (AudioContextClass) {
			audioCtx = new AudioContextClass();
		}
	}
	if (audioCtx && audioCtx.state === "suspended") {
		audioCtx.resume().catch(() => {});
	}
	return audioCtx;
}

export function playSound(type: "click" | "launch" | "chime" | "warning" | "achievement") {
	try {
		if (typeof window === "undefined") return;
		const s = settings.value as { soundEnabled?: boolean; soundVolume?: number };
		if (s && s.soundEnabled === false) return;
		const volume = typeof s?.soundVolume === "number" ? Math.max(0, Math.min(1, s.soundVolume)) : 0.5;

		const ctx = getAudioContext();
		if (!ctx) return;

		const now = ctx.currentTime;
		const masterGain = ctx.createGain();
		masterGain.gain.setValueAtTime(volume, now);
		masterGain.connect(ctx.destination);

		if (type === "click") {
			const osc = ctx.createOscillator();
			const gain = ctx.createGain();
			osc.type = "sine";
			osc.frequency.setValueAtTime(800, now);
			osc.frequency.exponentialRampToValueAtTime(400, now + 0.04);
			gain.gain.setValueAtTime(0.3, now);
			gain.gain.exponentialRampToValueAtTime(0.001, now + 0.04);
			osc.connect(gain);
			gain.connect(masterGain);
			osc.start(now);
			osc.stop(now + 0.04);
		} else if (type === "launch") {
			const osc1 = ctx.createOscillator();
			const osc2 = ctx.createOscillator();
			const gain = ctx.createGain();
			osc1.type = "sawtooth";
			osc2.type = "sine";
			osc1.frequency.setValueAtTime(110, now);
			osc1.frequency.exponentialRampToValueAtTime(440, now + 0.35);
			osc2.frequency.setValueAtTime(220, now);
			osc2.frequency.exponentialRampToValueAtTime(880, now + 0.35);
			gain.gain.setValueAtTime(0.15, now);
			gain.gain.exponentialRampToValueAtTime(0.001, now + 0.4);
			osc1.connect(gain);
			osc2.connect(gain);
			gain.connect(masterGain);
			osc1.start(now);
			osc2.start(now);
			osc1.stop(now + 0.4);
			osc2.stop(now + 0.4);
		} else if (type === "chime" || type === "achievement") {
			const notes = type === "achievement" ? [523.25, 659.25, 783.99, 1046.5] : [440, 660, 880];
			notes.forEach((freq, idx) => {
				const osc = ctx.createOscillator();
				const gain = ctx.createGain();
				const startTime = now + idx * 0.08;
				osc.type = "triangle";
				osc.frequency.setValueAtTime(freq, startTime);
				gain.gain.setValueAtTime(0.2, startTime);
				gain.gain.exponentialRampToValueAtTime(0.001, startTime + 0.3);
				osc.connect(gain);
				gain.connect(masterGain);
				osc.start(startTime);
				osc.stop(startTime + 0.3);
			});
		} else if (type === "warning") {
			const osc = ctx.createOscillator();
			const gain = ctx.createGain();
			osc.type = "square";
			osc.frequency.setValueAtTime(240, now);
			osc.frequency.setValueAtTime(180, now + 0.1);
			gain.gain.setValueAtTime(0.2, now);
			gain.gain.exponentialRampToValueAtTime(0.001, now + 0.25);
			osc.connect(gain);
			gain.connect(masterGain);
			osc.start(now);
			osc.stop(now + 0.25);
		}
	} catch {
	}
}
