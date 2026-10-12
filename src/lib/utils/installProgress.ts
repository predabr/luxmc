import type { DownloadProgress } from '$lib/api/types';

export function transferComplete(progress: DownloadProgress): boolean {
    return progress.phase === 'done' || (progress.phase === 'java' && progress.total > 0 && progress.completed >= progress.total);
}

export function installationStep(phase: string, completed = false): number {
    if (completed || phase === 'complete') return 3;
    if (['validation', 'indexing'].includes(phase)) return 2;
    if (['extracting', 'downloading', 'mods', 'files', 'overrides', 'preparing'].includes(phase)) return 1;
    return 0;
}

export function elapsedTime(startedAt: number, now: number): string {
    const seconds = Math.max(0, Math.floor((now - startedAt) / 1000));
    return seconds < 60 ? `${seconds}s` : `${Math.floor(seconds / 60)}m ${String(seconds % 60).padStart(2, '0')}s`;
}
