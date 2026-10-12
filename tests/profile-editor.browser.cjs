const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const { validatePublicProfile } = await import('../website/lib/public-profile.js');
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = 'docs/validation/profile-editor';
    fs.mkdirSync(output, { recursive: true });
    const results = [];
    try {
        for (const theme of ['dark', 'light']) {
            const page = await browser.newPage({ viewport: { width: 1440, height: 1050 }, reducedMotion: 'reduce' });
            const errors = [];
            page.on('pageerror', error => errors.push(error.message));
            await setupLauncherDemo(page);
            await page.addInitScript(theme => {
                const state = window.launcherDemo.state;
                state.settings.theme = `default-${theme}`;
                localStorage.setItem('luxmc_theme', theme);
                window.profileFixture = { id: state.account.uuid, username: 'LuxPlayer', role: 'owner', tags: ['Testador'], membership: { plan: 'pro', expiresAt: 1893456000 }, avatarUrl: '/grass_head.png', displayName: 'Lux Creator', status: 'Construindo mundos', description: '', banner: '', portrait: '', packs: [], collections: [] };
                window.profileSaves = [];
                window.rejectProfileSave = false;
                const invoke = window.electronAPI.invoke;
                window.electronAPI.invoke = async (command, args = {}) => {
                    if (command === 'social_request' && args.request.action === 'profile_get') return { profile: args.request.targetId ? { ...window.profileFixture, id: args.request.targetId, username: 'AlexBuilder', displayName: 'AlexBuilder', role: 'member' } : window.profileFixture };
                    if (command === 'social_request' && args.request.action === 'profile_save') {
                        if (window.rejectProfileSave) throw new Error('Falha de conexão simulada');
                        window.profileSaves.push(structuredClone(args.request));
                        Object.assign(window.profileFixture, args.request);
                        return { ok: true };
                    }
                    const result = await invoke(command, args);
                    if (command === 'social_request' && ['register', 'sync'].includes(args.request.action)) Object.assign(result.me, { role: 'owner', membership: { plan: 'pro', expiresAt: 1893456000 } });
                    return result;
                };
            }, theme);
            await page.goto('http://127.0.0.1:1420/friends');
            await page.getByRole('button', { name: 'Editar meu perfil', exact: true }).click();
            const dialog = page.getByRole('dialog');
            await dialog.getByLabel('Nome de exibição', { exact: true }).fill('Explorer');
            await dialog.getByLabel('Status do perfil', { exact: true }).fill('Explorando com meus amigos');
            await dialog.getByRole('button', { name: 'Meu estilo de jogo', exact: true }).click();
            await dialog.getByRole('button', { name: 'Fechar', exact: true }).click();
            await page.getByRole('alertdialog').waitFor();
            await page.getByRole('alertdialog').getByRole('button', { name: 'Continuar editando', exact: true }).click();
            assert.equal(await dialog.getByLabel('Nome de exibição', { exact: true }).inputValue(), 'Explorer');
            await dialog.screenshot({ path: `${output}/informacoes-${theme}.png` });

            await dialog.getByRole('tab', { name: 'Aparência', exact: true }).click();
            await dialog.getByRole('button', { name: 'Aplicar capa Aurora', exact: true }).click();
            const png = await page.evaluate(() => {
                const canvas = document.createElement('canvas'); canvas.width = 8000; canvas.height = 700;
                const context = canvas.getContext('2d'); context.fillStyle = 'steelblue'; context.fillRect(0, 0, 4000, 700); context.fillStyle = 'goldenrod'; context.fillRect(4000, 0, 4000, 700);
                return canvas.toDataURL('image/png').split(',')[1];
            });
            await dialog.getByLabel('Selecionar avatar do perfil', { exact: true }).setInputFiles({ name: 'avatar.png', mimeType: 'image/png', buffer: Buffer.from(png, 'base64') });
            await dialog.getByRole('button', { name: 'Aplicar enquadramento', exact: true }).waitFor();
            assert.equal(await dialog.getByRole('button', { name: 'Salvar perfil', exact: true }).isEnabled(), false);
            const sliders = dialog.getByRole('slider');
            await sliders.nth(0).focus(); await sliders.nth(0).press('End');
            await sliders.nth(1).focus(); await sliders.nth(1).press('End');
            await dialog.screenshot({ path: `${output}/enquadramento-${theme}.png` });
            await dialog.getByRole('button', { name: 'Aplicar enquadramento', exact: true }).click();
            await dialog.getByRole('button', { name: 'Aplicar enquadramento', exact: true }).waitFor({ state: 'hidden' });
            await dialog.screenshot({ path: `${output}/aparencia-${theme}.png` });

            await dialog.getByRole('tab', { name: 'Vitrine', exact: true }).click();
            await dialog.getByRole('button', { name: /^Vanilla Perfected / }).click();
            await dialog.getByLabel('Adicionar mod ou modpack', { exact: true }).fill('Cobblemon');
            await dialog.getByLabel('Adicionar mod ou modpack', { exact: true }).press('Enter');
            await dialog.getByRole('button', { name: 'Mover Cobblemon para cima', exact: true }).click();
            await dialog.getByLabel('Nome da coleção', { exact: true }).fill('Nossa turma');
            await dialog.getByLabel('Nome da coleção', { exact: true }).locator('..').getByRole('button', { name: 'Adicionar', exact: true }).click();
            await dialog.screenshot({ path: `${output}/vitrine-${theme}.png` });
            await dialog.getByRole('button', { name: 'Ver prévia', exact: true }).click();
            await dialog.getByRole('heading', { name: 'Explorer', exact: true }).waitFor();
            assert.equal(await dialog.getByRole('textbox').count(), 0);
            await dialog.screenshot({ path: `${output}/previa-${theme}.png` });
            await page.evaluate(() => window.rejectProfileSave = true);
            await dialog.getByRole('button', { name: 'Salvar perfil', exact: true }).click();
            await dialog.getByRole('alert').filter({ hasText: 'Falha de conexão simulada' }).waitFor();
            await dialog.getByRole('button', { name: 'Continuar editando', exact: true }).click();
            assert.equal(await dialog.getByRole('tab', { name: 'Vitrine', exact: true }).getAttribute('aria-selected'), 'true');
            await page.evaluate(() => window.rejectProfileSave = false);
            await page.keyboard.press('Control+s');
            await dialog.getByText('Perfil salvo. Suas alterações já estão disponíveis para seus amigos.', { exact: true }).waitFor();
            const firstSave = await page.evaluate(() => window.profileSaves[0]);
            assert.ok(validatePublicProfile(firstSave, 24));
            assert.deepEqual(firstSave.packs, ['Cobblemon', 'Vanilla Perfected']);
            assert.equal(firstSave.collections[0].title, 'Nossa turma');
            assert.ok(firstSave.banner.startsWith('data:image/webp;'));
            assert.ok(firstSave.portrait.startsWith('data:image/webp;'));
            const portraitColor = await page.evaluate(async value => {
                const bitmap = await createImageBitmap(await (await fetch(value)).blob());
                const canvas = document.createElement('canvas'); canvas.width = 1; canvas.height = 1;
                const context = canvas.getContext('2d'); context.drawImage(bitmap, 0, 0, 1, 1); bitmap.close();
                return Array.from(context.getImageData(0, 0, 1, 1).data);
            }, firstSave.portrait);
            assert.ok(portraitColor[0] > portraitColor[2] + 70);
            await dialog.getByRole('button', { name: 'Editar meu perfil', exact: true }).click();
            await dialog.getByRole('tab', { name: 'Aparência', exact: true }).click();
            const gif = Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7', 'base64');
            await dialog.getByLabel('Selecionar avatar do perfil', { exact: true }).setInputFiles({ name: 'avatar.gif', mimeType: 'image/gif', buffer: gif });
            await dialog.getByText('GIF', { exact: true }).waitFor();
            assert.equal(await dialog.getByRole('button', { name: 'Aplicar enquadramento', exact: true }).count(), 0);
            await dialog.getByRole('button', { name: 'Salvar perfil', exact: true }).click();
            await dialog.getByText('Perfil salvo. Suas alterações já estão disponíveis para seus amigos.', { exact: true }).waitFor();
            assert.ok((await page.evaluate(() => window.profileFixture.portrait)).startsWith('data:image/gif;'));
            await dialog.getByRole('button', { name: 'Editar meu perfil', exact: true }).click();
            await dialog.getByLabel('Nome de exibição', { exact: true }).fill('Rascunho temporário');
            await dialog.getByRole('button', { name: 'Fechar', exact: true }).click();
            await page.getByRole('alertdialog').getByRole('button', { name: 'Descartar e fechar', exact: true }).click();
            await dialog.waitFor({ state: 'hidden' });
            await page.getByRole('button', { name: 'Editar meu perfil', exact: true }).click();
            assert.equal(await dialog.getByLabel('Nome de exibição', { exact: true }).inputValue(), 'Explorer');
            await page.setViewportSize({ width: 720, height: 900 });
            const overflow = await dialog.evaluate(element => element.scrollWidth > element.clientWidth + 1);
            assert.equal(overflow, false);
            await dialog.screenshot({ path: `${output}/compacto-${theme}.png` });
            assert.deepEqual(errors, []);
            results.push({ theme, saved: true, serverValidation: true, cropApplied: true, gifPreserved: true, unsavedGuard: true, failureRetainsDraft: true, favouritesOrdered: true, overflow, errors });
            await page.close();
        }
        fs.writeFileSync(`${output}/browser.json`, JSON.stringify({ passed: true, fixtureOnly: true, scenarios: results }, null, 2));
        console.log('Editor de perfil aprovado: persistência, imagens, GIF, enquadramento, prévia, ordenação e proteção do rascunho.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exit(1); });
