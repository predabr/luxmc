import { api } from './client';
import type { TunnelStatus } from './tunnel';
import { translateUi } from '$lib/i18n/useTranslation.svelte';

type SharedAction = {action:'stop_sharing'} | {action:'sharing_kick';memberId:string} | {action:'sharing_lock';locked:boolean} | {action:'sharing_invitation'};

async function statusRequest(command:string,args?:Record<string,unknown>):Promise<TunnelStatus|null> {
    let timer:ReturnType<typeof setTimeout>|undefined;
    try {return await Promise.race([api.invoke<TunnelStatus|null>(command,args),new Promise<never>((_,reject)=>{timer=setTimeout(()=>reject(new Error(translateUi('dedicatedP2P.checkFailed'))),6000);})]);}
    finally {if(timer)clearTimeout(timer);}
}

export async function roomSessionStatus(): Promise<TunnelStatus|null> {
    const local = await statusRequest('tunnel_status');
    return local ?? await statusRequest('local_server_manage',{request:{action:'sharing_status'}});
}

export async function ensureRoomAvailable(): Promise<void> {
    if (await roomSessionStatus()) throw new Error(translateUi('dedicatedP2P.finishCurrent'));
}

export async function roomSessionAction<T>(command:string,args:Record<string,unknown>,request:SharedAction):Promise<T> {
    const local = await statusRequest('tunnel_status');
    if (local) return api.invoke<T>(command,args);
    const shared = await statusRequest('local_server_manage',{request:{action:'sharing_status'}});
    if (shared?.mode === 'host' && shared.hostedServer) return api.invoke<T>('local_server_manage',{request});
    return api.invoke<T>(command,args);
}
