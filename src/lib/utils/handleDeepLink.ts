import { goto } from "$app/navigation";
import { deepLinks } from "$lib/stores/deepLinks.svelte";
import { friendsState } from "$lib/stores/friends.svelte";
import { joinWorld } from "./directJoin";
import { parseDeepLink } from "./deepLink";

export async function handleDeepLink(value: string): Promise<void> {
    const action = parseDeepLink(value);
    if (action.kind === "install") {
        deepLinks.install = action;
        await goto("/mods");
    } else if (action.kind === "skin") {
        deepLinks.skin = action;
        await goto("/skins");
    } else if (action.kind === "server") {
        await joinWorld(action.address);
    } else {
        if (!friendsState.me && !friendsState.busy) await friendsState.connect();
        const deadline = Date.now() + 10000;
        while (friendsState.busy && Date.now() < deadline) await new Promise<void>(resolve => setTimeout(resolve, 100));
        await friendsState.refresh();
        const friend = friendsState.list.find(value => value.id === action.code);
        if (friend?.status === "in_game" && friend.serverIp && friend.serverPort) {
            const host = friend.serverIp.includes(":") && !friend.serverIp.startsWith("[") ? `[${friend.serverIp}]` : friend.serverIp;
            await joinWorld(`${host}:${friend.serverPort}`, friend);
        } else if (action.code.includes(":")) {
            await joinWorld(action.code);
        } else {
            await goto("/friends");
            throw new Error("Este amigo não tem um mundo compartilhado disponível. Atualize a conexão na aba Amigos.");
        }
    }
}
