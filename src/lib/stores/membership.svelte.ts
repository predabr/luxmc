import { membershipStatus, membershipSync, type AppearancePreferences, type AppearancePreset, type Membership } from '$lib/api/membership';
import { settings } from './settings.svelte';

let current = $state<Membership | null>(null);
let presets = $state<AppearancePreset[]>([]);
let owner = $state(false);
let ready = $state(false);
let error = $state('');
let busy = $state(false);
let identity = '';
let socialId = $state('');
let generation = 0;
let revision = 0;
let baseline = '';
let pending: AppearancePreferences | null = null;
let saveTimer: ReturnType<typeof setTimeout> | undefined;
let pollTimer: ReturnType<typeof setTimeout> | undefined;
export function appearanceSnapshot(): AppearancePreferences {
    const value = settings.value;
    return { density: value.density, motionStyle: value.motionStyle || 'expressive', motionSpeed: value.motionSpeed || 'balanced', wallpaperFps: value.wallpaperFps || 30, pauseSkinWhileGaming: value.pauseSkinWhileGaming !== false, preloadContentIcons: value.preloadContentIcons !== false, respectReducedMotion: value.respectReducedMotion !== false, gifQuality: value.gifQuality || 'high', interfaceOpacity: Math.min(1, Math.max(.35, value.interfaceOpacity ?? 1)) };
}
const paid = () => Boolean(current && current.plan !== 'free' && current.expiresAt && current.expiresAt > Date.now() / 1000);
async function pull(run: number) {
    if (run !== generation || !identity || busy || pending) return;
    busy = true;
    try {
        const remote = await membershipStatus(identity);
        if (run !== generation) return;
        current = remote.membership; owner = remote.identity.owner; socialId = remote.identity.id;
        if (paid() && (!ready || remote.settings.revision !== revision) && !pending) settings.patch(remote.settings.preferences);
        revision = remote.settings.revision; presets = remote.settings.presets;
        baseline = JSON.stringify(appearanceSnapshot()); ready = true; error = '';
    } catch (cause) { if (run === generation) error = String(cause); }
    finally { if (run === generation) busy = false; }
}
async function poll(run: number) {
    await pull(run);
    if (run === generation) pollTimer = setTimeout(() => void poll(run), 30000);
}
async function save(run: number) {
    if (run !== generation || !pending || busy || !paid()) return;
    const preferences = pending; pending = null; busy = true;
    try {
        const result = await membershipSync(identity, preferences, presets, revision);
        if (run !== generation) return;
        revision = result.revision; baseline = JSON.stringify(preferences); error = '';
    } catch (cause) {
        if (run === generation) { error = String(cause); ready = false; pending = null; }
    } finally {
        if (run === generation) { busy = false; if (pending) saveTimer = setTimeout(() => void save(run), 900); }
    }
}
export const membershipState = {
    get socialId() { return socialId; },
    ownerFor(accountId: string) { return ready && !error && owner && identity === accountId; },
    get current() { return current; }, get presets() { return presets; }, get owner() { return owner; }, get ready() { return ready; }, get error() { return error; }, get busy() { return busy; }, get paid() { return paid(); },
    connect(id: string) { this.disconnect(); identity = id; void poll(generation); },
    schedule(preferences: AppearancePreferences) { if (!ready || !identity || !paid() || JSON.stringify(preferences) === baseline) return; pending = preferences; clearTimeout(saveTimer); saveTimer = setTimeout(() => void save(generation), 900); },
    async refresh() { clearTimeout(saveTimer); pending = null; await pull(generation); },
    async savePreset(name: string) {
        if (!paid() || busy || !name.trim() || name.length > 40 || presets.length >= (current?.presets || 0)) throw new Error('Confira o nome e o limite de perfis do seu plano.');
        busy = true; clearTimeout(saveTimer); pending = null; const run = generation;
        const next = [...presets, { id: crypto.randomUUID(), name: name.trim(), preferences: appearanceSnapshot() }];
        try { const result = await membershipSync(identity, appearanceSnapshot(), next, revision); if (run === generation) { revision = result.revision; presets = next; baseline = JSON.stringify(appearanceSnapshot()); error = ''; } }
        finally { if (run === generation) busy = false; }
    },
    async applyPreset(id: string) { const preset = presets.find(item => item.id === id); if (!paid() || !preset) return; settings.patch(preset.preferences); this.schedule(appearanceSnapshot()); },
    async deletePreset(id: string) {
        if (!paid() || busy) return; busy = true; clearTimeout(saveTimer); pending = null; const run = generation; const next = presets.filter(item => item.id !== id);
        try { const result = await membershipSync(identity, appearanceSnapshot(), next, revision); if (run === generation) { revision = result.revision; presets = next; baseline = JSON.stringify(appearanceSnapshot()); } }
        finally { if (run === generation) busy = false; }
    },
    disconnect() { generation++; identity = ''; socialId = ''; current = null; owner = false; ready = false; error = ''; busy = false; pending = null; presets = []; clearTimeout(saveTimer); clearTimeout(pollTimer); }
};
