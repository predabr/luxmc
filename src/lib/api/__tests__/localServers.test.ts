import { describe, expect, it, vi } from 'vitest';
const mock = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock('../client', () => ({ api: mock }));
vi.mock('../tunnel', () => ({ roomIdentity: vi.fn() }));
vi.mock('$lib/stores/account.svelte', () => ({ account: { value: { id:'owner' } } }));

describe('server management IPC', () => {
    it('sends cloneable snapshots rather than reactive proxies and preserves nested settings', async () => {
        const { serverManage } = await import('../localServers');
        mock.invoke.mockImplementation(async (_command, args) => structuredClone(args));
        const settings = new Proxy({ seed:'my-world', gamemode:'creative', pvp:false }, {});
        const result = await serverManage<{request:{settings:typeof settings};accountId:string}>({action:'save_settings',id:'server',settings});
        expect(result.request.settings).toEqual({seed:'my-world',gamemode:'creative',pvp:false});
        expect(result.accountId).toBe('owner');
        settings.seed = 'later';
        expect(result.request.settings.seed).toBe('my-world');
    });
});
