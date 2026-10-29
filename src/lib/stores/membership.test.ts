import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import { membershipStatus, membershipSync, type AppearancePreset } from '$lib/api/membership';
import { membershipState, appearanceSnapshot } from './membership.svelte';
import { settings } from './settings.svelte';

vi.mock('$lib/api/membership', () => ({ membershipStatus: vi.fn(), membershipSync: vi.fn() }));
function response(plan: 'free'|'plus'|'pro' = 'plus', revision = 1, presets: AppearancePreset[] = []) {
    return { identity: { id: '22222222-2222-4222-8222-222222222222', username: 'Player', owner: false }, membership: { plan, expiresAt: Math.floor(Date.now()/1000)+86400, revision:1, collections: plan === 'pro' ? 24 : plan === 'plus' ? 12 : 6, presets: plan === 'pro' ? 20 : plan === 'plus' ? 6 : 0 }, settings: { preferences: {density:'compact' as const,motionStyle:'subtle' as const}, presets, revision } };
}
describe('membership synchronization', () => {
    it('a failed appearance upload does not disable a verified unexpired Pro plan', async () => {
        vi.mocked(membershipStatus).mockResolvedValue(response('pro'));
        vi.mocked(membershipSync).mockRejectedValue(new Error('appearance offline'));
        membershipState.connect('pro-account'); await vi.advanceTimersByTimeAsync(0);
        settings.patch({density:'spacious'}); membershipState.schedule(appearanceSnapshot());
        await vi.advanceTimersByTimeAsync(1000);
        expect(membershipState.ready).toBe(true);
        expect(membershipState.current?.plan).toBe('pro');
        expect(membershipState.paid).toBe(true);
        expect(membershipState.error).toContain('appearance offline');
        membershipState.disconnect();
        expect(membershipState.ready).toBe(false);
        expect(membershipState.paid).toBe(false);
    });
    it('owner privileges belong only to the verified current account and disappear on account change', async () => {
        vi.useFakeTimers();
        const verified = response('pro'); verified.identity.owner = true;
        vi.mocked(membershipStatus).mockResolvedValueOnce(verified).mockResolvedValue(response('pro'));
        membershipState.connect('owner-account'); await vi.advanceTimersByTimeAsync(0);
        expect(membershipState.ownerFor('owner-account')).toBe(true);
        expect(membershipState.ownerFor('another-account')).toBe(false);
        expect(membershipState.socialId).toBe(verified.identity.id);
        membershipState.connect('another-account');
        expect(membershipState.ownerFor('owner-account')).toBe(false);
        await vi.advanceTimersByTimeAsync(0);
        expect(membershipState.ownerFor('another-account')).toBe(false);
    });
    beforeEach(() => { vi.useFakeTimers(); vi.clearAllMocks(); membershipState.disconnect(); settings.patch({density:'comfortable',motionStyle:'expressive'}); });
    afterEach(() => { membershipState.disconnect(); vi.useRealTimers(); });
    it('applies verified remote preferences and debounces launcher edits without sending local paths', async () => {
        vi.mocked(membershipStatus).mockResolvedValue(response()); vi.mocked(membershipSync).mockResolvedValue({ok:true,revision:2});
        membershipState.connect('microsoft-one'); await vi.advanceTimersByTimeAsync(0);
        expect(settings.value.density).toBe('compact'); expect(membershipState.paid).toBe(true);
        settings.patch({density:'spacious',customWallpaperUrl:'C:/private/wallpaper.png',javaPath:'C:/java'}); membershipState.schedule(appearanceSnapshot());
        await vi.advanceTimersByTimeAsync(899); expect(membershipSync).not.toHaveBeenCalled(); await vi.advanceTimersByTimeAsync(1);
        expect(membershipSync).toHaveBeenCalledTimes(1); const [id, prefs] = vi.mocked(membershipSync).mock.calls[0]; expect(id).toBe('microsoft-one'); expect(prefs.density).toBe('spacious'); expect(prefs).not.toHaveProperty('customWallpaperUrl'); expect(prefs).not.toHaveProperty('javaPath');
    });
    it('an old account response cannot apply preferences after the user changes accounts', async () => {
        let finish: (value: ReturnType<typeof response>) => void = () => {};
        vi.mocked(membershipStatus).mockImplementationOnce(() => new Promise(resolve => finish = resolve)).mockResolvedValueOnce({...response('free'),identity:{id:'33333333-3333-4333-8333-333333333333',username:'Second',owner:false}});
        membershipState.connect('first'); membershipState.connect('second'); await vi.advanceTimersByTimeAsync(0); finish(response('pro')); await vi.advanceTimersByTimeAsync(0);
        expect(membershipState.current?.plan).toBe('free'); expect(settings.value.density).toBe('comfortable');
    });
    it('free and expired plans do not upload appearance settings or erase saved local data', async () => {
        const expired=response('free'); vi.mocked(membershipStatus).mockResolvedValue(expired); membershipState.connect('free-account'); await vi.advanceTimersByTimeAsync(0);
        settings.patch({density:'spacious'}); membershipState.schedule(appearanceSnapshot()); await vi.advanceTimersByTimeAsync(1000); expect(membershipSync).not.toHaveBeenCalled(); expect(settings.value.density).toBe('spacious');
    });
    it('saved presets synchronize, apply locally and are removed from the same account', async () => {
        vi.mocked(membershipStatus).mockResolvedValue(response()); vi.mocked(membershipSync).mockResolvedValue({ok:true,revision:2}); membershipState.connect('luxmc:registered'); await vi.advanceTimersByTimeAsync(0);
        await membershipState.savePreset('Meu visual'); expect(membershipState.presets).toHaveLength(1); const saved=membershipState.presets[0];
        settings.patch({density:'spacious'}); await membershipState.applyPreset(saved.id); expect(settings.value.density).toBe('compact');
        await membershipState.deletePreset(saved.id); expect(membershipState.presets).toHaveLength(0); expect(vi.mocked(membershipSync).mock.calls.at(-1)?.[0]).toBe('luxmc:registered');
    });
});
