import { api, listen, type UnlistenFn } from "./client";
import type {
	P2PConnectionInfo,
	P2PMessagePayload,
	HostLinkInfo,
} from "./types";

export async function p2pGetLocalInfo(): Promise<P2PConnectionInfo> {
	return api.invoke<P2PConnectionInfo>("p2p_get_local_info");
}

export async function p2pGetHostLink(port?: number): Promise<HostLinkInfo> {
	return api.invoke<HostLinkInfo>("p2p_get_host_link", { port });
}

export async function p2pStartListener(): Promise<boolean> {
	return api.invoke<boolean>("p2p_start_listener");
}

export async function p2pSendMessage(targetAddress: string, sender: string, text: string): Promise<boolean> {
	return api.invoke<boolean>("p2p_send_message", { targetAddress, sender, text });
}

export async function listenP2PMessage(
	callback: (payload: P2PMessagePayload) => void
): Promise<UnlistenFn> {
	return listen<P2PMessagePayload>("p2p-chat-message", (event) => {
		callback(event.payload);
	});
}
