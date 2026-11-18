const { chromium } = require('playwright-core');
const fs = require('node:fs');
const path = require('node:path');

(async () => {
    const root = path.resolve(__dirname, '..');
    const hero = 'static/brand/luxmc-hero.jpg';
    const radius = 0.2;
    const targets = [
        { out: 'src-tauri/icons/source-1024.png', size: 1024 },
        { out: 'static/logo.png', size: 512 },
        { out: 'static/brand/luxmc-mark-512.png', size: 512 },
        { out: 'static/branding/luxmc-emblem.png', size: 512 },
        { out: 'static/branding/luxmc-emblem-128.png', size: 128 },
        { out: 'static/favicon.png', size: 128 },
        { out: 'website/assets/favicon.png', size: 128 },
        { out: 'website/assets/logo.png', size: 512 }
    ];
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const page = await browser.newPage({ viewport: { width: 1200, height: 1200 }, deviceScaleFactor: 1 });
    await page.goto('file:///' + path.join(root, hero).replace(/\\/g, '/'));
    const render = async (size) => page.evaluate(async (size) => {
        const img = document.querySelector('img');
        const canvas = document.createElement('canvas');
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext('2d');
        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = 'high';
        const r = size * 0.2;
        ctx.beginPath();
        ctx.moveTo(r, 0);
        ctx.arcTo(size, 0, size, size, r);
        ctx.arcTo(size, size, 0, size, r);
        ctx.arcTo(0, size, 0, 0, r);
        ctx.arcTo(0, 0, size, 0, r);
        ctx.closePath();
        ctx.clip();
        ctx.drawImage(img, 0, 0, img.naturalWidth, img.naturalHeight, 0, 0, size, size);
        return canvas.toDataURL('image/png');
    }, size);
    for (const t of targets) {
        const dataUrl = await render(t.size);
        const outPath = path.join(root, t.out);
        fs.mkdirSync(path.dirname(outPath), { recursive: true });
        fs.writeFileSync(outPath, Buffer.from(dataUrl.split(',')[1], 'base64'));
        console.log('rendered', t.out, t.size);
    }
    await browser.close();
})().catch((error) => { console.error(error); process.exit(1); });
