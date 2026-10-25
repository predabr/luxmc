import { api } from "./client";
import type { DiscordActivityOptions } from "./types";
import { settings } from "$lib/stores/settings.svelte";

let revision = 0;
let queue: Promise<unknown> = Promise.resolve();
function enqueue<T>(action: () => Promise<T>): Promise<T> {
    const result = queue.then(action, action);
    queue = result.catch(() => {});
    return result;
}

export async function discordSetActivity(
	detailsOrOptions?: string | DiscordActivityOptions,
	state?: string,
	largeText?: string,
	largeImage?: string
): Promise<boolean> {
	const current = ++revision;
	if (settings.value.discordRpc === false) return false;
	if (typeof detailsOrOptions === "object" && detailsOrOptions !== null) {
		return enqueue(() => current !== revision || settings.value.discordRpc === false ? Promise.resolve(false) : api.invoke<boolean>("discord_set_activity", {
			details: detailsOrOptions.details,
			state: detailsOrOptions.state,
			largeText: detailsOrOptions.largeText,
			largeImage: detailsOrOptions.largeImage,
			smallText: detailsOrOptions.smallText,
			smallImage: detailsOrOptions.smallImage,
			startTime: detailsOrOptions.startTime,
			inGame: detailsOrOptions.inGame,
			clientId: detailsOrOptions.clientId,
			buttons: detailsOrOptions.buttons
		}));
	}
	return enqueue(() => current !== revision || settings.value.discordRpc === false ? Promise.resolve(false) : api.invoke<boolean>("discord_set_activity", { details: detailsOrOptions, state, largeText, largeImage }));
}

export async function discordClearActivity(): Promise<void> {
	revision++;
	return enqueue(() => api.invoke("discord_clear_activity"));
}
