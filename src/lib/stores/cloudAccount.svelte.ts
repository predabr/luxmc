import { luxAccountSync, type CloudPreferences } from "$lib/api/luxAccount";
import { settings } from "./settings.svelte";
import { themeStore } from "./theme.svelte";

let ready = $state(false);
let error = $state("");
let busy = $state(false);
let identity = "";
let generation = 0;
let revision = 0;
let baseline = "";
let pending: CloudPreferences | null = null;
let saveTimer: ReturnType<typeof setTimeout> | undefined;
let pollTimer: ReturnType<typeof setTimeout> | undefined;
const selected = (): CloudPreferences => ({ theme: settings.value.theme, accentTheme: settings.value.accentTheme, language: settings.value.language, animations: settings.value.animations });

async function pull(run: number) {
    if (run !== generation || !identity || busy || pending) return;
    try {
        const remote = await luxAccountSync(identity);
        if (run !== generation) return;
        if (pending || busy) return;
        const prefs: CloudPreferences = {};
        if (remote.preferences.theme) prefs.theme = remote.preferences.theme;
        if (remote.preferences.accentTheme) prefs.accentTheme = remote.preferences.accentTheme;
        if (remote.preferences.language) prefs.language = remote.preferences.language;
        if (typeof remote.preferences.animations === "boolean") prefs.animations = remote.preferences.animations;
        revision = remote.revision;
        settings.patch(prefs);
        if (prefs.theme) themeStore.setTheme(prefs.theme);
        if (prefs.accentTheme) themeStore.setAccent(prefs.accentTheme);
        baseline = JSON.stringify(selected());
        ready = true;
        error = "";
    } catch (cause) { if (run === generation) error = String(cause); }
}
async function poll(run: number) {
    await pull(run);
    if (run === generation) pollTimer = setTimeout(() => void poll(run), 30000);
}
async function save(run: number) {
    if (run !== generation || !pending || busy) return;
    const prefs = pending;
    pending = null;
    busy = true;
    try {
        const remote = await luxAccountSync(identity, prefs, revision);
        if (run !== generation) return;
        revision = remote.revision;
        baseline = JSON.stringify(prefs);
        error = "";
    } catch (cause) {
        if (run === generation) { error = String(cause); pending = prefs; }
    } finally {
        if (run === generation) {
            busy = false;
            if (pending && !error) saveTimer = setTimeout(() => void save(run), 700);
        }
    }
}
export const cloudAccount = {
    get ready() { return ready; }, get error() { return error; }, get busy() { return busy; },
    connect(id: string) {
        this.disconnect();
        identity = id;
        void poll(generation);
    },
    schedule(preferences: CloudPreferences) {
        if (!ready || !identity || JSON.stringify(preferences) === baseline) return;
        pending = preferences;
        clearTimeout(saveTimer);
        saveTimer = setTimeout(() => void save(generation), 900);
    },
    async reload() { pending = null; clearTimeout(saveTimer); await pull(generation); },
    disconnect() {
        generation++;
        identity = ""; ready = false; error = ""; busy = false; pending = null;
        clearTimeout(saveTimer); clearTimeout(pollTimer);
    }
};
