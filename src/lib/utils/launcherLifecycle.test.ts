import { beforeEach, afterEach, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    settings: { launcherActionOnLaunch: 'hide_reopen', showLogsOnLaunch: 'never' },
    game: { isGameRunning: true, gameSessionRevision: 0, activeLaunchSessionId: null as string | null },
    status: vi.fn(), hide: vi.fn(), show: vi.fn(), close: vi.fn(), goto: vi.fn()
}));
vi.mock('$lib/stores/settings.svelte', () => ({ settings: { value: mocks.settings } }));
vi.mock('$lib/stores/app.svelte', () => ({ appState: mocks.game }));
vi.mock('$lib/api/launch', () => ({ gameLaunchStatus: mocks.status }));
vi.mock('$app/navigation', () => ({ goto: mocks.goto }));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => ({ hide: mocks.hide, show: mocks.show, close: mocks.close }) }));
import { handlePostLaunchActions } from './launcherLifecycle';

beforeEach(() => {
    vi.clearAllMocks(); vi.useFakeTimers();
    mocks.settings.launcherActionOnLaunch = 'hide_reopen';
    mocks.settings.showLogsOnLaunch = 'never';
    mocks.game.isGameRunning = true; mocks.game.gameSessionRevision = 0; mocks.game.activeLaunchSessionId = null;
    mocks.status.mockResolvedValue({ running: true, windowReady: true });
    mocks.hide.mockResolvedValue(undefined); mocks.show.mockResolvedValue(undefined);
});
afterEach(() => vi.useRealTimers());

it('keeps the launcher visible until Minecraft has a visible window', async () => {
    mocks.status.mockResolvedValueOnce({ running: true, windowReady: false });
    const action = handlePostLaunchActions(42);
    await vi.advanceTimersByTimeAsync(900);
    expect(mocks.hide).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(100);
    await action;
    expect(mocks.hide).toHaveBeenCalledOnce();
    expect(mocks.status).toHaveBeenLastCalledWith(42, undefined);
});

it('does not hide after an exit that races with window detection', async () => {
    mocks.status.mockImplementation(async () => {
        mocks.game.gameSessionRevision++; mocks.game.isGameRunning = false;
        return { running: true, windowReady: true };
    });
    await handlePostLaunchActions(42);
    expect(mocks.hide).not.toHaveBeenCalled();
});

it('does not destroy the launcher window or the LAN monitor when close is selected', async () => {
    mocks.settings.launcherActionOnLaunch = 'close';
    await handlePostLaunchActions(42);
    expect(mocks.hide).toHaveBeenCalledOnce();
    expect(mocks.close).not.toHaveBeenCalled();
});

it('restores visibility if the game exits while hiding the launcher', async () => {
    mocks.hide.mockImplementation(async () => { mocks.game.isGameRunning = false; mocks.game.gameSessionRevision++; });
    await handlePostLaunchActions(42);
    expect(mocks.show).toHaveBeenCalledOnce();
});

it('keeps the launcher visible if status fails or Minecraft exits without a window', async () => {
    mocks.status.mockResolvedValue({ running: false, windowReady: false });
    await handlePostLaunchActions(42);
    mocks.status.mockRejectedValue(new Error('IPC failed'));
    await handlePostLaunchActions(43);
    expect(mocks.hide).not.toHaveBeenCalled();
});

it('requires readiness again immediately before hiding and passes the session identity', async () => {
    mocks.game.activeLaunchSessionId = 'session-42';
    mocks.status.mockResolvedValueOnce({ running: true, windowReady: true })
        .mockResolvedValueOnce({ running: true, windowReady: false });
    await handlePostLaunchActions(42);
    expect(mocks.status).toHaveBeenLastCalledWith(42, 'session-42');
    expect(mocks.hide).not.toHaveBeenCalled();
});

it('does not leave an endless monitor when a modpack never opens a window', async () => {
    mocks.status.mockResolvedValue({ running: true, windowReady: false });
    const action = handlePostLaunchActions(42);
    await vi.advanceTimersByTimeAsync(180_001);
    await action;
    expect(mocks.hide).not.toHaveBeenCalled();
    expect(vi.getTimerCount()).toBe(0);
});
