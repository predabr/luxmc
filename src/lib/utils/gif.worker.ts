import {parseGIF, decompressFrame} from 'gifuct-js';
import {GIFEncoder, quantize, applyPalette} from 'gifenc';

self.onmessage = (event: MessageEvent<ArrayBuffer>) => {
    try {
        const parsed = parseGIF(event.data);
        const frames = parsed.frames.filter(frame => 'image' in frame);
        const {width, height} = parsed.lsd;
        if (parsed.header.signature !== 'GIF' || !width || !height || !frames.length) throw new Error('invalid');
        if (width * height > 16000000 || width * height * frames.length > 128000000 || frames.length > 1500) throw new Error('limit');
        const source = new OffscreenCanvas(width, height);
        const context = source.getContext('2d', {willReadFrequently: true});
        if (!context) throw new Error('invalid');
        for (const limit of [192, 128, 96, 64]) {
            const ratio = Math.min(1, limit / Math.max(width, height));
            const target = new OffscreenCanvas(Math.max(1, Math.round(width * ratio)), Math.max(1, Math.round(height * ratio)));
            const outputContext = target.getContext('2d', {willReadFrequently: true});
            if (!outputContext) throw new Error('invalid');
            const encoder = GIFEncoder();
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
                const palette = quantize(rgba, 128, {format: 'rgba4444', oneBitAlpha: true});
                const transparentIndex = palette.findIndex(color => color[3] === 0);
                encoder.writeFrame(applyPalette(rgba, palette, 'rgba4444'), target.width, target.height, {palette, delay: frame.delay, repeat: 0, transparent: transparentIndex >= 0, transparentIndex: Math.max(0, transparentIndex), dispose: 2});
                if (frame.disposalType === 2) context.clearRect(dims.left, dims.top, dims.width, dims.height);
                else if (previous) context.putImageData(previous, 0, 0);
            }
            encoder.finish();
            const bytes = encoder.bytes();
            if (bytes.length <= 700000) {
                const buffer = bytes.slice().buffer;
                self.postMessage({buffer}, {transfer: [buffer]});
                return;
            }
        }
        throw new Error('limit');
    } catch (error) { self.postMessage({error: error instanceof Error ? error.message : 'invalid'}); }
};
