import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { goto } from "$app/navigation";
import { toast } from "./toasts.svelte";
import { joinWorld } from "$lib/utils/directJoin";
import { socialStreamTicket, friendsSnapshotSchema, socialCreateRoom, socialJoinRoom, socialCloseRoom, socialRegister, socialSync, socialSearch, socialFriendAction, type Friend, type SocialIdentity } from "$lib/api/social";
import { settings } from "./settings.svelte";
import { activeSkinStore } from "./skin.svelte";
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
let wake: (() => void) | undefined;
let refreshing: { run: number; promise: Promise<void> } | null = null;

function applyFriends(next: Friend[]) {
        for (const friend of next) {
            const previous = list.find(item => item.id === friend.id);
            if (friend.status === "pending" && friend.incoming && (!previous || !previous.incoming)) {
                toast(uiText("ui.d12511055139d26b", {arg0: (friend.username)}), "info", {
                    label: uiText("ui.768ddf45541f26d2"), run: () => { void goto("/friends?tab=pending"); }
                });
            }
            if (previous && friend.serverIp && friend.serverPort && (friend.serverIp !== previous.serverIp || friend.serverPort !== previous.serverPort)) {
                toast(`${friend.username} abriu ${friend.activity || uiText("ui.e7195a83674adfd2")}.`, "info", {
                    label: uiText("ui.42f2d3f3d3821670"), run: () => { void joinWorld(`${friend.serverIp}:${friend.serverPort}`, friend).catch(cause => toast(String(cause), "error")); }
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
        connection.onopen = () => { attempt = 0; void refresh(run); };
        connection.onclose = () => {
            if (socket === connection) socket = null;
            if (run === generation) reconnect = setTimeout(() => void connectStream(run, attempt + 1), Math.min(60000, 2000 * 2 ** Math.min(attempt, 5)));
        };
    } catch { if (run === generation) reconnect = setTimeout(() => void connectStream(run, attempt + 1), 60000); }
}

async function refreshNow(run: number) {
	if (!identity || run !== generation) return;
	try {
		const game = appState.activeGameDetails;
		const result = await socialSync(identity, {
			status: appState.isGameRunning && settings.value.publishGameActivity !== false ? "in_game" : "online",
            avatarUrl: settings.value.shareCustomAvatar === false ? "" : activeSkinStore.current.avatarUrl || account.value?.avatarUrl || undefined,
			instanceName: settings.value.publishGameActivity === false ? undefined : game?.name, mcVersion: settings.value.publishGameActivity === false ? undefined : game?.version, loader: settings.value.publishGameActivity === false ? undefined : game?.loader,
			serverHost: appState.isGameRunning && settings.value.publishGameActivity !== false ? sharedWorld?.host : undefined,
			serverPort: appState.isGameRunning && settings.value.publishGameActivity !== false ? sharedWorld?.port : undefined
		});
		if (run !== generation) return;
        applyFriends(result.friends);
        me = result.me;
		error = "";
	} catch (cause) {
		if (run !== generation) return;
		error = String(cause);
		list = list.map(friend => friend.status === "pending" ? friend : { ...friend, status: "offline", serverIp: null, serverPort: null });
	}
}

function refresh(run = generation): Promise<void> {
    if (!identity || run !== generation) return Promise.resolve();
    if (refreshing?.run === run) return refreshing.promise;
    const promise = refreshNow(run);
    refreshing = { run, promise };
    void promise.finally(() => { if (refreshing?.promise === promise) refreshing = null; });
    return promise;
}

async function poll(run: number) {
	await refresh(run);
	if (run === generation) { clearTimeout(timer); timer = setTimeout(() => void poll(run), document.hidden ? 45000 : socket?.readyState === WebSocket.OPEN ? 25000 : 5000); }
}

export const friendsState = {
    createRoom(host: string, port: number) { return socialCreateRoom(identity, host, port); },
    joinRoom(code: string) { return socialJoinRoom(identity, code); },
    closeRoom() { return socialCloseRoom(identity); },
	refresh,
	get list() { return list; },
	get me() { return me; },
    get accountId() { return identity; },
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
            wake = () => { if (!document.hidden && run === generation) { clearTimeout(timer); void poll(run); } };
            window.addEventListener("focus", wake);
            document.addEventListener("visibilitychange", wake);
			await poll(run);
		} catch (cause) { if (run === generation) { error = String(cause); timer = setTimeout(() => void friendsState.connect(), 15000); } }
		finally { if (run === generation) busy = false; }
	},
	disconnect() {
		generation += 1;
        if (wake) { window.removeEventListener("focus", wake); document.removeEventListener("visibilitychange", wake); wake = undefined; }
        refreshing = null;
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
		if (!me) throw new Error(uiText("ui.4567cd01684fa7fb"));
		return socialSearch(identity, query);
	},
	async action(action: "invite" | "accept" | "remove" | "block" | "unblock", targetId: string) {
		if (!me) throw new Error(uiText("ui.4567cd01684fa7fb"));
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
