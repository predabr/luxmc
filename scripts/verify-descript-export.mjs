import { readFileSync, writeFileSync, statSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { resolve, dirname, basename, join } from 'node:path';
import assert from 'node:assert/strict';

const ffmpeg = join(process.env.LOCALAPPDATA, 'Luxmc-video-tools/imageio/imageio_ffmpeg/binaries/ffmpeg-win-x86_64-v7.1.exe');
const ffprobe = join(process.env.LOCALAPPDATA, 'Luxmc-video-tools/ffprobe.exe');
const run = (binary, args) => {
    const result = spawnSync(binary, args, { windowsHide: true, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
    if (result.status !== 0) throw Error(result.stderr || result.error?.message || 'Media verification failed.');
    return result;
};

for (const file of process.argv.slice(2)) {
    const source = resolve(file);
    const stem = basename(source, '.mp4');
    const output = dirname(source);
    const metadata = JSON.parse(run(ffprobe, ['-v', 'error', '-show_entries', 'format=duration,size:stream=index,codec_name,codec_type,width,height,r_frame_rate,sample_rate,channels', '-of', 'json', source]).stdout);
    const video = metadata.streams.find(stream => stream.codec_type === 'video');
    const audio = metadata.streams.find(stream => stream.codec_type === 'audio');
    assert.ok(video && audio);
    const duration = Number(metadata.format.duration);
    assert.ok(duration > 20 && duration < 110);
    const vertical = video.height > video.width;
    const dimensions = vertical ? '270:480' : '480:270';
    run(ffmpeg, ['-hide_banner', '-loglevel', 'error', '-i', source, '-f', 'null', '-']);
    const sheet = join(output, `${stem} - revisão.png`);
    run(ffmpeg, ['-y', '-hide_banner', '-loglevel', 'error', '-i', source, '-vf', `fps=1/${duration / 12},scale=${dimensions},tile=4x3`, '-frames:v', '1', sheet]);
    const moments = [2, duration * .25, duration * .5, duration * .75, duration - 2];
    const frames = [];
    for (let index = 0; index < moments.length; index++) {
        const frame = join(output, `${stem} - quadro ${index + 1}.png`);
        run(ffmpeg, ['-y', '-hide_banner', '-loglevel', 'error', '-ss', String(moments[index]), '-i', source, '-frames:v', '1', frame]);
        frames.push({ second: moments[index], frame });
    }
    const measured = run(ffmpeg, ['-hide_banner', '-i', source, '-af', 'ebur128=peak=true', '-f', 'null', '-']);
    const loudness = measured.stderr.slice(measured.stderr.lastIndexOf('Summary:'));
    const result = { file: source, bytes: statSync(source).size, duration, vertical, video, audio, fullDecodePassed: true, contactSheet: sheet, frames, loudness };
    writeFileSync(join(output, `${stem} - verificação.json`), JSON.stringify(result, null, 2));
    console.log(JSON.stringify({ file: source, duration, width: video.width, height: video.height, fullDecodePassed: true, loudness }));
}
