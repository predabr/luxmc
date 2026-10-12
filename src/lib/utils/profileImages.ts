export function encodeProfileImage(canvas: HTMLCanvasElement): string {
    for (const quality of [.92, .85, .78, .68, .55]) {
        const value = canvas.toDataURL('image/webp', quality);
        if (value.length < 250000) return value;
    }
    throw new Error('image-size');
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
