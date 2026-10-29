import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { api } from "./client";
import { settings } from "$lib/stores/settings.svelte";
import { activeSkinStore } from "$lib/stores/skin.svelte";
import { friendsState } from "$lib/stores/friends.svelte";
import { account } from "$lib/stores/account.svelte";
import { ensureRoomAvailable, roomSessionStatus, roomSessionAction } from './roomSession';
export interface TunnelMember { displayName?: string | null; id: string; username: string; uuid: string; avatarUrl: string | null; joinedAt: number; isHost: boolean; preparation?:import('./studio').Preparation|null }
export interface RoomWorld { ownerId: string; ownerUsername: string; motd: string; localAddress: string | null; compatibility?: {mcVersion:string;loader:string;modFingerprint:string;modCount:number} | null }
export interface HostedServer { name: string; mcVersion: string; localServerId: string }
export interface TunnelStatus { hostedServer?: HostedServer | null; mode: "host" | "client"; invitation: string | null; localAddress: string | null; expiresAt: number; pingMs: number | null; transport: string; roomCode: string | null; members: TunnelMember[]; maxPlayers: number; roomLocked: boolean; worldReady?: boolean; worlds?: RoomWorld[] }
async function identity() {
    let avatarUrl = settings.value.shareCustomAvatar === false ? null : activeSkinStore.current.avatarUrl || account.value?.avatarUrl || null;
    const portrait = settings.value.shareCustomAvatar === false ? '' : friendsState.ownProfile?.portrait;
    if (portrait) {
        try {
            const image = new Image();
            image.crossOrigin = 'anonymous';
            await bounded(new Promise<void>((resolve,reject) => { image.onload = () => resolve(); image.onerror = reject; image.src = portrait; }),1500,'Avatar indisponível');
            const canvas = document.createElement('canvas');
            for (const size of [48,32,24,16]) {
                canvas.width = size; canvas.height = size;
                canvas.getContext('2d')?.drawImage(image,0,0,size,size);
                const thumbnail = canvas.toDataURL('image/png');
                if (thumbnail.length <= 2048) { avatarUrl = thumbnail; break; }
            }
        } catch {}
    }
    return {username:account.value?.username || 'Jogador',uuid:account.value?.uuid || '',avatarUrl,displayName:friendsState.ownProfile?.displayName || null};
}
export { identity as roomIdentity };
export async function hostWorld(port?:number):Promise<TunnelStatus> {
    await bounded(ensureRoomAvailable(),6000,uiText('dedicatedP2P.checkFailed'));
    return bounded(api.invoke("host_world",{port,identity:await identity(),accountId:account.value?.id}),45000,uiText("ui.7a713a94a9154c7a"));
}
function bounded<T>(request: Promise<T>, milliseconds: number, message: string): Promise<T> {
    return new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error(message)), milliseconds);
        request.then(value => { clearTimeout(timer); resolve(value); }, error => { clearTimeout(timer); reject(error); });
    });
}
let lastInvitation = "";
export const canReconnectTunnel = () => Boolean(lastInvitation);
export async function joinTunnel(invitation: string): Promise<TunnelStatus> {
    await bounded(ensureRoomAvailable(),6000,uiText('dedicatedP2P.checkFailed'));
    const result = await bounded<TunnelStatus>(api.invoke("join_world", { invitation, identity: await identity() }), 45000, uiText("ui.99e9b01d898b2847"));
    lastInvitation = invitation;
    return result;
}
export const reconnectTunnel = (): Promise<TunnelStatus> => joinTunnel(lastInvitation);
export async function stopSession(): Promise<void> { await bounded(roomSessionAction<void>('stop_session',{}, {action:'stop_sharing'}),10000,uiText('dedicatedP2P.checkFailed')); lastInvitation = ""; }
export const tunnelStatus = (): Promise<TunnelStatus | null> => bounded(roomSessionStatus(), 6000, uiText("ui.3bced0680504d8ed"));
export const saveTunnelServer = (profileId: string): Promise<string> => bounded(api.invoke("tunnel_save_server", { profileId }), 10000, uiText("ui.5d4a43a900d19dbf"));
export const kickTunnelMember = (memberId: string): Promise<void> => bounded(roomSessionAction('tunnel_kick_member',{memberId},{action:'sharing_kick',memberId}),10000,uiText('dedicatedP2P.checkFailed'));
export const setTunnelLocked = (locked: boolean): Promise<void> => bounded(roomSessionAction('tunnel_set_locked',{locked},{action:'sharing_lock',locked}),10000,uiText('dedicatedP2P.checkFailed'));
export const refreshTunnelInvitation = (): Promise<TunnelStatus> => bounded(roomSessionAction('tunnel_refresh_invitation',{}, {action:'sharing_invitation'}),20000,uiText('dedicatedP2P.checkFailed'));
