import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { ModpackUpdateInfo } from '$lib/api/types';
const mocks = vi.hoisted(() => ({ check: vi.fn(), diff: vi.fn(), apply: vi.fn(), refresh: vi.fn(), listen: vi.fn(), stop: vi.fn(), queue: vi.fn(), toast: vi.fn(), game: { isGameRunning: false, isLaunching: false }, installation: { busy: false }, progress: null as null | ((event: { payload: { profileId?: string; percent: number; status: string } }) => void) }));
vi.mock('$lib/api/java', () => ({ modpackCheckUpdate: mocks.check, modpackVersionDiff: mocks.diff, modpackUpdateAtomic: mocks.apply }));
vi.mock('$lib/api/client', () => ({ listen: mocks.listen }));
vi.mock('$lib/i18n/useTranslation.svelte', () => ({ translateUi: (key: string) => key }));
vi.mock('./profiles.svelte', () => ({ profiles: { list: [{ id: 'pack-a', name: 'Pack A' }], refresh: mocks.refresh } }));
vi.mock('./app.svelte', () => ({ appState: mocks.game }));
vi.mock('./modpackInstallation.svelte', () => ({ modpackInstallation: { state: mocks.installation, continueQueue: mocks.queue } }));
vi.mock('./toasts.svelte', () => ({ toast: mocks.toast }));
const available: ModpackUpdateInfo = { hasUpdate: true, currentVersion: '1', latestVersion: '2', changelog: null, source: 'modrinth', projectId: 'project', versionId: 'v2' };

beforeEach(() => {
    vi.resetModules(); vi.clearAllMocks(); mocks.game.isGameRunning = false; mocks.game.isLaunching = false; mocks.installation.busy = false;
    mocks.check.mockResolvedValue(available); mocks.diff.mockResolvedValue(null); mocks.refresh.mockResolvedValue(undefined); mocks.apply.mockResolvedValue(undefined);
    mocks.listen.mockImplementation(async (_: string, handler: typeof mocks.progress) => { mocks.progress = handler; return mocks.stop; });
});

describe('persistent modpack updates', () => {
    it('refreshes after installation without an old request deleting the newer pending check', async () => {
        const { modpackUpdates: updates } = await import('./modpackUpdates.svelte');
        let oldFinish!: (value: ModpackUpdateInfo) => void;
        let newFinish!: (value: ModpackUpdateInfo) => void;
        mocks.check.mockImplementationOnce(() => new Promise<ModpackUpdateInfo>(resolve => {oldFinish=resolve;}));
        const old = updates.check('pack-a');
        updates.installed('pack-a','modrinth','project','v2','2');
        mocks.check.mockImplementationOnce(() => new Promise<ModpackUpdateInfo>(resolve => {newFinish=resolve;}));
        const fresh = updates.check('pack-a',true);
        oldFinish(available); await old;
        const shared = updates.check('pack-a',true);
        expect(mocks.check).toHaveBeenCalledTimes(2);
        newFinish({...available,currentVersionId:'v2',versionId:'v2'});
        await Promise.all([fresh,shared]);
        expect(updates.forProfile('pack-a')?.info?.hasUpdate).toBe(false);
    });
    it('deduplicates checks and keeps the full project reference when no update exists', async () => {
        const { modpackUpdates: updates } = await import('./modpackUpdates.svelte');
        mocks.check.mockResolvedValue({ ...available, hasUpdate: false });
        await Promise.all([updates.check('pack-a'), updates.check('pack-a')]);
        expect(mocks.check).toHaveBeenCalledOnce();
        expect(updates.forProfile('pack-a')?.info?.projectId).toBe('project');
    });
    it('checks release metadata without downloading pack archives or scanning dependencies', async () => {
        const { modpackUpdates: updates } = await import('./modpackUpdates.svelte');
        await updates.check('pack-a');
        expect(updates.forProfile('pack-a')?.info?.hasUpdate).toBe(true);
        expect(mocks.diff).not.toHaveBeenCalled();
    });
    it('continues progress outside a page and clears the outdated notice after success', async () => {
        const { modpackUpdates: updates } = await import('./modpackUpdates.svelte');
        await updates.check('pack-a');
        let complete!: () => void;
        mocks.apply.mockImplementation(() => new Promise<void>(resolve => { complete = resolve; }));
        const applying = updates.apply('pack-a', 'v2', 'modrinth', 'project', '2');
        await vi.waitFor(() => expect(mocks.apply).toHaveBeenCalledOnce());
        mocks.progress!({ payload: { profileId: 'other-pack', percent: 90, status: 'Wrong instance' } });
        expect(updates.task?.percent).toBe(2);
        mocks.progress!({ payload: { profileId: 'pack-a', percent: 65, status: 'Downloading mods' } });
        expect(updates.task?.percent).toBe(65);
        expect(updates.forProfile('pack-a')?.info?.hasUpdate).toBe(true);
        await expect(updates.apply('pack-a', 'v2', 'modrinth', 'project')).rejects.toThrow();
        complete(); await applying;
        expect(updates.task).toMatchObject({ busy: false, complete: true, percent: 100 });
        expect(updates.forProfile('pack-a')?.info).toMatchObject({ hasUpdate: false, currentVersion: '2', projectId: 'project' });
        expect(mocks.stop).toHaveBeenCalledOnce(); expect(mocks.queue).toHaveBeenCalledOnce();
    });
    it('ignores a check started before the update completed', async () => {
        const { modpackUpdates: updates } = await import('./modpackUpdates.svelte');
        let finish!: (value: ModpackUpdateInfo) => void;
        mocks.check.mockImplementation(() => new Promise<ModpackUpdateInfo>(resolve => { finish = resolve; }));
        const checking = updates.check('pack-a');
        await updates.apply('pack-a', 'v2', 'modrinth', 'project', '2');
        finish(available); await checking;
        expect(updates.forProfile('pack-a')?.info?.hasUpdate).toBe(false);
    });
    it('keeps the old version available on failure and cleans up the operation', async () => {
        const { modpackUpdates: updates } = await import('./modpackUpdates.svelte');
        await updates.check('pack-a'); mocks.apply.mockRejectedValue(new Error('Download failed'));
        await expect(updates.apply('pack-a', 'v2', 'modrinth', 'project')).rejects.toThrow('Download failed');
        expect(updates.forProfile('pack-a')?.info?.currentVersion).toBe('1');
        expect(updates.task).toMatchObject({ busy: false, complete: false, error: 'Error: Download failed' });
        expect(mocks.stop).toHaveBeenCalledOnce(); expect(mocks.refresh).not.toHaveBeenCalled();
    });
    it('blocks changes while Minecraft is launching', async () => {
        const { modpackUpdates: updates } = await import('./modpackUpdates.svelte'); mocks.game.isLaunching = true;
        await expect(updates.apply('pack-a', 'v2', 'modrinth', 'project')).rejects.toThrow();
        expect(mocks.apply).not.toHaveBeenCalled();
    });
    it('discards an old update result when a freshly installed pack is linked', async () => {
        const { modpackUpdates: updates } = await import('./modpackUpdates.svelte');
        let finish!: (value: ModpackUpdateInfo) => void;
        mocks.check.mockImplementation(() => new Promise<ModpackUpdateInfo>(resolve => { finish = resolve; }));
        const checking = updates.check('pack-a');
        updates.installed('pack-a', 'modrinth', 'project', 'v2', '2');
        finish(available); await checking;
        expect(updates.forProfile('pack-a')?.info).toMatchObject({ hasUpdate: false, currentVersion: '2', versionId: 'v2' });
    });
});
