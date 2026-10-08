export async function prepareAnimatedImage(file: File): Promise<string> {
    if (!file.size || file.size > 12 * 1024 * 1024) throw new Error('limit');
    let bytes = await file.arrayBuffer();
    if (file.size > 700000) {
        bytes = await new Promise<ArrayBuffer>((resolve, reject) => {
            const worker = new Worker(new URL('./gif.worker.ts', import.meta.url), {type: 'module'});
            const timer = setTimeout(() => { worker.terminate(); reject(new Error('limit')); }, 60000);
            const finish = () => { clearTimeout(timer); worker.terminate(); };
            worker.onmessage = (event: MessageEvent<{buffer?: ArrayBuffer; error?: string}>) => {
                finish();
                if (event.data.buffer) resolve(event.data.buffer);
                else reject(new Error(event.data.error || 'invalid'));
            };
            worker.onerror = () => { finish(); reject(new Error('invalid')); };
            worker.postMessage(bytes, [bytes]);
        });
    }
    return new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(String(reader.result));
        reader.onerror = () => reject(new Error('invalid'));
        reader.readAsDataURL(new Blob([bytes], {type: 'image/gif'}));
    });
}
