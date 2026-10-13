const { chromium } = require('playwright-core');
const { spawn } = require('node:child_process');
const { readFileSync, writeFileSync, existsSync } = require('node:fs');
const { resolve, join } = require('node:path');
const assert = require('node:assert/strict');

(async () => {
    const binary = process.env.LUXMC_NATIVE_BINARY;
    const archive = process.env.LUXMC_VALIDATION_ARCHIVE;
    const output = process.env.LUXMC_VALIDATION_REPORT;
    assert.ok(binary && archive && output);
    const port = Number(process.env.LUXMC_NATIVE_DEBUG_PORT || 9226);
    assert.ok(Number.isInteger(port) && port >= 1024 && port <= 65535);
    const report = { nativeTauriDispatch: true, phases: [] };
    const persist = () => writeFileSync(output, JSON.stringify(report, null, 2));
    const app = spawn(binary, [], {
        windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'],
        env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`, RUST_LOG: 'warn,luxmc_lib::commands::instances=info,luxmc_lib::core::mods::validation=info' }
    });
    let launchError;
    app.on('error', error => { launchError = error; });
    for (const stream of [app.stdout, app.stderr]) stream.on('data', chunk => {
        for (const line of chunk.toString().split('\n')) if (/Modpack registration phase completed|Slow archive validation/.test(line)) {
            report.phases.push(line.replace(/\u001b\[[0-9;]*m/g, ''));
            console.log(line.trim());
            persist();
        }
    });
    let browser;
    let page;
    let fixture;
    let initial;
    try {
        for (let n = 0; n < 60; n++) {
            if (launchError) throw launchError;
            assert.equal(app.exitCode, null, 'O launcher encerrou antes da conexão');
            try { browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: 1000 }); break; }
            catch { await new Promise(resolvePromise => setTimeout(resolvePromise, 500)); }
        }
        assert.ok(browser, 'WebView de teste indisponível');
        for (let n = 0; n < 60; n++) {
            page = browser.contexts()[0].pages().find(item => item.url().includes('tauri.localhost'));
            if (page) break;
            await new Promise(resolvePromise => setTimeout(resolvePromise, 500));
        }
        assert.ok(page, 'Janela Tauri ausente');
        await page.waitForFunction(() => Boolean(window.__TAURI_INTERNALS__), { timeout: 30000 });
        const call = (command, args = {}) => page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
        initial = await call('profiles_list');
        for (const name of ['RLCraft', 'All the Mods 10 - ATM10']) {
            const matches = initial.filter(profile => profile.name === name);
            assert.equal(matches.length, 1);
            const started = performance.now();
            await call('instance_repair', { profileId: matches[0].id });
            await call('instance_repair_modpack', { profileId: matches[0].id });
            assert.equal(app.exitCode, null, 'O launcher encerrou durante o reparo');
            report[name] = { repairedWithoutLauncherCrash: true, repairSeconds: +((performance.now() - started) / 1000).toFixed(2) };
            persist();
        }
        const started = performance.now();
        const imported = await call('instance_import_modpack', { filePath: archive, profileName: `Validação nativa temporária ${Date.now()}`, mcVersion: '1.21.1', loader: 'neoforge', ramMb: 6144 });
        fixture = imported.profile || imported;
        assert.ok(fixture.id && !initial.some(profile => profile.id === fixture.id));
        report.installSeconds = +((performance.now() - started) / 1000).toFixed(2);
        const root = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', fixture.id, '.minecraft');
        assert.equal(resolve(fixture.gameDir).toLowerCase(), root.toLowerCase());
        const manifest = JSON.parse(readFileSync(join(root, 'manifest.json'), 'utf8'));
        const cc = manifest.files.filter(file => [282001, 1676502].includes(file.projectID));
        assert.equal(cc.length, 1);
        assert.equal(cc[0].fileID, 8778582);
        report.duplicateResolvedDuringImport = true;
        report.launcherRemainedOpen = app.exitCode === null;
    } catch (error) { report.error = error.message; process.exitCode = 1; }
    finally {
        if (fixture && page) {
            try {
                assert.ok(initial && !initial.some(profile => profile.id === fixture.id));
                const root = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', fixture.id);
                assert.equal(resolve(fixture.gameDir).toLowerCase(), resolve(root, '.minecraft').toLowerCase());
                await page.evaluate(id => window.__TAURI_INTERNALS__.invoke('profiles_delete', { id }), fixture.id);
                assert.equal(existsSync(root), false);
                const final = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('profiles_list'));
                assert.deepEqual(final.map(profile => profile.id).sort(), initial.map(profile => profile.id).sort());
                report.originalInstancesPreserved = true;
            } catch (error) { report.cleanupError = error.message; process.exitCode = 1; }
        }
        persist();
        if (browser) await browser.close();
        if (app.exitCode === null) app.kill();
    }
})().catch(error => { console.error(error.message); process.exitCode = 1; });
