import { Howl } from "howler";

const sounds: Record<string, Howl> = {};

function getSound(name: string): Howl | null {
	if (typeof window === "undefined") return null;
	if (!sounds[name]) {
		try {
			sounds[name] = new Howl({ src: ["/sounds/${name}.mp3"], volume: 0.3 });
		} catch {
			return null;
		}
	}
	return sounds[name];
}

export function playClick() {
	getSound("click")?.play();
}

export function playSuccess() {
	getSound("success")?.play();
}

export function playError() {
	getSound("error")?.play();
}

export function playHover() {
	getSound("hover")?.play();
}

export function playLaunch() {
	getSound("launch")?.play();
}
