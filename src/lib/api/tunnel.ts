import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { api } from "./client";
import { settings } from "$lib/stores/settings.svelte";
import { activeSkinStore } from "$lib/stores/skin.svelte";
import { account } from "$lib/stores/account.svelte";
export interface TunnelMember { id: string; username: string; uuid: string; avatarUrl: string | null; joinedAt: number; isHost: boolean }
export interface TunnelStatus { mode: "host" | "client"; invitation: string | null; localAddress: string | null; expiresAt: number; pingMs: number | null; transport: string; roomCode: string | null; members: TunnelMember[]; maxPlayers: number; roomLocked: boolean }
const identity = () => ({ username: account.value?.username || "Jogador", uuid: account.value?.uuid || "", avatarUrl: settings.value.shareCustomAvatar === false ? null : activeSkinStore.current.avatarUrl || account.value?.avatarUrl || null });
export const hostWorld = (port: number): Promise<TunnelStatus> => bounded(api.invoke("host_world", { port, identity: identity() }), 45000, uiText("ui.7a713a94a9154c7a"));
function bounded<T>(request: Promise<T>, milliseconds: number, message: string): Promise<T> {
    return new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error(message)), milliseconds);
        request.then(value => { clearTimeout(timer); resolve(value); }, error => { clearTimeout(timer); reject(error); });
    });
}
export const joinTunnel = (invitation: string): Promise<TunnelStatus> => bounded(api.invoke("join_world", { invitation, identity: identity() }), 45000, uiText("ui.99e9b01d898b2847"));
export const stopSession = (): Promise<void> => api.invoke("stop_session");
export const tunnelStatus = (): Promise<TunnelStatus | null> => bounded(api.invoke("tunnel_status"), 6000, uiText("ui.3bced0680504d8ed"));
export const saveTunnelServer = (profileId: string): Promise<string> => bounded(api.invoke("tunnel_save_server", { profileId }), 10000, uiText("ui.5d4a43a900d19dbf"));
export const kickTunnelMember = (memberId: string): Promise<void> => api.invoke("tunnel_kick_member", { memberId });
export const setTunnelLocked = (locked: boolean): Promise<void> => api.invoke("tunnel_set_locked", { locked });
export const refreshTunnelInvitation = (): Promise<TunnelStatus> => api.invoke("tunnel_refresh_invitation");
