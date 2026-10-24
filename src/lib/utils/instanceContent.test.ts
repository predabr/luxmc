import { describe, expect, it } from 'vitest';
import { instanceContentEntries } from './instanceContent';
import type { FileTreeEntry } from '$lib/api/types';

const file = (name: string, isDir = false): FileTreeEntry => ({ name, isDir, path: name, size: 4096 });
describe('instance content listings', () => {
    it('keeps only actual enabled and disabled mods without exposing loader caches as mods', () => {
        const entries = [file('memory_repo', true), file('AccuracyCrosshair.jar'), file('Example.JAR.disabled'), file('readme.txt'), file('.index')];
        expect(instanceContentEntries(entries, 'mods').map(entry => entry.name)).toEqual(['AccuracyCrosshair.jar', 'Example.JAR.disabled']);
        expect(instanceContentEntries(entries, 'config')).toBe(entries);
    });
    it('keeps extracted shaders and archives while leaving settings and hidden files in the file manager', () => {
        const entries = [file('Better MC - High', true), file('Better MC - High.txt'), file('Complementary.zip'), file('.cache', true)];
        expect(instanceContentEntries(entries, 'shaderpacks').map(entry => entry.name)).toEqual(['Better MC - High', 'Complementary.zip']);
        expect(instanceContentEntries(entries, 'saves/World/datapacks').map(entry => entry.name)).toEqual(['Better MC - High', 'Complementary.zip']);
    });
});
