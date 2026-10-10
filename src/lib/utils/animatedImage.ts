import { settings } from '$lib/stores/settings.svelte';

export async function prepareAnimatedImage(file: File): Promise<string> {
    if (!file.size || file.size > 12 * 1024 * 1024) throw new Error('limit');
    let bytes = await file.arrayBuffer();
    let mime = 'image/gif';
    if (file.size > 700000) {
        bytes = await new Promise<ArrayBuffer>((resolve, reject) => {
            const worker = new Worker(new URL('./gif.worker.ts', import.meta.url), {type: 'module'});
            const timer = setTimeout(() => { worker.terminate(); reject(new Error('limit')); }, 60000);
            const finish = () => { clearTimeout(timer); worker.terminate(); };
            worker.onmessage = (event: MessageEvent<{buffer?: ArrayBuffer; mime?: 'image/gif' | 'image/webp'; error?: string}>) => {
                finish();
                if (event.data.buffer) { mime = event.data.mime === 'image/webp' ? 'image/webp' : 'image/gif'; resolve(event.data.buffer); }
                else reject(new Error(event.data.error || 'invalid'));
            };
            worker.onerror = () => { finish(); reject(new Error('invalid')); };
            worker.postMessage({ bytes, quality: settings.value.gifQuality ?? 'high' }, [bytes]);
        });
    }
    return new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(String(reader.result));
        reader.onerror = () => reject(new Error('invalid'));
        reader.readAsDataURL(new Blob([bytes], {type: mime}));
    });
}

export function isAnimatedImage(value: string): boolean {
    if (value.startsWith('data:image/gif;')) return true;
    if (!value.startsWith('data:image/webp;')) return false;
    try { const header = atob(value.split(',')[1].slice(0, 64)); return header.slice(12, 16) === 'VP8X' && (header.charCodeAt(20) & 2) !== 0; } catch { return false; }
}
