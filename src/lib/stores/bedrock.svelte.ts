import { bedrockState, bedrockInstall, type BedrockState, type BedrockInstallProgress } from "$lib/api/bedrock";
import { listen } from "@tauri-apps/api/event";

let value = $state<BedrockState | null>(null);
let error = $state("");
let request: Promise<void> | null = null;
let installing = $state(false);
let progress = $state<BedrockInstallProgress | null>(null);
let installationName = $state("");

export const bedrock = {
	get value() { return value; },
	get error() { return error; },
	get installing() { return installing; },
	get progress() { return progress; },
	get installationName() { return installationName; },
	async install(versionId: string, name: string): Promise<void> {
		if (installing) throw new Error("Uma instalação Bedrock já está em andamento.");
		installing = true; error = ""; installationName = name; progress = {phase: "preparing", percent: 0, downloaded: 0, total: 0};
		let unlisten: (() => void) | undefined;
		try {
			unlisten = await listen<BedrockInstallProgress>("bedrock-install-progress", event => { progress = event.payload; });
			await bedrockInstall(versionId, name);
			await bedrock.refresh();
		} catch (failure) { error = String(failure); throw failure; }
		finally { unlisten?.(); installing = false; progress = null; }
	},
	refresh(): Promise<void> {
		if (request) return request;
		request = bedrockState().then((next) => { value = next; error = ""; })
			.catch((failure: unknown) => { error = String(failure); })
			.finally(() => { request = null; });
		return request;
	}
};
