import { describe, expect, it } from 'vitest';
import { mergeSavedSettings } from './settingsPersistence';

describe('saved appearance recovery', () => {
    it('restores the latest complete appearance after an interrupted backup write', () => {
        const native = { theme: 'light', customBackground: 'custom', customWallpaperUrl: 'my.gif', settingsSavedAt: 200 };
        const backup = { theme: 'dark', customBackground: 'obsidian', settingsSavedAt: 100 };
        expect(mergeSavedSettings(native, backup)).toEqual(native);
        expect(mergeSavedSettings(backup, native)).toEqual(native);
    });
    it('preserves legacy preferences and native-only account fields', () => {
        expect(mergeSavedSettings<{ theme: string; account?: string; settingsSavedAt?: number }>({ theme: 'dark', account: 'kept' }, { theme: 'light' })).toEqual({ theme: 'light', account: 'kept' });
    });
});
