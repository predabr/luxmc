import { beforeEach, expect, it, vi } from 'vitest';
const mocks = vi.hoisted(() => ({ invoke: vi.fn(), state: { gameSessionRevision: 0 } }));
vi.mock('./client', () => ({ api: { invoke: mocks.invoke } }));
vi.mock('$lib/stores/app.svelte', () => ({ appState: mocks.state }));
import { launchGame } from './launch';
const request = { versionId: '1.20.1', accountId: 'account', profileId: 'pack' };
beforeEach(() => { vi.clearAllMocks(); mocks.state.gameSessionRevision = 0; });
it('rejects a successful command response if game-exit arrived first', async () => {
    mocks.invoke.mockImplementation(async () => { mocks.state.gameSessionRevision++; return { pid: 42 }; });
    await expect(launchGame(request)).rejects.toThrow('encerrou durante o início');
});
it('accepts a live session and rejects a missing process', async () => {
    mocks.invoke.mockResolvedValue({ pid: 42 });
    await expect(launchGame(request)).resolves.toEqual({ pid: 42 });
    mocks.invoke.mockResolvedValue({ pid: 0 });
    await expect(launchGame(request)).rejects.toThrow();
});
