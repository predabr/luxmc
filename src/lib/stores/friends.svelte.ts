import { toast } from "./toasts.svelte";
import { joinWorld } from "$lib/utils/directJoin";
import { socialStreamTicket, friendsSnapshotSchema, socialCreateRoom, socialJoinRoom, socialCloseRoom, socialRegister, socialSync, socialSearch, socialFriendAction, type Friend, type SocialIdentity } from "$lib/api/social";
import { account } from "./account.svelte";
import { appState } from "./app.svelte";

let list = $state<Friend[]>([]);
let me = $state<SocialIdentity | null>(null);
let error = $state("");
let busy = $state(false);
let sharedWorld = $state<{ host: string; port: number } | null>(null);
let favourites = $state<string[]>([]);
let socket: WebSocket | null = null;
let reconnect: ReturnType<typeof setTimeout> | undefined;
let identity = "";
let generation = 0;
let timer: ReturnType<typeof setTimeout> | undefined;

function applyFriends(next: Friend[]) {
        for (const friend of next) {
            const previous = list.find(item => item.id === friend.id);
            if (previous && friend.serverIp && friend.serverPort && (friend.serverIp !== previous.serverIp || friend.serverPort !== previous.serverPort)) {
                toast(`${friend.username} abriu ${friend.activity || "um mundo LAN"}.`, "info", {
                    label: "Entrar agora", run: () => { void joinWorld(`${friend.serverIp}:${friend.serverPort}`, friend).catch(cause => toast(String(cause), "error")); }
                });
            }
        }
		list = next;
}

async function connectStream(run: number, attempt = 0) {
    if (run !== generation || !identity) return;
    try {
        const url = await socialStreamTicket(identity);
        if (!url || run !== generation) return;
        const connection = new WebSocket(url);
        socket = connection;
        connection.onmessage = event => {
            if (run !== generation || typeof event.data !== "string" || event.data.length > 262144) return;
            try { applyFriends(friendsSnapshotSchema.parse(JSON.parse(event.data)).friends); error = ""; } catch { connection.close(1008, "Invalid snapshot"); }
        };
        connection.onopen = () => { attempt = 0; };
        connection.onclose = () => {
            if (socket === connection) socket = null;
            if (run === generation) reconnect = setTimeout(() => void connectStream(run, attempt + 1), Math.min(60000, 2000 * 2 ** Math.min(attempt, 5)));
        };
    } catch { if (run === generation) reconnect = setTimeout(() => void connectStream(run, attempt + 1), 60000); }
}

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
        applyFriends(result.friends);
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
    createRoom(host: string, port: number) { return socialCreateRoom(identity, host, port); },
    joinRoom(code: string) { return socialJoinRoom(identity, code); },
    closeRoom() { return socialCloseRoom(identity); },
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
			void connectStream(run);
			await poll(run);
		} catch (cause) { if (run === generation) error = String(cause); }
		finally { if (run === generation) busy = false; }
	},
	disconnect() {
		generation += 1;
        clearTimeout(reconnect);
        if (socket) { socket.onclose = null; socket.onmessage = null; socket.close(); socket = null; }
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
	async action(action: "invite" | "accept" | "remove" | "block" | "unblock", targetId: string) {
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
