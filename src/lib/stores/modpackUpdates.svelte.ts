import { modpackCheckUpdate, modpackUpdateAtomic, modpackVersionDiff } from '$lib/api/java';
import type { ModpackUpdateInfo, ModpackVersionDiff } from '$lib/api/types';
import { listen } from '$lib/api/client';
import { profiles } from './profiles.svelte';
import { appState } from './app.svelte';
import { modpackInstallation } from './modpackInstallation.svelte';
import { contentActivity } from './contentActivity.svelte';
import { toast } from './toasts.svelte';
import { translateUi as t } from '$lib/i18n/useTranslation.svelte';

type Entry = { info: ModpackUpdateInfo | null; diff: ModpackVersionDiff | null; checking: boolean; checkedAt: number; revision: number };
type Task = { profileId: string; name: string; version: string; busy: boolean; percent: number; text: string; error: string; complete: boolean };
let entries = $state<Record<string, Entry>>({});
let task = $state<Task | null>(null);
const checks = new Map<string, Promise<void>>();
let libraryCheck: Promise<void> | null = null;
let libraryCheckedAt = 0;

async function checkLibrary(): Promise<void> {
    if (libraryCheck) return libraryCheck;
    if (Date.now() - libraryCheckedAt < 300_000 || appState.isLaunching || appState.isGameRunning || modpackInstallation.state.busy) return;
    libraryCheckedAt = Date.now();
    const pending = profiles.list.map(profile => profile.id);
    libraryCheck = Promise.all(Array.from({length:2},async () => {
        while (pending.length) {
            if (appState.isLaunching || appState.isGameRunning || modpackInstallation.state.busy) { libraryCheckedAt=0; break; }
            const id = pending.shift();
            if (id) await check(id).catch(() => {});
        }
    })).then(() => {}).finally(() => {libraryCheck=null;});
    return libraryCheck;
}

function entryFor(profileId: string): Entry {
    if (!entries[profileId]) entries[profileId] = { info: null, diff: null, checking: false, checkedAt: 0, revision: 0 };
    return entries[profileId];
}

async function check(profileId: string, force = false): Promise<void> {
    if (!profileId || task?.busy && task.profileId === profileId) return;
    const entry = entryFor(profileId);
    if (!force && Date.now() - entry.checkedAt < 120_000) return;
    const pending = checks.get(profileId);
    if (pending) return pending;
    const revision = entry.revision;
    entry.checking = true;
    const request = (async () => {
        try {
            const info = await modpackCheckUpdate(profileId);
            if (entry.revision !== revision) return;
            entry.info = info;
            entry.diff = null;
            entry.checkedAt = Date.now();
            if (info?.hasUpdate && info.projectId && info.versionId) {
                const diff = await modpackVersionDiff(profileId, info.versionId, info.source).catch(() => null);
                if (entry.revision === revision) entry.diff = diff;
            }
        } finally {
            if (entry.revision === revision) entry.checking = false;
            checks.delete(profileId);
        }
    })();
    checks.set(profileId, request);
    return request;
}

async function apply(profileId: string, versionId: string, source: string, projectId: string, version = ''): Promise<void> {
    if (task?.busy || modpackInstallation.state.busy || appState.isGameRunning || appState.isLaunching) {
        toast(t('studio.installBusy'), 'warning');
        throw new Error(t('studio.installBusy'));
    }
    const entry = entryFor(profileId);
    entry.revision += 1;
    entry.checking = false;
    contentActivity.updateProfileId = profileId;
    const operation = $state<Task>({ profileId, name: profiles.list.find(profile => profile.id === profileId)?.name || projectId, version, busy: true, percent: 2, text: t('ui.b35e32e9887ce30f'), error: '', complete: false });
    task = operation;
    let stop: (() => void) | undefined;
    try {
        stop = await listen<{ profileId?: string; percent?: number; status?: string }>('modpack-progress', event => {
            if (!operation.busy || event.payload.profileId && event.payload.profileId !== profileId) return;
            if (event.payload.status) operation.text = event.payload.status;
            if (typeof event.payload.percent === 'number') operation.percent = Math.max(operation.percent, Math.min(99, Math.max(0, event.payload.percent)));
        });
        await modpackUpdateAtomic(profileId, versionId, source, projectId);
        entry.info = { hasUpdate: false, currentVersion: version || versionId, latestVersion: version || versionId, versionId, source, projectId, changelog: null };
        entry.diff = null;
        entry.checkedAt = Date.now();
        operation.complete = true;
        operation.percent = 100;
        operation.text = t('ui.d438a4bceadd94fb');
        toast(operation.text, 'success');
        await profiles.refresh().catch(error => toast(String(error), 'warning'));
    } catch (error) {
        operation.error = String(error);
        operation.text = t('downloadsDesign.failed');
        toast(operation.error, 'error');
        throw error;
    } finally {
        stop?.();
        operation.busy = false;
        contentActivity.updateProfileId = '';
        modpackInstallation.continueQueue();
    }
}

export const modpackUpdates = {
    get task() { return task; },
    forProfile(profileId: string) { return entries[profileId]; },
    check,
    checkLibrary,
    apply,
    installed(profileId: string, source: string, projectId: string, versionId: string, version: string) {
        const entry = entryFor(profileId);
        entry.revision++;
        entry.info = { hasUpdate: false, currentVersion: version, latestVersion: version, changelog: null, source, projectId, versionId };
        entry.diff = null;
        entry.checking = false;
        entry.checkedAt = Date.now();
        libraryCheckedAt = 0;
    },
    dismiss() { if (!task?.busy) task = null; }
};
