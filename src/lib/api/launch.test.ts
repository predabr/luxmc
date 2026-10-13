import { beforeEach, expect, it, vi } from 'vitest';
const mocks = vi.hoisted(() => ({ invoke: vi.fn(), state: { gameSessionRevision: 0, activeLaunchSessionId: null as string | null, lastLaunchError: '', wasManuallyTerminated: false } }));
vi.mock('./client', () => ({ api: { invoke: mocks.invoke } }));
vi.mock('$lib/stores/app.svelte', () => ({ appState: mocks.state }));
import { cancelLaunch, launchGame, sanitizeLaunchDiagnostic } from './launch';
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

it('keeps a durable preparation failure and resets it for a new attempt', async () => {
    mocks.invoke.mockRejectedValueOnce(new Error('Java incompatível'));
    await expect(launchGame(request)).rejects.toThrow('Java incompatível');
    expect(mocks.state.lastLaunchError).toContain('Java incompatível');
    mocks.invoke.mockResolvedValueOnce({ pid: 42, sessionId: 'new-session' });
    await launchGame(request);
    expect(mocks.state.lastLaunchError).toBe('');
    expect(mocks.state.activeLaunchSessionId).toBe('new-session');
});

it('cancels only the explicit backend session', async () => {
    mocks.invoke.mockResolvedValue(true);
    await expect(cancelLaunch('current-session')).resolves.toBe(true);
    expect(mocks.invoke).toHaveBeenCalledWith('cancel_launch', { sessionId: 'current-session' });
});

it('redacts credentials in command errors and copied diagnostics', async () => {
    const failure = 'Authorization: Bearer private-access refresh_token="private-refresh" password:private-password eyJheader.payload.signature';
    mocks.invoke.mockRejectedValueOnce(new Error(failure));
    await expect(launchGame(request)).rejects.toThrow('[redacted]');
    for (const secret of ['private-access', 'private-refresh', 'private-password', 'eyJheader']) {
        expect(mocks.state.lastLaunchError).not.toContain(secret);
        expect(sanitizeLaunchDiagnostic(failure)).not.toContain(secret);
    }
    expect(sanitizeLaunchDiagnostic('x'.repeat(5000))).toHaveLength(4096);
});
