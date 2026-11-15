export function encodeProfileImage(canvas: HTMLCanvasElement): string {
    for (const quality of [.92, .85, .78, .68, .55]) {
        const value = canvas.toDataURL('image/webp', quality);
        if (value.length < 250000) return value;
    }
    throw new Error('image-size');
}

export async function imageDimensions(file: Blob): Promise<{ width: number; height: number } | null> {
    const head = new Uint8Array(await file.slice(0, 65536).arrayBuffer());
    const view = new DataView(head.buffer, head.byteOffset, head.byteLength);
    const text = (start: number, length: number) => String.fromCharCode(...head.subarray(start, start + length));
    if (head.length >= 24 && head[0] === 0x89 && text(1, 3) === 'PNG') {
        return { width: view.getUint32(16), height: view.getUint32(20) };
    }
    if (head.length >= 10 && (text(0, 6) === 'GIF87a' || text(0, 6) === 'GIF89a')) {
        return { width: view.getUint16(6, true), height: view.getUint16(8, true) };
    }
    if (head.length >= 30 && text(0, 4) === 'RIFF' && text(8, 4) === 'WEBP') {
        const chunk = text(12, 4);
        if (chunk === 'VP8 ') { return { width: view.getUint16(26, true) & 0x3fff, height: view.getUint16(28, true) & 0x3fff }; }
        if (chunk === 'VP8L') { const bits = view.getUint32(21, true); return { width: (bits & 0x3fff) + 1, height: ((bits >>> 14) & 0x3fff) + 1 }; }
        if (chunk === 'VP8X') { return { width: (view.getUint8(24) | view.getUint8(25) << 8 | view.getUint8(26) << 16) + 1, height: (view.getUint8(27) | view.getUint8(28) << 8 | view.getUint8(29) << 16) + 1 }; }
        return null;
    }
    if (head.length >= 4 && head[0] === 0xff && head[1] === 0xd8) {
        let offset = 2;
        while (offset + 9 < head.length) {
            if (head[offset] !== 0xff) { offset++; continue; }
            const marker = head[offset + 1];
            if (marker === 0xd8 || marker === 0x01 || marker >= 0xd0 && marker <= 0xd7) { offset += 2; continue; }
            const length = view.getUint16(offset + 2);
            if (marker >= 0xc0 && marker <= 0xcf && marker !== 0xc4 && marker !== 0xc8 && marker !== 0xcc) {
                return { width: view.getUint16(offset + 7), height: view.getUint16(offset + 5) };
            }
            if (length < 2) return null;
            offset += 2 + length;
        }
        return null;
    }
    return null;
}

export function profileCover(preset: 'prism' | 'aurora' | 'horizon' | 'constellation'): string {
    const canvas = document.createElement('canvas');
    canvas.width = 1280; canvas.height = 384;
    const context = canvas.getContext('2d');
    if (!context) throw new Error('canvas');
    const tokens = getComputedStyle(document.documentElement);
    const color = (token: string, alpha = 1) => `rgb(${tokens.getPropertyValue(`--${token}`).trim()} / ${alpha})`;
    const gradient = context.createLinearGradient(0, 0, canvas.width, canvas.height);
    gradient.addColorStop(0, color('bg-subtle'));
    gradient.addColorStop(1, color('bg-overlay'));
    context.fillStyle = gradient; context.fillRect(0, 0, canvas.width, canvas.height);
    if (preset === 'prism') {
        for (let index = 0; index < 7; index++) {
            context.fillStyle = color(index % 2 ? 'info' : 'brand-400', .1 + index * .025);
            context.beginPath(); context.moveTo(index * 180 - 250, 0); context.lineTo(index * 180 + 90, 0); context.lineTo(index * 180 + 480, 384); context.lineTo(index * 180 + 140, 384); context.closePath(); context.fill();
        }
    } else if (preset === 'aurora') {
        for (const [x, y, token] of [[200, 40, 'brand-400'], [650, 240, 'info'], [1100, 30, 'success']] as const) {
            const glow = context.createRadialGradient(x, y, 10, x, y, 490);
            glow.addColorStop(0, color(token, .4)); glow.addColorStop(1, color(token, 0));
            context.fillStyle = glow; context.fillRect(0, 0, canvas.width, canvas.height);
        }
    } else if (preset === 'horizon') {
        const sky = context.createLinearGradient(0, 0, 0, 384);
        sky.addColorStop(0, color('brand-500', .48)); sky.addColorStop(1, color('info', .1));
        context.fillStyle = sky; context.fillRect(0, 0, canvas.width, canvas.height);
        context.fillStyle = color('warning', .75); context.fillRect(985, 72, 56, 56);
        for (let layer = 0; layer < 3; layer++) {
            context.fillStyle = color(layer === 0 ? 'brand-400' : layer === 1 ? 'bg-subtle' : 'bg-overlay', layer === 0 ? .27 : .85);
            for (let index = 0; index < 22; index++) { const height = 55 + layer * 43 + (Math.sin(index * .85 + layer) + 1) * 42; context.fillRect(index * 64, 384 - height, 64, height); }
        }
    } else {
        for (let index = 0; index < 45; index++) {
            const x = (index * 193 + 67) % 1280, y = (index * 97 + 29) % 384;
            context.fillStyle = color(index % 4 ? 'brand-300' : 'warning', index % 3 ? .6 : .28);
            context.fillRect(x, y, index % 4 ? 2 : 4, index % 4 ? 2 : 4);
            if (index % 4 === 0) { context.strokeStyle = color('brand-400', .15); context.beginPath(); context.moveTo(x, y); context.lineTo((x + 193) % 1280, (y + 97) % 384); context.stroke(); }
        }
    }
    return encodeProfileImage(canvas);
}
