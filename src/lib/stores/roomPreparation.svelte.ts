import { roomPrepare, roomRecipe, type RoomRecipe } from '$lib/api/studio';
import { listen } from '$lib/api/client';
import { profiles } from './profiles.svelte';
import { modpackInstallation } from './modpackInstallation.svelte';
import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
const state=$state({busy:false,percent:0,name:'',error:'',profileId:'',ownerId:'',text:''});
async function prepare(ownerId:string) {
    if(state.busy || modpackInstallation.state.busy)throw new Error(t('studio.installBusy'));
    Object.assign(state,{busy:true,percent:0,name:'',error:'',profileId:'',ownerId,text:t('studio.phase.planning')});
    const cleanup:(()=>void)[]=[];
    try {
        cleanup.push(await listen<{percent:number;name:string;profileId:string}>('room-preparation-progress',event=>{state.percent=event.payload.percent;state.name=event.payload.name;state.profileId=event.payload.profileId;state.text=t('studio.phase.downloading');}));
        cleanup.push(await listen<{percent?:number;status:string}>('modpack-progress',event=>{state.percent=Math.min(85,event.payload.percent||0);state.text=event.payload.status;}));
        const profile=await roomPrepare(ownerId);state.profileId=profile.id;state.name=profile.name;state.percent=100;state.text=t('studio.phase.ready');await profiles.refresh();
    }catch(cause){state.error=String(cause);state.text=t('studio.phase.failed');}finally{cleanup.forEach(stop=>stop());state.busy=false;}
}
export const roomPreparation={get state(){return state;},prepare,recipe:(id:string):Promise<RoomRecipe>=>roomRecipe(id),dismiss(){if(!state.busy)Object.assign(state,{ownerId:'',error:'',profileId:''});}};
