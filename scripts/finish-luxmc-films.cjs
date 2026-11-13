const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { chromium } = require('playwright-core');

const root = path.resolve('docs/media/2026-10-10');
const work = path.join(root, 'finishing');
const exportsDirectory = path.join(root, 'exports');
const ffmpeg = path.join(process.env.LOCALAPPDATA, 'Luxmc-video-tools/imageio/imageio_ffmpeg/binaries/ffmpeg-win-x86_64-v7.1.exe');
const node = process.execPath;
const sourceCss = fs.readFileSync('website/design.css', 'utf8');
const tokens = Object.fromEntries(['bg', 'bg-elevated', 'fg', 'fg-muted', 'brand-400', 'brand-500', 'border'].map(name => [name, sourceCss.match(new RegExp(`--${name}: ([\\d ]+);`))[1]]));
const color = name => `rgb(${tokens[name]})`;
const ffColor = name => `0x${tokens[name].split(' ').map(value => Number(value).toString(16).padStart(2, '0')).join('')}`;
const uri = (file, type) => `data:${type};base64,${fs.readFileSync(file).toString('base64')}`;
const escaped = value => value.replace(/[&<>]/g, symbol => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[symbol]));
const media = id => path.join(root, 'captures', `${id}.mp4`);
const photo = id => path.join(root, 'captures', `foto-${id}.png`);
const run = args => {
    const result = spawnSync(ffmpeg, ['-y', '-hide_banner', ...args], { encoding: 'utf8', windowsHide: true, maxBuffer: 32 * 1024 * 1024 });
    if (result.status !== 0) throw Error(result.stderr || result.error?.message || 'Rendering failed.');
    return result;
};
const horizontal = [
    { start: 0, end: 6.666667, source: media('biblioteca'), title: 'Seu próximo<br>mundo.', hero: true, crop: [390, 0, 1350, 1080], label: 'LUXMC 3.6', detail: 'Organize. Personalize. Explore.' },
    { start: 6.666667, end: 10.8, source: media('biblioteca'), title: 'Cada mundo, no seu lugar.', crop: [410, 30, 1330, 980], label: '01 / BIBLIOTECA' },
    { start: 10.8, end: 15.933333, source: path.join(root, 'captures/raw/criar-instancia.webm'), seek: 11.25, title: 'Uma instância.<br>Do seu jeito.', hero: true, crop: [612, 0, 696, 1080], narrow: true, label: '02 / JAVA + BEDROCK', detail: 'Versões à sua escolha.<br>Vanilla e loaders.<br>Configurações separadas.' },
    { start: 15.933333, end: 25.266667, source: media('modpacks'), title: 'Explore uma nova experiência.', crop: [270, 145, 1320, 855], label: '03 / MODRINTH + CURSEFORGE' },
    { start: 25.266667, end: 33.7, source: media('conteudos'), title: 'Tudo no lugar certo.', crop: [250, 10, 1360, 1010], label: '04 / MODS · RECURSOS · SHADERS' },
    { start: 33.7, end: 42.9, source: media('personalizador'), title: 'O visual também é seu.', crop: [250, 70, 1450, 965], label: '05 / PERSONALIZAÇÃO' },
    { start: 42.9, end: 46.933333, source: media('amigos'), seek: 1.4, title: 'Seu perfil. Sua turma.', crop: [305, 30, 1570, 830], label: '06 / AMIGOS' },
    { start: 46.933333, end: 51.433333, source: media('configuracoes'), title: 'Controle a sua experiência.', crop: [540, 10, 1180, 1050], label: '07 / AJUSTES' },
    { start: 51.433333, end: 61.833333, source: media('bedrock'), title: 'Bedrock, na sua biblioteca.', crop: [255, 10, 1400, 990], label: '08 / MINECRAFT PARA WINDOWS', note: 'Minecraft para Windows requer licença.' },
    { start: 61.833333, end: 72.166667, source: photo('home'), title: 'Seu próximo mundo<br>começa aqui.', cta: true, label: 'LUXMC 3.6' }
];
const vertical = [
    { start: 0, end: 3.8, source: media('biblioteca'), title: 'Seu próximo<br>mundo.', crop: [360, 10, 920, 1070], label: 'LUXMC 3.6' },
    { start: 3.8, end: 6.7, source: path.join(root, 'captures/raw/criar-instancia.webm'), seek: 11.25, title: 'Escolha sua<br>versão.', crop: [612, 0, 696, 1080], narrow: true, label: 'JAVA + BEDROCK' },
    { start: 6.7, end: 11.733333, source: media('modpacks'), title: 'Mude a<br>experiência.', crop: [370, 165, 910, 870], label: 'MODPACKS' },
    { start: 11.733333, end: 15.6, source: media('conteudos'), title: 'Tudo no<br>lugar certo.', crop: [350, 15, 965, 1050], label: 'MODS · RECURSOS · SHADERS' },
    { start: 15.6, end: 19.766667, source: media('personalizador'), title: 'O visual<br>também é seu.', crop: [255, 120, 525, 920], label: 'SKINS EM 3D' },
    { start: 19.766667, end: 23.2, source: media('amigos'), seek: 1.5, title: 'Seu perfil.<br>Sua turma.', crop: [310, 20, 1060, 1000], label: 'AMIGOS' },
    { start: 23.2, end: 30.233333, source: photo('home'), title: 'Seu próximo<br>mundo.', cta: true, label: 'LUXMC 3.6' }
];

function geometry(scene, portrait) {
    if (scene.cta) return null;
    if (portrait) return scene.narrow ? [170, 354, 740, 1148] : [72, 366, 936, 1140];
    if (scene.hero) return scene.narrow ? [1110, 60, 620, 962] : [900, 196, 900, 720];
    return [112, 162, 1696, 800];
}

function artwork(scene, portrait) {
    const [width, height] = portrait ? [1080, 1920] : [1920, 1080];
    const box = geometry(scene, portrait);
    const layout = portrait ? 'portrait' : 'landscape';
    const isHero = scene.hero || scene.cta;
    const logo = uri('static/logo.png', 'image/png');
    const font = uri('website/assets/fonts/space-grotesk-latin.woff2', 'font/woff2');
    const mono = uri('website/assets/fonts/plex-mono-latin.woff2', 'font/woff2');
    const vars = Object.entries(tokens).map(([name, value]) => `--${name}:${value}`).join(';');
    const frame = box ? `<div class="frame" style="left:${box[0] - 2}px;top:${box[1] - 2}px;width:${box[2] + 4}px;height:${box[3] + 4}px"></div>` : '';
    const hero = scene.cta ? `<div class="cta"><img src="${logo}"><div class="eyebrow">LUXMC 3.6</div><h1>${scene.title}</h1><div class="download">Baixe Lux MC Launcher <span>↗</span></div><div class="url">luxmc-r92.pages.dev</div></div>` : scene.hero && !portrait ? `<div class="hero"><img src="${logo}"><div class="eyebrow">${scene.label}</div><h1>${scene.title}</h1><p>${scene.detail}</p></div>` : `<div class="head"><div class="eyebrow">${scene.label}</div><h1>${scene.title}</h1></div><img class="mark" src="${logo}">`;
    return `<style>@font-face{font-family:Lux;src:url(${font})}@font-face{font-family:Mono;src:url(${mono})}*{box-sizing:border-box}html,body{margin:0;width:${width}px;height:${height}px;overflow:hidden}body{${vars};font-family:Lux,Arial;color:rgb(var(--fg));background:rgb(var(--bg))}.stage{position:relative;width:100%;height:100%;background:radial-gradient(ellipse at 92% 0%,rgb(var(--brand-500)/.18),transparent 48%),radial-gradient(ellipse at 0% 100%,rgb(var(--brand-400)/.06),transparent 52%),linear-gradient(145deg,rgb(var(--bg)),rgb(var(--bg-elevated)))}.grid{position:absolute;inset:0;opacity:.27;background-image:linear-gradient(rgb(var(--border)/.55) 1px,transparent 1px),linear-gradient(90deg,rgb(var(--border)/.55) 1px,transparent 1px);background-size:108px 108px;mask-image:linear-gradient(150deg,black,transparent 65%)}.line{position:absolute;left:0;top:0;height:5px;width:100%;background:linear-gradient(90deg,rgb(var(--brand-400)),rgb(var(--brand-500)/.06))}.frame{position:absolute;border:2px solid rgb(var(--brand-400)/.5);border-radius:4px;background:rgb(var(--bg));box-shadow:0 24px 100px rgb(var(--bg)/.9),0 0 48px rgb(var(--brand-500)/.08)}.eyebrow{font:500 20px Mono,monospace;letter-spacing:4px;color:rgb(var(--brand-400));margin-bottom:12px}.head{position:absolute;left:112px;top:40px}h1{margin:0;font-size:56px;line-height:1.06;letter-spacing:-2px;font-weight:700}.mark{position:absolute;width:94px;height:94px;object-fit:contain;right:112px;top:34px}.hero{position:absolute;left:110px;top:148px;width:690px}.hero img{width:130px;height:130px;object-fit:contain;margin-bottom:28px}.hero h1{font-size:108px;letter-spacing:-5px;line-height:1.02;margin-top:26px}.hero p{font-size:30px;line-height:1.6;color:rgb(var(--fg-muted));margin-top:36px}.footer{position:absolute;left:112px;bottom:29px;font:15px Mono,monospace;color:rgb(var(--fg-muted));letter-spacing:1px}.signature{position:absolute;right:112px;bottom:29px;font:15px Mono;color:rgb(var(--brand-400))}.cta{position:absolute;inset:90px 120px 120px;text-align:center}.cta img{width:150px;height:150px;object-fit:contain;margin-bottom:20px}.cta .eyebrow{font-size:28px}.cta h1{font-size:100px;line-height:1.03;letter-spacing:-4px;margin:28px 0 42px}.download{display:inline-block;padding:20px 36px;background:rgb(var(--brand-400));border-radius:18px;color:rgb(var(--bg));font-size:31px;font-weight:700;box-shadow:0 12px 46px rgb(var(--brand-500)/.18)}.download span{margin-left:34px}.url{font:24px Mono;letter-spacing:1px;margin-top:30px;color:rgb(var(--fg-muted))}.note{position:absolute;left:112px;bottom:98px;padding:7px 12px;background:rgb(var(--bg)/.95);border-radius:8px;font-size:18px;color:rgb(var(--fg-muted))}.portrait .head{left:72px;top:140px;width:825px}.portrait h1{font-size:84px;letter-spacing:-3px}.portrait .eyebrow{font-size:20px;margin-bottom:18px}.portrait .mark{width:80px;height:80px;right:73px;top:136px}.portrait .footer{left:72px;bottom:144px;font-size:17px;letter-spacing:0}.portrait .signature{right:72px;bottom:106px;font-size:17px}.portrait .cta{inset:258px 76px 300px}.portrait .cta img{width:230px;height:230px;margin-bottom:56px}.portrait .cta .eyebrow{font-size:26px}.portrait .cta h1{font-size:106px;margin:38px 0 64px;letter-spacing:-5px}.portrait .download{font-size:29px;padding:26px 28px}.portrait .url{font-size:29px;margin-top:42px}.portrait .cta h1{line-height:1.02}</style><div class="stage ${layout}"><div class="grid"></div><div class="line"></div>${frame}${hero}<div class="footer">Interface gravada com dados de demonstração.</div><div class="signature">LUXMC / 3.6</div>${scene.note ? `<div class="note">${scene.note}</div>` : ''}</div>`;
}

function ass(srtFile, output, portrait) {
    const data = fs.readFileSync(srtFile, 'utf8');
    const seconds = value => { const [h, m, s] = value.replace(',', '.').split(':').map(Number); return h * 3600 + m * 60 + s; };
    const time = value => `${Math.floor(value / 3600)}:${String(Math.floor(value / 60) % 60).padStart(2, '0')}:${(value % 60).toFixed(2).padStart(5, '0')}`;
    const events = [];
    const max = portrait ? 30 : 61;
    for (const block of data.trim().split(/\r?\n\r?\n/)) {
        const lines = block.split(/\r?\n/);
        const times = lines[1].split(' --> ');
        const text = lines.slice(2).join(' ').trim();
        if (text === 'O') continue;
        const parts = [];
        let line = '';
        for (const word of text.split(' ')) {
            if ((line + ' ' + word).trim().length > max && line) { parts.push(line); line = word; }
            else line = (line + ' ' + word).trim();
        }
        if (line) parts.push(line);
        const start = text === 'visual também é seu.' ? 33.3 : seconds(times[0]);
        const content = text === 'visual também é seu.' ? 'O visual também é seu.' : parts.join('\\N');
        events.push(`Dialogue: 0,${time(start)},${time(seconds(times[1]))},Default,,0,0,0,,{\\fad(80,80)}${content}`);
    }
    const dimensions = portrait ? [1080, 1920] : [1920, 1080];
    fs.writeFileSync(output, `[Script Info]\nScriptType: v4.00+\nPlayResX: ${dimensions[0]}\nPlayResY: ${dimensions[1]}\nWrapStyle: 2\nScaledBorderAndShadow: yes\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,${portrait ? 48 : 38},&H00FFFFFF,&H00FFFFFF,&H00101012,&HAA101012,-1,0,0,0,100,100,0,0,1,3,1,2,${portrait ? 100 : 170},${portrait ? 150 : 170},${portrait ? 273 : 59},1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n${events.join('\n')}\n`);
}

async function renderFilm(portrait, page) {
    const title = portrait ? 'Shorts' : 'Filme';
    const scenes = portrait ? vertical : horizontal;
    const size = portrait ? [1080, 1920] : [1920, 1080];
    const chunks = [];
    const duration = scenes.at(-1).end;
    await page.setViewportSize({ width: size[0], height: size[1] });
    for (let index = 0; index < scenes.length; index++) {
        const scene = scenes[index];
        const seconds = scene.end - scene.start;
        const frame = path.join(work, `${title}-${index}.png`);
        await page.setContent(artwork(scene, portrait));
        await page.evaluate(() => document.fonts.ready);
        await page.locator('img').first().evaluate(image => image.decode());
        await page.screenshot({ path: frame });
        const destination = path.join(work, `${title}-${index}.mp4`);
        chunks.push(destination);
        if (process.argv.includes('--artwork-only')) continue;
        if (process.argv.includes('--mux-only') && fs.existsSync(destination)) continue;
        if (process.argv.includes('--resume') && fs.existsSync(destination) && !(portrait ? [0, 1] : [0, 2]).includes(index)) continue;
        const still = scene.source.endsWith('.png');
        const args = ['-loglevel', 'error', '-loop', '1', '-framerate', '30', '-i', frame];
        if (still) args.push('-loop', '1', '-framerate', '30');
        if (scene.seek) args.push('-ss', String(scene.seek));
        args.push('-i', scene.source);
        const box = geometry(scene, portrait);
        const intro = Math.min(.24, seconds / 8);
        const crop = scene.crop ? [scene.crop[2], scene.crop[3], scene.crop[0], scene.crop[1]].join(':') : null;
        let graph;
        if (box) {
            const movement = `crop=iw/1.025:ih/1.025:x=(iw-ow)*(0.3+0.3*t/${seconds}):y=(ih-oh)*0.5,scale=${box[2]}:${box[3]}`;
            graph = `[0:v]format=yuv420p,setsar=1[base];[1:v]crop=${crop},scale=${box[2]}:${box[3]}:force_original_aspect_ratio=increase,crop=${box[2]}:${box[3]},${movement},setsar=1,format=yuva420p,tpad=stop_mode=clone:stop_duration=${seconds},fade=t=in:st=0:d=${intro}:alpha=1[clip];[base][clip]overlay=x='${box[0]}+36*exp(-t*16)':y=${box[1]}:shortest=1,fade=t=in:st=0:d=0.12,drawbox=x=0:y=ih-5:w=iw:h=5:color=${ffColor('brand-500')}@.22:t=fill[out]`;
        } else {
            graph = `[1:v]scale=${size[0]}:${size[1]}:force_original_aspect_ratio=increase,crop=${size[0]}:${size[1]},gblur=sigma=28,eq=brightness=-.25:saturation=.25,format=yuv420p[photo];[0:v]format=rgba,colorchannelmixer=aa=.92[art];[photo][art]overlay=0:0,fade=t=in:st=0:d=0.22[out]`;
        }
        args.push('-filter_complex_threads', '1', '-filter_complex', graph, '-map', '[out]', '-an', '-t', String(seconds), '-r', '30', '-c:v', 'libx264', '-preset', 'fast', '-crf', '18', '-threads', '2', '-pix_fmt', 'yuv420p', destination);
        run(args);
        console.log(`${title}: ${index + 1}/${scenes.length}, ${seconds.toFixed(2)}s.`);
    }
    if (process.argv.includes('--artwork-only')) return;
    const list = path.join(work, `${title}.txt`);
    fs.writeFileSync(list, chunks.map(file => `file '${path.basename(file)}'`).join('\n'));
    const joined = path.join(work, `${title}-joined.mp4`);
    run(['-loglevel', 'error', '-f', 'concat', '-safe', '0', '-i', list, '-c', 'copy', joined]);
    const original = path.join(exportsDirectory, `Lux MC Launcher 3.6 - ${title} Oficial.mp4`);
    const normalized = path.join(work, `${title}-mix.wav`);
    const filter = 'loudnorm=I=-16:TP=-1.5:LRA=7';
    const measurement = run(['-i', original, '-vn', '-af', `${filter}:print_format=json`, '-f', 'null', '-']).stderr;
    const values = JSON.parse(measurement.match(/\{\s*"input_i"[^}]+\}/)[0]);
    const normalize = `${filter}:measured_I=${values.input_i}:measured_TP=${values.input_tp}:measured_LRA=${values.input_lra}:measured_thresh=${values.input_thresh}:offset=${values.target_offset}:linear=true,afade=t=in:d=0.18,afade=t=out:st=${duration - 1.25}:d=1.25`;
    run(['-loglevel', 'error', '-i', original, '-vn', '-af', normalize, '-ar', '48000', '-c:a', 'pcm_s24le', normalized]);
    const captions = path.join(work, `${title}.ass`);
    ass(path.join(exportsDirectory, portrait ? 'shorts.srt' : 'filme.srt'), captions, portrait);
    const assFilter = path.relative(process.cwd(), captions).replaceAll('\\', '/');
    const final = path.join(exportsDirectory, `Lux MC Launcher 3.6 - ${title} Final.mp4`);
    run(['-loglevel', 'error', '-i', joined, '-i', normalized, '-map', '0:v:0', '-map', '1:a:0', '-vf', `ass='${assFilter}',fade=t=out:st=${duration - .35}:d=0.35`, '-t', String(duration), '-r', '30', '-c:v', 'libx264', '-preset', 'fast', '-crf', '18', '-threads', '2', '-c:a', 'aac', '-b:a', '256k', '-ar', '48000', '-movflags', '+faststart', final]);
    fs.writeFileSync(path.join(work, `${title}-mix.json`), JSON.stringify({ source: original, description: 'Descript narration and stock music mix; timing preserved. No video from the watermarked preview used.', measuredInput: values, targetIntegratedLoudness: -16, targetTruePeak: -1.5, dimensions: size, duration, final }, null, 2));
    console.log(`Completed: ${final}`);
}

(async () => {
    fs.mkdirSync(work, { recursive: true });
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const page = await browser.newPage();
    try {
        if (!process.argv.includes('--portrait-only')) await renderFilm(false, page);
        await renderFilm(true, page);
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
