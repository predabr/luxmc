import type { DownloadProgress } from '$lib/api/types';

export function transferComplete(progress: DownloadProgress): boolean {
    return progress.phase === 'done' || (progress.phase === 'java' && progress.total > 0 && progress.completed >= progress.total);
}

export function installationStep(phase: string, completed = false): number {
    if (completed || phase === 'complete') return 3;
    if (phase === 'indexing') return 3;
    if (phase === 'validation') return 2;
    if (['extracting', 'downloading', 'mods', 'files', 'overrides', 'preparing'].includes(phase)) return 1;
    return 0;
}

export function installationProgressPercent(phase: string, progress: number, packageProgress = -1): number {
    if (phase === 'complete') return 100;
    const clamp = (value: number) => Math.min(100, Math.max(0, value));
    if (phase === 'package') return Math.round(clamp(packageProgress >= 0 ? packageProgress : progress) * 0.25);
    if (['extracting', 'downloading', 'mods', 'files', 'overrides', 'preparing'].includes(phase)) {
        return Math.round(25 + clamp(progress) * 0.25);
    }
    if (phase === 'validation') {
        const validationProgress = progress >= 90 ? clamp((progress - 96) * 50) : clamp(progress);
        return Math.round(50 + validationProgress * 0.25);
    }
    if (phase === 'indexing') {
        const indexingProgress = progress >= 90 ? clamp((progress - 96) * 25) : clamp(progress);
        return Math.round(75 + indexingProgress * 0.25);
    }
    return Math.round(clamp(progress) * 0.25);
}

export function elapsedTime(startedAt: number, now: number): string {
    const seconds = Math.max(0, Math.floor((now - startedAt) / 1000));
    return seconds < 60 ? `${seconds}s` : `${Math.floor(seconds / 60)}m ${String(seconds % 60).padStart(2, '0')}s`;
}
