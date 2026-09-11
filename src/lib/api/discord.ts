import { api } from "./client";
import type { DiscordActivityOptions } from "./types";

export async function discordSetActivity(
	detailsOrOptions?: string | DiscordActivityOptions,
	state?: string,
	largeText?: string,
	largeImage?: string
): Promise<boolean> {
	if (typeof detailsOrOptions === "object" && detailsOrOptions !== null) {
		return api.invoke<boolean>("discord_set_activity", {
			details: detailsOrOptions.details,
			state: detailsOrOptions.state,
			largeText: detailsOrOptions.largeText,
			largeImage: detailsOrOptions.largeImage,
			smallText: detailsOrOptions.smallText,
			smallImage: detailsOrOptions.smallImage,
			startTime: detailsOrOptions.startTime,
			inGame: detailsOrOptions.inGame,
			clientId: detailsOrOptions.clientId
		});
	}
	return api.invoke<boolean>("discord_set_activity", { details: detailsOrOptions, state, largeText, largeImage });
}

export async function discordClearActivity(): Promise<void> {
	return api.invoke("discord_clear_activity");
}
