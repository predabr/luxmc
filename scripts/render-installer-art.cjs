const { chromium } = require('playwright-core');
const { spawn } = require('node:child_process');
const { join, resolve } = require('node:path');
const { writeFileSync } = require('node:fs');
async function main() {
    const root = resolve(__dirname, '..');
    const python = process.env.LUXMC_PYTHON || join(process.env.USERPROFILE, '.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe');
    const server = spawn(python, ['-m', 'http.server', '8776', '--bind', '127.0.0.1'], { cwd: root, windowsHide: true, stdio: 'ignore' });
    let browser;
    try {
        for (let attempt = 0; attempt < 40; attempt++) {
            try { if ((await fetch('http://127.0.0.1:8776/packaging/windows/scene.html')).ok) break; } catch {}
            if (attempt === 39) throw new Error('Servidor de prévia indisponível.');
            await new Promise(resolve => setTimeout(resolve, 100));
        }
        browser = await chromium.launch({ executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe', headless: true, args: ['--enable-webgl', '--use-angle=swiftshader'] });
        const page = await browser.newPage({ viewport: { width: 656, height: 1256 }, deviceScaleFactor: 1 });
        const errors = [];
        page.on('pageerror', error => errors.push(error.message));
        await page.goto('http://127.0.0.1:8776/packaging/windows/scene.html');
        await page.waitForFunction(() => window.artReady === true);
        await page.screenshot({ path: join(root, 'packaging/windows/sidebar-preview.png') });
        await page.setViewportSize({ width: 600, height: 228 });
        await page.goto('http://127.0.0.1:8776/packaging/windows/scene.html?header');
        await page.waitForFunction(() => window.artReady === true);
        await page.screenshot({ path: join(root, 'packaging/windows/header-preview.png') });
        if (errors.length) throw new Error(errors.join('\n'));
        writeFileSync(join(root, 'docs/validation/installer-art-render.json'), JSON.stringify({ renderer: 'Three.js WebGL', errors, sidebar: [656, 1256], header: [600, 228] }, null, 2));
        console.log('Arte 3D renderizada sem erros WebGL.');
    } finally {
        if (browser) await browser.close();
        server.kill();
    }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
