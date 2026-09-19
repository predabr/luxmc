import { socialRegister, socialSync, socialSearch, socialFriendAction, type Friend, type SocialIdentity } from "$lib/api/social";
import { account } from "./account.svelte";
import { appState } from "./app.svelte";

let list = $state<Friend[]>([]);
let me = $state<SocialIdentity | null>(null);
let error = $state("");
let busy = $state(false);
let sharedWorld = $state<{ host: string; port: number } | null>(null);
let favourites = $state<string[]>([]);
let identity = "";
let generation = 0;
let timer: ReturnType<typeof setTimeout> | undefined;

async function refresh(run = generation) {
	if (!identity || run !== generation) return;
	try {
		const game = appState.activeGameDetails;
		const result = await socialSync(identity, {
			status: appState.isGameRunning ? "in_game" : "online",
			instanceName: game?.name, mcVersion: game?.version, loader: game?.loader,
			serverHost: appState.isGameRunning ? sharedWorld?.host : undefined,
			serverPort: appState.isGameRunning ? sharedWorld?.port : undefined
		});
		if (run !== generation) return;
		list = result.friends;
		error = "";
	} catch (cause) {
		if (run !== generation) return;
		error = String(cause);
		list = list.map(friend => friend.status === "pending" ? friend : { ...friend, status: "offline", serverIp: null, serverPort: null });
	}
}

async function poll(run: number) {
	await refresh(run);
	if (run === generation) timer = setTimeout(() => void poll(run), 25000);
}

export const friendsState = {
	refresh,
	get list() { return list; },
	get me() { return me; },
	get error() { return error; },
	get busy() { return busy; },
	get favourites() { return favourites; },
	async connect() {
		const current = account.value;
		if (!current || busy) return;
		this.disconnect();
		const run = generation;
		busy = true;
		identity = current.id.startsWith("luxmc:") ? current.id : current.uuid || current.id;
		try {
			const result = await socialRegister(identity, current.username);
			if (run !== generation) return;
			me = result;
			try { const saved: unknown = JSON.parse(localStorage.getItem(`luxmc_social_favourites_${identity}`) || "[]"); favourites = Array.isArray(saved) ? saved.filter((id): id is string => typeof id === "string") : []; } catch { favourites = []; }
			await poll(run);
		} catch (cause) { if (run === generation) error = String(cause); }
		finally { if (run === generation) busy = false; }
	},
	disconnect() {
		generation += 1;
		clearTimeout(timer);
		identity = "";
		list = [];
		me = null;
		sharedWorld = null;
		busy = false;
		error = "";
	},
	async search(query: string) {
		if (!me) throw new Error("Conecte-se à rede social primeiro.");
		return socialSearch(identity, query);
	},
	async action(action: "invite" | "accept" | "remove", targetId: string) {
		if (!me) throw new Error("Conecte-se à rede social primeiro.");
		const run = generation;
		await socialFriendAction(identity, action, targetId);
		await refresh(run);
	},
	async shareWorld(world: { host: string; port: number } | null) { sharedWorld = world; await refresh(); },
	toggleFavourite(id: string) {
		favourites = favourites.includes(id) ? favourites.filter(value => value !== id) : [...favourites, id];
		localStorage.setItem(`luxmc_social_favourites_${identity}`, JSON.stringify(favourites));
	}
};
