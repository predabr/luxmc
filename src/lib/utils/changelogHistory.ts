import { modsChangelog } from '$lib/api/mods';

const cache = new Map<string, { body: string; expires: number }>();
const pending = new Map<string, Promise<string>>();
const queue: Array<() => void> = [];
let active = 0;

function drain() {
    while (active < 3 && queue.length) queue.shift()!();
}

export function versionChangelog(projectId: string, source: string, versionId: string): Promise<string> {
    const key = `${source}:${projectId}:${versionId}`;
    const existing = cache.get(key);
    if (existing && existing.expires > Date.now()) return Promise.resolve(existing.body);
    const inflight = pending.get(key);
    if (inflight) return inflight;
    const request = new Promise<string>((resolve, reject) => {
        queue.push(() => {
            active += 1;
            void modsChangelog(projectId, versionId, source).then(body => {
                if (cache.size >= 256) cache.delete(cache.keys().next().value!);
                cache.set(key, { body, expires: Date.now() + 600_000 });
                resolve(body);
            }).catch(reject).finally(() => {
                active -= 1;
                pending.delete(key);
                drain();
            });
        });
    });
    pending.set(key, request);
    drain();
    return request;
}
