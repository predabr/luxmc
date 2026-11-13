const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('./launcher-video-fixture.cjs');
const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const output = path.resolve('docs/media/2026-10-10/captures');
const ffmpeg = path.join(process.env.LOCALAPPDATA, 'Luxmc-video-tools/imageio/imageio_ffmpeg/binaries/ffmpeg-win-x86_64-v7.1.exe');
const pause = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds));
const shots = [
    { id: 'biblioteca', route: '/instances', seconds: 9, action: async page => {
        await page.getByRole('heading', { name: /Suas instâncias/ }).waitFor();
        await page.mouse.move(730, 540, { steps: 35 });
        await pause(1400);
        await page.mouse.move(1340, 560, { steps: 40 });
    } },
    { id: 'criar-instancia', route: '/instances', seconds: 11, action: async page => {
        await page.getByRole('button', { name: /Nova Instância|Nova instância/ }).last().click();
        await pause(1900);
        await page.getByRole('button', { name: /Java Edition/ }).click();
        await pause(1300);
        await page.getByRole('dialog').getByRole('button', { name: 'Fabric', exact: true }).click();
        await pause(1500);
        await page.getByRole('dialog').getByPlaceholder(/Pesquisar versão/).fill('1.21');
    } },
    { id: 'modpacks', route: '/mods', seconds: 10, action: async page => {
        await page.locator('.catalog-card').first().waitFor();
        await page.mouse.move(655, 480, { steps: 30 });
        await pause(1700);
        await page.mouse.wheel(0, 400);
        await pause(1900);
        await page.mouse.wheel(0, -400);
    } },
    { id: 'conteudos', route: '/instances/demo-vp', seconds: 11, action: async page => {
        await page.getByRole('tab', { name: 'Mods', exact: true }).click();
        await pause(1500);
        await page.getByRole('tab', { name: 'Pacotes de recursos', exact: true }).click();
        await pause(1800);
        await page.getByRole('tab', { name: 'Shaders', exact: true }).click();
        await pause(1700);
        await page.getByRole('tab', { name: 'Mods', exact: true }).click();
    } },
    { id: 'personalizador', route: '/skins', seconds: 11, action: async page => {
        await page.locator('main canvas').first().waitFor();
        await page.getByRole('button', { name: 'Correr', exact: true }).click();
        await pause(2200);
        await page.getByRole('button', { name: 'Andar', exact: true }).click();
        const box = await page.locator('main canvas').first().boundingBox();
        await page.mouse.move(box.x + box.width * .45, box.y + box.height * .5);
        await page.mouse.down();
        await page.mouse.move(box.x + box.width * .75, box.y + box.height * .5, { steps: 50 });
        await page.mouse.up();
    } },
    { id: 'amigos', route: '/friends', seconds: 9, action: async page => {
        await page.getByRole('heading', { name: 'Amigos', exact: true }).waitFor();
        await page.getByRole('button', { name: /^Online/ }).first().click();
        await pause(1800);
        await page.getByRole('button', { name: /^Todos/ }).first().click();
        await pause(1400);
        await page.getByPlaceholder(/Buscar amigos/).first().fill('Alex');
    } },
    { id: 'configuracoes', route: '/settings', seconds: 9, action: async page => {
        const heading = page.getByText('Experiência do launcher', { exact: true });
        if (await heading.count()) await heading.scrollIntoViewIfNeeded();
        else await page.locator('main').evaluate(node => { node.scrollTop = 620; });
        await pause(2200);
        await page.mouse.wheel(0, 260);
    } },
    { id: 'bedrock', route: '/bedrock/bedrock-fixture', seconds: 9, action: async page => {
        await page.getByRole('heading', { name: 'Bedrock 1.26.52', exact: true }).waitFor();
        await page.getByRole('button', { name: /Pacotes de recursos/ }).click();
        await pause(1900);
        await page.getByRole('button', { name: /Mundos/ }).click();
    } }
];

(async () => {
    fs.mkdirSync(path.join(output, 'raw'), { recursive: true });
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const manifest = { applicationVersion: '3.6.0', releaseRevision: 2, provenance: 'Current launcher frontend with isolated demonstration data; no real account, game or downloads changed.', screenshots: [], shots: [] };
    try {
        const imagePage = await browser.newPage();
        for (const name of ['home', 'mods', 'instances', 'friends', 'personalizer']) {
            const svg = fs.readFileSync(`website/assets/captures-3.6/${name}-window.svg`);
            const header = svg.toString().match(/width="(\d+)" height="(\d+)"/);
            const width = Number(header[1]), height = Number(header[2]);
            await imagePage.setViewportSize({ width, height });
            await imagePage.setContent(`<style>html,body{margin:0}</style><img width="${width}" height="${height}" src="data:image/svg+xml;base64,${svg.toString('base64')}">`);
            await imagePage.locator('img').evaluate(image => image.decode());
            const file = path.join(output, `foto-${name}.png`);
            await imagePage.locator('img').screenshot({ path: file });
            manifest.screenshots.push({ name, file, width, height, provenance: 'User-provided screenshot rendered through the published window viewport; taskbar excluded.' });
        }
        await imagePage.close();
        for (const shot of shots) {
            const context = await browser.newContext({ viewport: { width: 1920, height: 1080 }, locale: 'pt-BR', reducedMotion: 'no-preference', recordVideo: { dir: path.join(output, 'raw'), size: { width: 1920, height: 1080 } } });
            const page = await context.newPage();
            const errors = [];
            page.on('pageerror', error => errors.push(String(error)));
            const started = Date.now();
            await setupLauncherDemo(page);
            await page.addInitScript(() => {
                const state = window.launcherDemo.state;
                Object.assign(state.settings, { accentTheme: 'gold', motionIntensity: 'cinematic', motionSpeed: 'balanced', respectReducedMotion: false });
                localStorage.setItem('luxmc_accent', 'gold');
                const original = window.electronAPI.invoke;
                const invoke = async (command, args) => {
                    if (command === 'bedrock_state') return { supported: true, provider: null, warning: null, instances: [{ id: 'bedrock-fixture', name: 'Bedrock 1.26.52', profileId: 'managed', installationId: 'package' }], installations: [{ id: 'package', profileId: 'managed', profileName: 'Luxmc', name: 'Bedrock', version: '1.26.52.3', directory: 'C:/Luxmc/Demonstracao/bedrock' }] };
                    if (command === 'bedrock_instance_content') return { directory: 'C:/Luxmc/Demonstracao/bedrock', isolated: true, worldAccounts: [], worldAccount: null, entries: [{ name: 'shared/resource', title: 'Meu pacote de recursos', kind: 'resources', icon: '/grass_block.png' }, { name: 'shared/world', title: 'Meu mundo', kind: 'worlds', icon: '/grass_block.png' }] };
                    return original(command, args);
                };
                window.electronAPI.invoke = invoke;
                window.__TAURI_INTERNALS__.invoke = invoke;
            });
            await page.goto(`http://127.0.0.1:1420${shot.route}`, { waitUntil: 'networkidle' });
            await page.locator('main').waitFor();
            await page.waitForTimeout(1200);
            await page.addStyleTag({ content: '.capture-cursor{position:fixed;width:15px;height:15px;border:2px solid rgb(var(--fg));border-radius:50%;background:rgb(var(--brand-500));box-shadow:0 0 0 5px rgb(var(--brand-500)/.16);pointer-events:none;z-index:2147483647;transform:translate(-50%,-50%)}' });
            await page.evaluate(() => {
                const cursor = document.createElement('div');
                cursor.className = 'capture-cursor';
                cursor.style.left = '1740px'; cursor.style.top = '960px';
                document.body.append(cursor);
                document.addEventListener('mousemove', event => { cursor.style.left = `${event.clientX}px`; cursor.style.top = `${event.clientY}px`; });
            });
            const from = (Date.now() - started) / 1000;
            const captureStart = Date.now();
            await shot.action(page);
            await pause(Math.max(1000, shot.seconds * 1000 - (Date.now() - captureStart)));
            await page.screenshot({ path: path.join(output, `${shot.id}-preview.png`) });
            const video = page.video();
            await context.close();
            const raw = path.join(output, 'raw', `${shot.id}.webm`);
            await video.saveAs(raw);
            await video.delete();
            const file = path.join(output, `${shot.id}.mp4`);
            const encoded = spawnSync(ffmpeg, ['-y', '-hide_banner', '-loglevel', 'error', '-ss', String(from), '-i', raw, '-t', String(shot.seconds), '-an', '-vf', 'fps=30', '-c:v', 'libx264', '-preset', 'veryfast', '-crf', '18', '-threads', '2', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', file], { windowsHide: true, encoding: 'utf8' });
            if (encoded.status !== 0) throw Error(encoded.stderr || `${shot.id}: encode failed`);
            if (errors.length) throw Error(`${shot.id}: ${errors.join('; ')}`);
            manifest.shots.push({ id: shot.id, file, seconds: shot.seconds, width: 1920, height: 1080, fps: 30, demonstrationData: true, errors });
            fs.writeFileSync(path.join(output, 'manifest.json'), JSON.stringify(manifest, null, 2));
            console.log(`Captured ${shot.id}: ${shot.seconds}s, 1920×1080, 30 fps.`);
        }
    } finally {
        await browser.close();
    }
})().catch(error => { console.error(error); process.exitCode = 1; });
