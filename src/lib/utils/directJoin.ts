import { launchGame } from "$lib/api/launch";
import { profiles } from "$lib/stores/profiles.svelte";
import { account } from "$lib/stores/account.svelte";
import { appState } from "$lib/stores/app.svelte";
import { gamingStats } from "$lib/stores/gamingStats.svelte";
import type { Friend } from "$lib/api/social";

import { parseJoinAddress } from "./joinAddress";
export { parseJoinAddress } from "./joinAddress";

let joining = false;
export async function joinWorld(address: string, friend?: Friend, profileId?: string) {
	if (joining || appState.isGameRunning) throw new Error("O Minecraft já está iniciando ou em execução.");
	const target = parseJoinAddress(address);
	const profile = friend?.mcVersion
		? profiles.list.find(value => value.mcVersion === friend.mcVersion && (!friend.loader || value.loader.toLowerCase() === friend.loader.toLowerCase()))
		: profiles.list.find(value => value.id === profileId) || profiles.active || profiles.list[0];
	if (!profile) throw new Error(friend?.mcVersion ? `Crie uma instância ${friend.loader || "Minecraft"} ${friend.mcVersion} para entrar neste mundo.` : "Selecione uma instância primeiro.");
	const current = account.value;
	if (!current) throw new Error("Selecione sua conta antes de jogar.");
	joining = true;
	try {
		await launchGame({ versionId: profile.mcVersion, profileId: profile.id, accountId: current.uuid || current.id, serverIp: target.host, serverPort: target.port, enableVulkan: profile.useVulkan, skinUrl: current.skinUrl, skinVariant: current.skinVariant, capeUrl: current.capeUrl });
		appState.activeGameDetails = { name: profile.name, version: profile.mcVersion, loader: profile.loader, profileId: profile.id };
		appState.isGameRunning = true;
		gamingStats.onGameStart();
	} finally { joining = false; }
}
