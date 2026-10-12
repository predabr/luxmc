import { describe, expect, it } from 'vitest';
import { elapsedTime, installationStep, transferComplete } from './installProgress';
import type { DownloadProgress } from '$lib/api/types';

describe('installation progress', () => {
    const progress: DownloadProgress = { phase: 'assets', total: 12, completed: 12, currentFile: '', bytesDownloaded: 50, totalBytes: 50, speed: null };
    it('keeps the installation active when a concurrent group or an individual file finishes', () => {
        for (const phase of ['client', 'libraries', 'assets', 'asset_index', 'downloading']) expect(transferComplete({...progress, phase})).toBe(false);
        expect(transferComplete({...progress, phase: 'done'})).toBe(true);
        expect(transferComplete({...progress, phase: 'java'})).toBe(true);
        expect(transferComplete({...progress, phase: 'java', completed: 1})).toBe(false);
    });
    it('maps work to the correct visible step and never declares validation complete prematurely', () => {
        expect(installationStep('package')).toBe(0);
        expect(installationStep('preparing')).toBe(1);
        expect(installationStep('downloading')).toBe(1);
        expect(installationStep('validation')).toBe(2);
        expect(installationStep('indexing')).toBe(2);
        expect(installationStep('complete')).toBe(3);
    });
    it('formats elapsed duration without inventing a completion estimate', () => {
        expect(elapsedTime(10_000, 9_000)).toBe('0s');
        expect(elapsedTime(0, 59_999)).toBe('59s');
        expect(elapsedTime(0, 65_000)).toBe('1m 05s');
    });
});
