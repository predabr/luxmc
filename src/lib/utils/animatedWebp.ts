type AnimatedFrame = { rgba: Uint8ClampedArray; delay: number; opaque: boolean };

function uint24(bytes: Uint8Array, offset: number, value: number) { bytes[offset] = value & 255; bytes[offset + 1] = (value >> 8) & 255; bytes[offset + 2] = (value >> 16) & 255; }
function join(parts: Uint8Array[]) { const result = new Uint8Array(parts.reduce((length, part) => length + part.length, 0)); let offset = 0; for (const part of parts) { result.set(part, offset); offset += part.length; } return result; }
function chunk(type: string, payload: Uint8Array) { const bytes = new Uint8Array(8 + payload.length + (payload.length & 1)); bytes.set(new TextEncoder().encode(type)); new DataView(bytes.buffer).setUint32(4, payload.length, true); bytes.set(payload, 8); return bytes; }

export async function encodeAnimatedWebp(frames: AnimatedFrame[], canvas: OffscreenCanvas, limit: number): Promise<Uint8Array | null> {
    const context = canvas.getContext('2d', { willReadFrequently: true });
    if (!context || !frames.length) return null;
    const extended = new Uint8Array(10);
    extended[0] = 2 | (frames.some(frame => !frame.opaque) ? 16 : 0);
    uint24(extended, 4, canvas.width - 1); uint24(extended, 7, canvas.height - 1);
    const parts = [chunk('VP8X', extended), chunk('ANIM', new Uint8Array(6))];
    let length = 12 + parts.reduce((size, part) => size + part.length, 0);
    for (const frame of frames) {
        context.putImageData(new ImageData(frame.rgba, canvas.width, canvas.height), 0, 0);
        const blob = await canvas.convertToBlob({ type: 'image/webp', quality: .82 });
        if (blob.type !== 'image/webp') return null;
        const bytes = new Uint8Array(await blob.arrayBuffer());
        const view = new DataView(bytes.buffer);
        const imageChunks: Uint8Array[] = [];
        let imageFound = false;
        for (let offset = 12; offset + 8 <= bytes.length;) {
            const type = String.fromCharCode(...bytes.subarray(offset, offset + 4));
            const size = view.getUint32(offset + 4, true);
            const end = offset + 8 + size + (size & 1);
            if (end > bytes.length) return null;
            if (['ALPH', 'VP8 ', 'VP8L'].includes(type)) imageChunks.push(bytes.slice(offset, end));
            if (type === 'VP8 ' || type === 'VP8L') imageFound = true;
            offset = end;
        }
        if (!imageFound) return null;
        const header = new Uint8Array(16);
        uint24(header, 6, canvas.width - 1); uint24(header, 9, canvas.height - 1); uint24(header, 12, frame.delay);
        header[15] = 2;
        const encoded = chunk('ANMF', join([header, ...imageChunks]));
        parts.push(encoded); length += encoded.length;
        if (length > limit) return null;
    }
    const header = new Uint8Array(12);
    header.set(new TextEncoder().encode('RIFF')); header.set(new TextEncoder().encode('WEBP'), 8);
    new DataView(header.buffer).setUint32(4, length - 8, true);
    return join([header, ...parts]);
}
