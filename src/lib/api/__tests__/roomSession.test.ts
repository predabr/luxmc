import { beforeEach, describe, expect, it, vi } from 'vitest';
const mock=vi.hoisted(()=>({invoke:vi.fn()}));
vi.mock('../client',()=>({api:mock}));
vi.mock('$lib/i18n/useTranslation.svelte',()=>({translateUi:(key:string)=>key}));
import { ensureRoomAvailable, roomSessionAction, roomSessionStatus } from '../roomSession';

describe('shared LAN and dedicated room controls',()=>{
    beforeEach(()=>{mock.invoke.mockReset();});
    it('keeps an existing LAN session in its owning process',async()=>{
        const lan={mode:'client'};mock.invoke.mockResolvedValue(lan);
        expect(await roomSessionStatus()).toBe(lan);
        expect(mock.invoke).toHaveBeenCalledTimes(1);
        await roomSessionAction('tunnel_set_locked',{locked:true},{action:'sharing_lock',locked:true});
        expect(mock.invoke).toHaveBeenLastCalledWith('tunnel_set_locked',{locked:true});
    });
    it('finds the dedicated room in the worker instead of showing an empty LAN screen',async()=>{
        const shared={mode:'host',hostedServer:{name:'Our server'}};
        mock.invoke.mockImplementation(async(command)=>command==='tunnel_status'?null:shared);
        expect(await roomSessionStatus()).toBe(shared);
        await expect(ensureRoomAvailable()).rejects.toThrow('dedicatedP2P.finishCurrent');
    });
    it.each([
        ['stop_session',{}, {action:'stop_sharing'}],
        ['tunnel_set_locked',{locked:true},{action:'sharing_lock',locked:true}],
        ['tunnel_kick_member',{memberId:'peer'},{action:'sharing_kick',memberId:'peer'}],
        ['tunnel_refresh_invitation',{}, {action:'sharing_invitation'}],
    ] as const)('routes %s to the process hosting the server room',async(command,args,request)=>{
        mock.invoke.mockImplementation(async(command,args)=>command==='tunnel_status'?null:args.request.action==='sharing_status'?{mode:'host',hostedServer:{name:'Our server'}}:null);
        await roomSessionAction(command,args,request);
        expect(mock.invoke).toHaveBeenLastCalledWith('local_server_manage',{request});
        expect(mock.invoke).not.toHaveBeenCalledWith('local_server_stop',expect.anything());
    });
    it('does not hide worker errors and accidentally allow a second room',async()=>{
        mock.invoke.mockImplementation(async(command)=>{if(command==='tunnel_status')return null;throw new Error('worker unavailable');});
        await expect(ensureRoomAvailable()).rejects.toThrow('worker unavailable');
    });
});
