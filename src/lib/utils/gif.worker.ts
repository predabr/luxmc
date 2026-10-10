import {parseGIF, decompressFrame} from 'gifuct-js';
import {GIFEncoder, quantize, applyPalette} from 'gifenc';
import { encodeAnimatedWebp } from './animatedWebp';

self.onmessage = async (event: MessageEvent<{bytes: ArrayBuffer; quality: 'high' | 'balanced'}>) => {
    try {
        const parsed = parseGIF(event.data.bytes);
        const frames = parsed.frames.filter(frame => 'image' in frame);
        const {width, height} = parsed.lsd;
        if (parsed.header.signature !== 'GIF' || !width || !height || !frames.length) throw new Error('invalid');
        if (width * height > 16000000 || width * height * frames.length > 128000000 || frames.length > 1500) throw new Error('limit');
        const source = new OffscreenCanvas(width, height);
        const context = source.getContext('2d', {willReadFrequently: true});
        if (!context) throw new Error('invalid');
        const high = event.data.quality === 'high';
        for (const limit of high ? [320, 256, 192, 128, 96, 64] : [192, 128, 96, 64]) {
            const ratio = Math.min(1, limit / Math.max(width, height));
            const target = new OffscreenCanvas(Math.max(1, Math.round(width * ratio)), Math.max(1, Math.round(height * ratio)));
            const outputContext = target.getContext('2d', {willReadFrequently: true});
            if (!outputContext) throw new Error('invalid');
            outputContext.imageSmoothingEnabled = true;
            outputContext.imageSmoothingQuality = 'high';
            const encoder = GIFEncoder();
            const cached = target.width * target.height * frames.length * 4 <= 96 * 1024 * 1024;
            const images: Array<{ rgba: Uint8ClampedArray; delay: number; opaque: boolean }> = [];
            const write = (rgba: Uint8ClampedArray, delay: number, previous: Uint8ClampedArray | null, nextOpaque: boolean) => {
                const palette = [[0, 0, 0, 0], ...quantize(rgba, high ? 255 : 127, {format: 'rgba4444', oneBitAlpha: true}).filter(color => color[3] !== 0)];
                const indices = applyPalette(rgba, palette, 'rgba4444');
                if (previous) {
                    for (let pixel = 0, index = 0; pixel < rgba.length; pixel += 4, index++) {
                        if (previous[pixel + 3] >= 128 && Math.abs(rgba[pixel] - previous[pixel]) <= 2 && Math.abs(rgba[pixel + 1] - previous[pixel + 1]) <= 2 && Math.abs(rgba[pixel + 2] - previous[pixel + 2]) <= 2) indices[index] = 0;
                    }
                }
                encoder.writeFrame(indices, target.width, target.height, {palette, delay, repeat: 0, transparent: true, transparentIndex: 0, dispose: nextOpaque ? 1 : 2});
            };
            context.clearRect(0, 0, width, height);
            for (const raw of frames) {
                const bounds = raw.image.descriptor;
                if (!bounds.width || !bounds.height || bounds.left + bounds.width > width || bounds.top + bounds.height > height) throw new Error('invalid');
                const frame = decompressFrame(raw, parsed.gct, true);
                const dims = frame.dims;
                if (!dims.width || !dims.height || dims.left + dims.width > width || dims.top + dims.height > height) throw new Error('invalid');
                const previous = frame.disposalType === 3 ? context.getImageData(0, 0, width, height) : null;
                const patch = new OffscreenCanvas(dims.width, dims.height);
                const patchContext = patch.getContext('2d');
                if (!patchContext) throw new Error('invalid');
                patchContext.putImageData(new ImageData(frame.patch, dims.width, dims.height), 0, 0);
                context.drawImage(patch, dims.left, dims.top);
                outputContext.clearRect(0, 0, target.width, target.height);
                outputContext.drawImage(source, 0, 0, target.width, target.height);
                const rgba = outputContext.getImageData(0, 0, target.width, target.height).data;
                if (cached) {
                    let opaque = true;
                    for (let pixel = 3; pixel < rgba.length; pixel += 4) { if (rgba[pixel] < 128) { opaque = false; break; } }
                    images.push({ rgba, delay: frame.delay, opaque });
                } else write(rgba, frame.delay, null, false);
                if (frame.disposalType === 2) context.clearRect(dims.left, dims.top, dims.width, dims.height);
                else if (previous) context.putImageData(previous, 0, 0);
            }
            if (high && cached) {
                const optimized = await encodeAnimatedWebp(images, target, 700000).catch(() => null);
                if (optimized) { const buffer = optimized.slice().buffer; self.postMessage({buffer, mime: 'image/webp'}, {transfer: [buffer]}); return; }
            }
            for (let index = 0; index < images.length; index++) {
                const frame = images[index];
                write(frame.rgba, frame.delay, index > 0 && frame.opaque ? images[index - 1].rgba : null, images[(index + 1) % images.length].opaque);
            }
            encoder.finish();
            const bytes = encoder.bytes();
            if (bytes.length <= 700000) {
                const buffer = bytes.slice().buffer;
                self.postMessage({buffer, mime: 'image/gif'}, {transfer: [buffer]});
                return;
            }
        }
        throw new Error('limit');
    } catch (error) { self.postMessage({error: error instanceof Error ? error.message : 'invalid'}); }
};
