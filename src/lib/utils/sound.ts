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

export function playSound(type: "click" | "launch" | "chime" | "warning" | "achievement" | "delete") {
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
		} else if (type === "delete") {
			const osc = ctx.createOscillator();
			const gain = ctx.createGain();
			osc.type = "sine";
			osc.frequency.setValueAtTime(360, now);
			osc.frequency.exponentialRampToValueAtTime(120, now + 0.14);
			gain.gain.setValueAtTime(0.28, now);
			gain.gain.exponentialRampToValueAtTime(0.001, now + 0.14);
			osc.connect(gain);
			gain.connect(masterGain);
			osc.start(now);
			osc.stop(now + 0.14);
		}
	} catch {
	}
}

let soundscapeMasterGain: GainNode | null = null;
let soundscapeInterval: number | null = null;
let soundscapeNodes: AudioNode[] = [];

export function startSoundscape(mode: "overworld" | "cave" | "end" = "overworld") {
	try {
		stopSoundscape();
		const ctx = getAudioContext();
		if (!ctx) return;

		const s = settings.value as { soundscapeVolume?: number; soundEnabled?: boolean };
		if (s?.soundEnabled === false) return;
		const vol = typeof s?.soundscapeVolume === "number" ? Math.max(0, Math.min(1, s.soundscapeVolume)) : 0.25;

		const master = ctx.createGain();
		master.gain.setValueAtTime(0, ctx.currentTime);
		master.gain.linearRampToValueAtTime(vol * 0.4, ctx.currentTime + 2.0);
		master.connect(ctx.destination);
		soundscapeMasterGain = master;

		const freqs = mode === "cave" 
			? [65.41, 98.0, 130.81, 146.83]
			: mode === "end"
			? [55.0, 110.0, 164.81, 220.0]
			: [130.81, 164.81, 196.0, 246.94];

		freqs.forEach((freq, idx) => {
			const osc = ctx.createOscillator();
			const gain = ctx.createGain();
			const filter = ctx.createBiquadFilter();

			osc.type = mode === "cave" ? "triangle" : "sine";
			osc.frequency.setValueAtTime(freq, ctx.currentTime);

			filter.type = "lowpass";
			filter.frequency.setValueAtTime(mode === "cave" ? 300 : 800, ctx.currentTime);

			gain.gain.setValueAtTime(0.08 / (idx + 1), ctx.currentTime);

			osc.connect(filter);
			filter.connect(gain);
			gain.connect(master);
			osc.start();

			soundscapeNodes.push(osc, gain, filter);
		});

		soundscapeInterval = window.setInterval(() => {
			if (!soundscapeMasterGain) return;
			try {
				const now = ctx.currentTime;
				const chimeOsc = ctx.createOscillator();
				const chimeGain = ctx.createGain();
				const chimeFilter = ctx.createBiquadFilter();

				const scale = mode === "cave"
					? [130.81, 146.83, 164.81, 196.0, 220.0]
					: mode === "end"
					? [220.0, 246.94, 277.18, 329.63, 440.0]
					: [261.63, 293.66, 329.63, 392.0, 440.0, 523.25];

				const note = scale[Math.floor(Math.random() * scale.length)];
				chimeOsc.type = "sine";
				chimeOsc.frequency.setValueAtTime(note, now);

				chimeFilter.type = "bandpass";
				chimeFilter.frequency.setValueAtTime(note, now);
				chimeFilter.Q.setValueAtTime(3.0, now);

				chimeGain.gain.setValueAtTime(0, now);
				chimeGain.gain.linearRampToValueAtTime(0.05, now + 0.4);
				chimeGain.gain.exponentialRampToValueAtTime(0.0001, now + 3.5);

				chimeOsc.connect(chimeFilter);
				chimeFilter.connect(chimeGain);
				chimeGain.connect(soundscapeMasterGain);

				chimeOsc.start(now);
				chimeOsc.stop(now + 3.6);
				chimeOsc.onended = () => {
					try {
						chimeOsc.disconnect();
						chimeFilter.disconnect();
						chimeGain.disconnect();
					} catch {}
				};
			} catch {}
		}, 4500);

	} catch {}
}

export function stopSoundscape() {
	try {
		if (soundscapeInterval) {
			clearInterval(soundscapeInterval);
			soundscapeInterval = null;
		}
		if (soundscapeMasterGain) {
			soundscapeMasterGain.gain.linearRampToValueAtTime(0, (audioCtx?.currentTime || 0) + 1.0);
			setTimeout(() => {
				soundscapeNodes.forEach((node) => {
					try {
						if ("stop" in node && typeof (node as any).stop === "function") {
							(node as any).stop();
						}
						node.disconnect();
					} catch {}
				});
				soundscapeNodes = [];
				soundscapeMasterGain?.disconnect();
				soundscapeMasterGain = null;
			}, 1100);
		}
	} catch {}
}

export function setSoundscapeVolume(volume: number) {
	if (soundscapeMasterGain && audioCtx) {
		const vol = Math.max(0, Math.min(1, volume));
		soundscapeMasterGain.gain.linearRampToValueAtTime(vol * 0.4, audioCtx.currentTime + 0.1);
	}
}

export function isSoundscapeActive(): boolean {
	return soundscapeMasterGain !== null;
}

