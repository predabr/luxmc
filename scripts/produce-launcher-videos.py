import argparse
import hashlib
import json
import math
import os
import re
import shutil
import subprocess
import time
import wave
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent
MEDIA = ROOT / "docs" / "media" / "2026-10-05"
WORK = MEDIA / "production"
EXPORT = MEDIA / "exports"
FONT_DIR = Path(os.environ.get("WINDIR", "C:/Windows")) / "Fonts"
FONT_REGULAR = FONT_DIR / "segoeui.ttf"
FONT_BOLD = FONT_DIR / "segoeuib.ttf"
FPS = 30
RATE = 44100
CREATE_NO_WINDOW = 0x08000000 if os.name == "nt" else 0
DEFAULTS = {
    "home": ("Sua biblioteca, do seu jeito", "Instâncias, versões e modpacks organizados em um só lugar."),
    "discover": ("Encontre sua próxima aventura", "Explore modpacks, mods, recursos e shaders com filtros de versão e loader."),
    "new-instance": ("Comece com a versão certa", "Crie uma instância Vanilla ou escolha o loader do seu modpack."),
    "instance": ("Um espaço para cada mundo", "Gerencie mods, recursos, backups e os detalhes de cada instância."),
    "skins": ("Sua identidade em movimento", "Visualize a skin em três dimensões e aplique sua aparência à conta."),
    "namemc": ("Uma skin nova, sem complicação", "Abra o NameMC, escolha uma skin e importe para a biblioteca."),
    "friends": ("Sua turma tem um lugar", "Busque amigos, envie pedidos e acompanhe as solicitações recebidas."),
    "p2p": ("Prepare a sala com seus amigos", "Compartilhe o convite da sala. Esta tela usa participantes de demonstração."),
    "settings": ("O controle fica com você", "Ajuste idioma, aparência, Java, privacidade e armazenamento nas configurações."),
    "appearance": ("Um launcher com a sua cara", "Personalize o fundo, a transparência e o movimento da interface."),
    "java": ("Preparação inteligente", "Defina a memória e o Java. O cache reaproveita bibliotecas já preparadas."),
    "storage": ("Saiba onde está cada arquivo", "Acompanhe o armazenamento e acesse os dados das suas instâncias."),
    "login": ("Entre com a sua conta", "Use a conta Microsoft ou uma conta Lux MC para acessar seu perfil."),
    "installer": ("Pronto para o Windows", "O instalador reúne o launcher e os arquivos necessários em uma instalação."),
}
SHORT_FOCUS = {
    "home": [260, 140, 1000, 830],
    "discover": [260, 230, 920, 800],
    "skins": [270, 220, 540, 760],
    "namemc": [710, 230, 740, 740],
    "friends": [250, 115, 1040, 860],
    "p2p": [450, 135, 1000, 835],
    "settings": [480, 135, 975, 810],
    "appearance": [500, 210, 920, 770],
}


def run(command, log=None):
    result = subprocess.run(command, capture_output=True, text=True, encoding="utf-8", errors="replace", creationflags=CREATE_NO_WINDOW)
    if log:
        Path(log).write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(result.stderr[-5000:] or result.stdout[-5000:])
    return result.stdout


def find_tools(custom):
    candidates = [Path(custom)] if custom else []
    command = shutil.which("ffmpeg")
    if command:
        candidates.append(Path(command))
    tools = Path(os.environ.get("LOCALAPPDATA", "C:/Users/preda/AppData/Local")) / "Luxmc-video-tools"
    if tools.exists():
        candidates.extend(tools.rglob("ffmpeg.exe"))
        candidates.extend(tools.rglob("ffmpeg-win*.exe"))
    ffmpeg = next((path for path in candidates if path.is_file()), None)
    if not ffmpeg:
        raise RuntimeError("FFmpeg is required; set --ffmpeg to a local executable")
    probes = [ffmpeg.with_name("ffprobe.exe")]
    if tools.exists():
        probes.extend(tools.rglob("ffprobe.exe"))
    ffprobe = next((path for path in probes if path.is_file()), None)
    return str(ffmpeg), str(ffprobe) if ffprobe else None


def font(size, bold=False):
    return ImageFont.truetype(str(FONT_BOLD if bold else FONT_REGULAR), size)


def lines(text, size, width, bold=False, max_lines=3):
    face = font(size, bold)
    words = str(text).split()
    out, current = [], ""
    for word in words:
        candidate = f"{current} {word}".strip()
        if face.getlength(candidate) > width and current:
            out.append(current)
            current = word
        else:
            current = candidate
    if current:
        out.append(current)
    if len(out) > max_lines:
        return lines(text, max(18, size - 2), width, bold, max_lines)
    return out, face


def text_block(draw, text, position, size, width, fill=(239, 244, 255, 255), bold=False, centered=False, max_lines=3):
    wrapped, face = lines(text, size, width, bold, max_lines)
    x, y = position
    spacing = int(size * 1.22)
    for index, line in enumerate(wrapped):
        left = x + (width - face.getlength(line)) / 2 if centered else x
        draw.text((left, y + index * spacing), line, font=face, fill=fill)
    return len(wrapped) * spacing


def background(size):
    width, height = size
    yy, xx = np.mgrid[0:height, 0:width].astype(np.float32)
    blue = np.exp(-(((xx - width * .12) / (width * .65)) ** 2 + ((yy - height * .22) / (height * .8)) ** 2))
    gold = np.exp(-(((xx - width * .88) / (width * .75)) ** 2 + ((yy - height * .9) / (height * .7)) ** 2))
    pixels = np.empty((height, width, 3), dtype=np.uint8)
    for channel, (base, a, b) in enumerate([(6, 12, 10), (10, 22, 8), (19, 48, 4)]):
        pixels[:, :, channel] = np.clip(base + a * blue + b * gold, 0, 255)
    im = Image.fromarray(pixels).convert("RGBA")
    grid = Image.new("RGBA", size)
    draw = ImageDraw.Draw(grid)
    gap = 96 if width > height else 90
    for xpos in range(0, width, gap):
        draw.line([(xpos, 0), (xpos, height)], fill=(154, 183, 235, 10), width=1)
    for ypos in range(0, height, gap):
        draw.line([(0, ypos), (width, ypos)], fill=(154, 183, 235, 10), width=1)
    draw.line([(0, height * .73), (width, height * .16)], fill=(100, 145, 227, 18), width=2)
    im.alpha_composite(grid)
    return im


def ui_geometry(vertical):
    return (50, 448, 980, 1020) if vertical else (96, 80, 1728, 972)


def assets(variant, shot, index, count):
    vertical = variant == "shorts"
    size = (1080, 1920) if vertical else (1920, 1080)
    width, height = size
    basename = WORK / f"{variant}-{index:02d}-{shot['id']}"
    bg = background(size)
    x, y, boxw, boxh = ui_geometry(vertical)
    glow = Image.new("RGBA", size)
    gd = ImageDraw.Draw(glow)
    gd.rounded_rectangle((x - 12, y - 12, x + boxw + 12, y + boxh + 12), radius=26, fill=(0, 0, 0, 150))
    bg.alpha_composite(glow.filter(ImageFilter.GaussianBlur(22)))
    draw = ImageDraw.Draw(bg)
    draw.rounded_rectangle((x - 8, y - 8, x + boxw + 8, y + boxh + 8), radius=22, fill=(10, 15, 24, 255), outline=(133, 166, 220, 95), width=1)
    bg_path = basename.with_name(basename.name + "-bg.png")
    bg.convert("RGB").save(bg_path)
    overlay = Image.new("RGBA", size)
    draw = ImageDraw.Draw(overlay)
    if vertical:
        draw.text((64, 95), "LUX MC", font=font(31, True), fill=(217, 194, 133, 255))
        draw.text((64, 142), "MINECRAFT, DO SEU JEITO", font=font(19, True), fill=(151, 179, 222, 255))
        draw.rounded_rectangle((794, 102, 1016, 140), radius=19, fill=(133, 166, 220, 22), outline=(133, 166, 220, 55))
        draw.text((820, 108), "DEMONSTRAÇÃO", font=font(17, True), fill=(183, 200, 229, 255))
        title = shot.get("shortTitle", shot["title"])
        text_block(draw, title, (64, 228), 65, 940, bold=True, max_lines=2)
        draw.text((64, 393), f"{index:02d}  /  {count:02d}", font=font(20, True), fill=(130, 170, 239, 255))
        draw.line([(64, 1828), (1016, 1828)], fill=(149, 174, 216, 45), width=3)
        draw.line([(64, 1828), (64 + 952 * index / count, 1828)], fill=(141, 174, 232, 220), width=3)
        draw.text((64, 1850), "LUX MC LAUNCHER", font=font(18, True), fill=(145, 165, 199, 255))
    else:
        draw.text((98, 27), "LUX MC", font=font(23, True), fill=(222, 196, 128, 255))
        draw.text((263, 26), f"{index:02d}  /  {shot['title'].upper()}", font=font(24, True), fill=(228, 237, 253, 255))
        draw.text((1635, 29), "DEMONSTRAÇÃO", font=font(17, True), fill=(150, 176, 218, 255))
        for ypos in range(900, 1080):
            alpha = int(225 * ((ypos - 900) / 180) ** .7)
            draw.line([(0, ypos), (1920, ypos)], fill=(5, 8, 15, alpha))
    title_path = basename.with_name(basename.name + "-title.png")
    overlay.save(title_path)
    captions = shot["shortCaptions"] if vertical else shot["captions"]
    caption_paths = []
    for cue_index, cue in enumerate(captions):
        cap = Image.new("RGBA", size)
        cd = ImageDraw.Draw(cap)
        if vertical:
            wrapped, face = lines(cue, 45, 908, True, 3)
            box_height = len(wrapped) * 56 + 62
            cd.rounded_rectangle((60, 1560, 1020, 1560 + box_height), radius=24, fill=(4, 7, 13, 205), outline=(140, 171, 222, 45), width=1)
            text_block(cd, cue, (86, 1590), 45, 908, bold=True, centered=True, max_lines=3)
        else:
            wrapped, face = lines(cue, 35, 1420, True, 2)
            box_height = len(wrapped) * 43 + 32
            top = 1017 - box_height
            cd.rounded_rectangle((210, top, 1710, 1017), radius=18, fill=(4, 7, 13, 216), outline=(140, 171, 222, 45), width=1)
            text_block(cd, cue, (250, top + 13), 35, 1420, bold=True, centered=True, max_lines=2)
        cap_path = basename.with_name(basename.name + f"-caption-{cue_index:02d}.png")
        cap.save(cap_path)
        caption_paths.append(cap_path)
    return bg_path, title_path, caption_paths


def make_bookend(variant, closing):
    vertical = variant == "shorts"
    size = (1080, 1920) if vertical else (1920, 1080)
    width, height = size
    im = background(size)
    draw = ImageDraw.Draw(im)
    logo = Image.open(ROOT / "static" / "logo.png").convert("RGBA")
    logosize = 610 if vertical else 450
    logo.thumbnail((logosize, logosize), Image.Resampling.LANCZOS)
    logo_y = 440 if vertical else 70
    im.alpha_composite(logo, ((width - logo.width) // 2, logo_y))
    title = "Seu próximo mundo\ncomeça aqui." if closing else "Seu Minecraft.\nSeu jeito de jogar."
    title_y = 1130 if vertical else 550
    title_size = 78 if vertical else 67
    for line in title.split("\n"):
        text_block(draw, line, (60 if vertical else 180, title_y), title_size, width - (120 if vertical else 360), bold=True, centered=True, max_lines=1)
        title_y += int(title_size * 1.23)
    subtitle = "Biblioteca • Mods • Skins • Amigos" if not closing else "Lux MC Launcher para Windows"
    text_block(draw, subtitle, (80 if vertical else 150, 1415 if vertical else 745), 31 if vertical else 32, width - (160 if vertical else 300), fill=(160, 183, 222, 255), centered=True, max_lines=2)
    small = "Tour da interface • Dados de demonstração"
    text_block(draw, small, (60 if vertical else 160, 1770 if vertical else 972), 22 if vertical else 20, width - (120 if vertical else 320), fill=(111, 139, 183, 255), centered=True, max_lines=1)
    path = WORK / f"{variant}-{'outro' if closing else 'intro'}.png"
    im.convert("RGB").save(path)
    return path


def render_bookend(ffmpeg, variant, closing, duration, output):
    source = make_bookend(variant, closing)
    size = "1080x1920" if variant == "shorts" else "1920x1080"
    frames = round(duration * FPS)
    vf = f"scale=iw*1.05:ih*1.05,zoompan=z='1.03-0.000012*on':x='iw/2-iw/zoom/2':y='ih/2-ih/zoom/2':d={frames}:s={size}:fps={FPS},fade=t=in:st=0:d=0.25,fade=t=out:st={duration-.25}:d=0.25,format=yuv420p"
    run([ffmpeg, "-y", "-hide_banner", "-loglevel", "error", "-i", str(source), "-vf", vf, "-t", str(duration), "-an", "-c:v", "libx264", "-preset", "veryfast", "-crf", "18", "-threads", "4", "-movflags", "+faststart", str(output)], output.with_suffix(".log"))


def source_dimensions(ffmpeg, path):
    result = subprocess.run([ffmpeg, "-hide_banner", "-i", str(path)], capture_output=True, text=True, errors="replace", creationflags=CREATE_NO_WINDOW)
    match = re.search(r"Video:.*?,\s*(\d{2,5})x(\d{2,5})(?:[ ,])", result.stderr)
    if match:
        return int(match.group(1)), int(match.group(2))
    if path.suffix.lower() in {".png", ".jpg", ".jpeg", ".webp"}:
        return Image.open(path).size
    raise RuntimeError(f"Cannot read source dimensions: {path}")


def installer_stage(source):
    size = (1728, 972)
    im = background(size)
    asset = Image.open(source).convert("RGBA")
    asset.thumbnail((535, 926), Image.Resampling.LANCZOS)
    im.alpha_composite(asset, (95, (size[1]-asset.height)//2))
    draw = ImageDraw.Draw(im)
    draw.text((744, 165), "LUX MC LAUNCHER", font=font(26, True), fill=(218, 193, 132, 255))
    text_block(draw, "Seu launcher.", (740, 248), 76, 870, bold=True, max_lines=1)
    text_block(draw, "Sua coleção.", (740, 342), 76, 870, bold=True, max_lines=1)
    features = ["Vanilla e modpacks", "Skins em três dimensões", "Sua turma em um só lugar"]
    for index, feature in enumerate(features):
        ypos = 508+index*88
        draw.ellipse((744, ypos+8, 774, ypos+38), fill=(40, 82, 139, 255), outline=(112, 158, 231, 255), width=2)
        draw.line([(752, ypos+23), (759, ypos+29), (766, ypos+16)], fill=(224, 237, 255, 255), width=3)
        draw.text((800, ypos), feature, font=font(34, True), fill=(217, 229, 248, 255))
    draw.text((744, 836), "WINDOWS  •  LUX MC LAUNCHER", font=font(22, True), fill=(141, 169, 213, 255))
    target = WORK / "installer-stage.png"
    im.convert("RGB").save(target)
    return target


def render_key(manifest_dir, variant, shot, index, count, duration):
    source = manifest_dir / shot["path"]
    signature = {
        "version": 2,
        "source": hashlib.sha256(source.read_bytes()).hexdigest(),
        "shot": shot,
        "variant": variant,
        "index": index,
        "count": count if variant == "shorts" else None,
        "duration": duration,
        "focus": SHORT_FOCUS.get(shot["id"]),
    }
    return hashlib.sha256(json.dumps(signature, ensure_ascii=False, sort_keys=True).encode("utf-8")).hexdigest()


def render_shot(ffmpeg, manifest_dir, variant, shot, index, count, duration, output):
    bg, title, captions = assets(variant, shot, index, count)
    path = manifest_dir / shot["path"]
    if not path.is_file():
        raise FileNotFoundError(path)
    vertical = variant == "shorts"
    if shot["id"] == "installer" and not vertical:
        path = installer_stage(path)
    x, y, width, height = ui_geometry(vertical)
    source_size = source_dimensions(ffmpeg, path)
    prefix = ""
    focus = shot.get("productionFocus") or SHORT_FOCUS.get(shot["id"], shot.get("shortFocus"))
    if vertical and focus:
        cx, cy, cw, ch = map(int, focus)
        cw, ch = min(cw, source_size[0]), min(ch, source_size[1])
        cx, cy = min(max(cx, 0), source_size[0] - cw), min(max(cy, 0), source_size[1] - ch)
        prefix = f"crop={cw}:{ch}:{cx}:{cy},"
    command = [ffmpeg, "-y", "-hide_banner", "-loglevel", "error", "-loop", "1", "-framerate", "1", "-i", str(bg)]
    image = shot.get("type") == "image" or path.suffix.lower() in {".png", ".jpg", ".jpeg", ".webp"}
    command += ["-loop", "1", "-framerate", "30", "-i", str(path)] if image else ["-ss", str(shot.get("start", 0)), "-i", str(path)]
    command += ["-loop", "1", "-framerate", "1", "-i", str(title)]
    for caption in captions:
        command += ["-loop", "1", "-framerate", "1", "-i", str(caption)]
    motion = f"zoompan=z='1+min(0.025,on*0.000045)':d=1:s={width}x{height}:fps={FPS}," if image else ""
    graph = [f"[0:v]fps={FPS},format=yuv420p,setsar=1[base]", f"[1:v]{prefix}scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:color=0x0A0F18,{motion}fps={FPS},setsar=1,tpad=stop_mode=clone:stop_duration={duration},trim=duration={duration},setpts=PTS-STARTPTS[ui]", f"[base][ui]overlay={x}:{y}:shortest=1[placed]", "[placed][2:v]overlay=0:0:shortest=1[titled]"]
    state = "titled"
    cue_duration = duration / max(1, len(captions))
    for cue_index in range(len(captions)):
        next_state = f"cap{cue_index}"
        start, end = shot.get("cueTimings", [[cue_index * cue_duration, (cue_index + 1) * cue_duration]])[cue_index if shot.get("cueTimings") else 0]
        graph.append(f"[{state}][{3+cue_index}:v]overlay=0:0:enable='between(t,{start:.3f},{end:.3f})':shortest=1[{next_state}]")
        state = next_state
    graph.append(f"[{state}]fade=t=in:st=0:d=0.16:color=0x080D17,fade=t=out:st={duration-.16}:d=0.16:color=0x080D17,format=yuv420p[out]")
    graph_path = output.with_suffix(".filters.txt")
    graph_path.write_text(";\n".join(graph), encoding="utf-8")
    command += ["-filter_complex_script", str(graph_path), "-map", "[out]", "-t", str(duration), "-an", "-r", str(FPS), "-c:v", "libx264", "-preset", "veryfast", "-crf", "18", "-threads", "4", "-filter_complex_threads", "2", "-movflags", "+faststart", str(output)]
    run(command, output.with_suffix(".log"))
    output.with_suffix(".render.json").write_text(json.dumps({"key": render_key(manifest_dir, variant, shot, index, count, duration)}), encoding="utf-8")


def timestamp(seconds):
    milliseconds = round(seconds * 1000)
    hours, milliseconds = divmod(milliseconds, 3600000)
    minutes, milliseconds = divmod(milliseconds, 60000)
    seconds, milliseconds = divmod(milliseconds, 1000)
    return f"{hours:02d}:{minutes:02d}:{seconds:02d},{milliseconds:03d}"


def write_srt(path, timeline):
    cues = []
    number = 1
    for item in timeline:
        duration = item["duration"] / max(1, len(item["captions"]))
        for index, text in enumerate(item["captions"]):
            relative_start, relative_end = item.get("cueTimings", [[index * duration, (index + 1) * duration]])[index if item.get("cueTimings") else 0]
            start = item["start"] + relative_start
            end = item["start"] + relative_end - .08
            cues.append(f"{number}\n{timestamp(start)} --> {timestamp(end)}\n{text}\n")
            number += 1
    path.write_text("\n".join(cues), encoding="utf-8-sig")


def write_wave(path, data, rate=RATE):
    pcm = np.clip(data, -1, 1)
    with wave.open(str(path), "wb") as output:
        output.setnchannels(2 if data.ndim > 1 else 1)
        output.setsampwidth(2)
        output.setframerate(rate)
        output.writeframes((pcm * 32767).astype("<i2").tobytes())


def music(path, duration, transitions):
    count = int((duration + 1) * RATE)
    track = np.zeros((count, 2), dtype=np.float32)
    rng = np.random.default_rng(20261005)
    beat = 60 / 108
    harmony = [[57, 60, 64, 67], [53, 57, 60, 64], [48, 55, 60, 64], [55, 59, 62, 67]]

    def add(signal, start, level, pan=0):
        index = int(start * RATE)
        end = min(count, index + len(signal))
        if index >= count:
            return
        track[index:end, 0] += signal[:end-index] * level * math.sqrt((1-pan)/2)
        track[index:end, 1] += signal[:end-index] * level * math.sqrt((1+pan)/2)

    def tone(note, length, pluck=True):
        tt = np.arange(int(length * RATE), dtype=np.float32) / RATE
        freq = 440 * 2 ** ((note - 69) / 12)
        attack = np.minimum(1, tt / .009)
        envelope = attack * np.exp(-tt * (2.7 if pluck else 1.05)) * np.minimum(1, (length-tt) / .08)
        shape = np.sin(2*np.pi*freq*tt + .45*np.sin(2*np.pi*freq*2*tt)*np.exp(-tt*5)) + .13*np.sin(2*np.pi*freq*3*tt)
        return shape * envelope

    bars = math.ceil(duration / (beat * 4))
    for bar in range(bars):
        start = bar * beat * 4
        chord = harmony[(bar // 2) % len(harmony)]
        for note in chord:
            add(tone(note+12, beat*5, False), start, .019, -.28)
        for step in range(8):
            note = chord[[0, 2, 1, 3, 2, 1, 3, 2][step]] + 24
            add(tone(note, beat*.9), start + step*beat*.5, .045, .22 if step % 2 else -.22)
        for pulse in range(4):
            hit = start + pulse * beat
            add(tone(chord[0]-12, beat*.9, False), hit, .16)
            tt = np.arange(int(.3*RATE), dtype=np.float32) / RATE
            kick = np.sin(2*np.pi*(45*tt + 75*.027*(1-np.exp(-tt/.027)))) * np.exp(-tt*21)
            add(kick, hit, .31)
            for eighth in range(2):
                tt = np.arange(int(.1*RATE), dtype=np.float32) / RATE
                noise = rng.normal(0, 1, len(tt)).astype(np.float32)
                high = noise - np.convolve(noise, np.ones(5, dtype=np.float32)/5, mode="same")
                add(high * np.exp(-tt*75), hit+eighth*beat*.5, .026, .28)
            if pulse % 2:
                tt = np.arange(int(.18*RATE), dtype=np.float32) / RATE
                noise = rng.normal(0, .8, len(tt)).astype(np.float32)
                body = np.sin(2*np.pi*190*tt) * np.exp(-tt*35)
                add((noise*.22+body*.7)*np.exp(-tt*23), hit, .072, -.1)
    for transition in transitions:
        tt = np.arange(int(.23*RATE), dtype=np.float32) / RATE
        swoosh = rng.normal(0, .2, len(tt)).astype(np.float32)
        swoosh = np.convolve(swoosh, np.ones(7, dtype=np.float32)/7, mode="same")
        add(swoosh*np.sin(np.pi*tt/.23), max(0, transition-.17), .14, -.1)
    fade = min(int(RATE*2), count//2)
    track[:fade] *= np.linspace(0, 1, fade)[:, None]
    track[-fade:] *= np.linspace(1, 0, fade)[:, None]
    peak = float(np.max(np.abs(track)))
    if peak > .92:
        track *= .92 / peak
    write_wave(path, track)


def narration(ffmpeg, timeline, variant, enabled):
    path = WORK / f"{variant}-narration.wav"
    if not enabled:
        return None
    tasks = []
    for index, item in enumerate(timeline):
        for cue_index, cue in enumerate(item["captions"]):
            voice_path = WORK / f"{variant}-voice-{index:02d}-{cue_index:02d}.wav"
            tasks.append({"path": str(voice_path), "text": cue, "id": item["id"], "start": item["start"] + cue_index * item["duration"] / len(item["captions"]), "duration": item["duration"] / len(item["captions"])})
    manifest = WORK / f"{variant}-narration.json"
    manifest.write_text(json.dumps(tasks, ensure_ascii=False, indent=2), encoding="utf-8-sig")
    powershell = shutil.which("pwsh") or shutil.which("powershell") or "powershell.exe"
    run([powershell, "-NoProfile", "-File", str(ROOT / "scripts" / "produce-launcher-narration.ps1"), "-Manifest", str(manifest)], WORK / f"{variant}-narration.log")
    cursor = 0
    for item in timeline:
        cues = tasks[cursor:cursor+len(item["captions"])]
        cursor += len(cues)
        raw_lengths = []
        for cue in cues:
            with wave.open(cue["path"], "rb") as source:
                raw_lengths.append(source.getnframes()/source.getframerate())
        budget = item["duration"] - .45*len(cues)
        raw_total = sum(raw_lengths)
        elapsed = 0
        item["cueTimings"] = []
        for cue, raw in zip(cues, raw_lengths):
            slot = budget*raw/raw_total + .45
            cue["start"] = item["start"] + elapsed
            cue["duration"] = slot
            item["cueTimings"].append([elapsed, elapsed+slot])
            elapsed += slot
    manifest.write_text(json.dumps(tasks, ensure_ascii=False, indent=2), encoding="utf-8-sig")
    duration = timeline[-1]["start"] + timeline[-1]["duration"]
    data = np.zeros(int((duration+1)*RATE), dtype=np.float32)
    voice_lengths = []
    for index, item in enumerate(tasks):
        original = Path(item["path"])
        converted = original.with_name(original.stem + "-44k.wav")
        with wave.open(str(original), "rb") as source:
            voice_duration = source.getnframes()/source.getframerate()
        available = item["duration"] - .45
        speed = max(1, voice_duration / max(1, available))
        if speed > 1.36:
            raise RuntimeError(f"Narration too long for {item['id']}: {voice_duration:.2f}s in {item['duration']:.2f}s. Shorten captions.")
        run([ffmpeg, "-y", "-hide_banner", "-loglevel", "error", "-i", str(original), "-af", f"atempo={speed:.5f},highpass=f=80,lowpass=f=11000,alimiter=limit=.92", "-ar", str(RATE), "-ac", "1", str(converted)])
        with wave.open(str(converted), "rb") as source:
            speech = np.frombuffer(source.readframes(source.getnframes()), dtype="<i2").astype(np.float32) / 32768
        start = int((item["start"]+.18)*RATE)
        end = min(len(data), start+len(speech))
        data[start:end] += speech[:end-start]
        voice_lengths.append({"id": item["id"], "duration": voice_duration, "speed": speed})
    write_wave(path, data)
    (WORK / f"{variant}-voice-timing.json").write_text(json.dumps(voice_lengths, indent=2), encoding="utf-8")
    return path


def read_metadata(ffmpeg, ffprobe, path):
    if ffprobe:
        return json.loads(run([ffprobe, "-v", "error", "-show_entries", "format=duration,size:stream=index,codec_name,codec_type,width,height,r_frame_rate,sample_rate,channels", "-of", "json", str(path)]))
    result = subprocess.run([ffmpeg, "-hide_banner", "-i", str(path)], capture_output=True, text=True, errors="replace", creationflags=CREATE_NO_WINDOW)
    duration_match = re.search(r"Duration: (\d+):(\d+):(\d+\.\d+)", result.stderr)
    dims = source_dimensions(ffmpeg, path)
    duration = sum(float(value)*multiplier for value, multiplier in zip(duration_match.groups(), [3600, 60, 1]))
    return {"format": {"duration": str(duration), "size": str(path.stat().st_size)}, "streams": [{"codec_type": "video", "codec_name": "h264", "width": dims[0], "height": dims[1]}, {"codec_type": "audio", "codec_name": "aac"}], "probe": "ffmpeg stderr; ffprobe unavailable"}


def contact_sheet(ffmpeg, path, duration, vertical):
    times = np.linspace(1, duration-1, 12)
    tilew = 270 if vertical else 480
    tileh = 480 if vertical else 270
    sheet = Image.new("RGB", (tilew*4, (tileh+40)*3), (7, 11, 19))
    draw = ImageDraw.Draw(sheet)
    for index, moment in enumerate(times):
        frame = WORK / f"{path.stem}-frame-{index:02d}.jpg"
        run([ffmpeg, "-y", "-hide_banner", "-loglevel", "error", "-ss", str(moment), "-i", str(path), "-frames:v", "1", "-vf", f"scale={tilew}:{tileh}", str(frame)])
        image = Image.open(frame)
        left, top = index % 4 * tilew, index // 4 * (tileh+40)
        sheet.paste(image, (left, top))
        draw.text((left+12, top+tileh+9), f"{moment:06.1f}s", font=font(20), fill=(214, 225, 245))
    target = EXPORT / f"{path.stem}-contactsheet.jpg"
    sheet.save(target, quality=94)
    return target


def audio_quality(ffmpeg, path, variant):
    logfile = WORK / f"{variant}-audio-loudness.log"
    target = "NUL" if os.name == "nt" else "/dev/null"
    run([ffmpeg, "-hide_banner", "-i", str(path), "-af", "ebur128=peak=true", "-f", "null", target], logfile)
    summary = logfile.read_text(encoding="utf-8").rsplit("Summary:", 1)[-1]
    integrated = float(re.search(r"I:\s*(-?\d+\.\d+) LUFS", summary).group(1))
    lra = float(re.search(r"LRA:\s*(-?\d+\.\d+) LU", summary).group(1))
    peak = float(re.search(r"Peak:\s*(-?\d+\.\d+) dBFS", summary).group(1))
    assert peak < -.5
    assert -19 <= integrated <= -14
    return {"integratedLufs": integrated, "loudnessRangeLu": lra, "truePeakDbfs": peak, "fullDecode": "passed"}


def produce(ffmpeg, ffprobe, manifest_path, manifest, variant, voice, reuse):
    vertical = variant == "shorts"
    all_shots = []
    for raw in manifest["shots"]:
        shot = dict(raw)
        default = DEFAULTS.get(shot["id"], (shot["id"].replace("-", " ").title(), "Conheça as opções desta tela do Lux MC Launcher."))
        shot["title"] = shot.get("normalTitle", shot.get("title", default[0]))
        shot["captions"] = shot.get("captions") or [default[1]]
        shot["shortCaptions"] = [shot.get("shortCaption") or shot["captions"][0]]
        all_shots.append(shot)
    shots = [item for item in all_shots if item.get("shortDuration", 0) > 0] if vertical else all_shots
    if not shots:
        shots = [item for item in all_shots if item["id"] in {"home", "discover", "skins", "namemc", "friends", "p2p", "settings"}] if vertical else all_shots
    intro_duration = 3.0 if vertical else 4.0
    outro_duration = 5.0 if vertical else 7.0
    durations = [float(item.get("shortDuration", 6.5) if vertical else max(12, item.get("normalDuration", 12)) if item["id"] == "installer" else item.get("normalDuration", 12)) for item in shots]
    if vertical:
        factor = (55-intro_duration-outro_duration)/sum(durations)
        durations = [round(value*factor, 3) for value in durations]
    timeline = [{"id": "intro", "start": 0, "duration": intro_duration, "captions": ["Lux MC." if vertical else "Lux MC. Seu jeito de jogar."]}]
    start = intro_duration
    for shot, duration in zip(shots, durations):
        timeline.append({"id": shot["id"], "start": start, "duration": duration, "captions": shot["shortCaptions"] if vertical else shot["captions"]})
        start += duration
    timeline.append({"id": "outro", "start": start, "duration": outro_duration, "captions": ["Lux MC Launcher. Seu próximo mundo começa aqui."]})
    total = start+outro_duration
    voice_path = narration(ffmpeg, timeline, variant, voice)
    (WORK / f"{variant}-timeline.json").write_text(json.dumps(timeline, ensure_ascii=False, indent=2), encoding="utf-8")
    for shot, entry in zip(shots, timeline[1:-1]):
        if entry.get("cueTimings"):
            shot["cueTimings"] = entry["cueTimings"]
    segments = []
    intro = WORK / f"{variant}-00-intro.mp4"
    if not reuse or not intro.exists():
        render_bookend(ffmpeg, variant, False, intro_duration, intro)
    segments.append(intro)
    for index, (shot, duration) in enumerate(zip(shots, durations), 1):
        output = WORK / f"{variant}-{index:02d}-{shot['id']}.mp4"
        cache_path = output.with_suffix(".render.json")
        cached = cache_path.exists() and json.loads(cache_path.read_text(encoding="utf-8")).get("key") == render_key(manifest_path.parent, variant, shot, index, len(shots), duration)
        if not reuse or not output.exists() or not cached:
            render_shot(ffmpeg, manifest_path.parent, variant, shot, index, len(shots), duration, output)
        segments.append(output)
        print(f"Rendered {variant}: {index}/{len(shots)} {shot['id']}", flush=True)
    outro = WORK / f"{variant}-99-outro.mp4"
    if not reuse or not outro.exists():
        render_bookend(ffmpeg, variant, True, outro_duration, outro)
    segments.append(outro)
    concat = WORK / f"{variant}-concat.txt"
    concat.write_text("\n".join("file '" + str(path.resolve()).replace("\\", "/").replace("'", "'\\''") + "'" for path in segments), encoding="utf-8")
    picture = WORK / f"{variant}-picture.mp4"
    run([ffmpeg, "-y", "-hide_banner", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(concat), "-c", "copy", str(picture)], WORK / f"{variant}-concat.log")
    music_path = WORK / f"{variant}-original-music.wav"
    music(music_path, total, [item["start"] for item in timeline[1:]])
    output_name = "Lux MC Launcher - Shorts.mp4" if vertical else "Lux MC Launcher - Tour Completo.mp4"
    output = EXPORT / output_name
    command = [ffmpeg, "-y", "-hide_banner", "-loglevel", "info", "-i", str(picture), "-i", str(music_path)]
    if voice_path:
        command += ["-i", str(voice_path), "-filter_complex", "[1:a]volume=.46[music];[2:a]asplit=2[voice][side];[music][side]sidechaincompress=threshold=.025:ratio=4:attack=25:release=450[duck];[duck][voice]amix=inputs=2:duration=longest:normalize=0,loudnorm=I=-16:TP=-1.5:LRA=9[audio]"]
    else:
        command += ["-filter_complex", "[1:a]loudnorm=I=-18:TP=-1.5:LRA=9[audio]"]
    command += ["-map", "0:v", "-map", "[audio]", "-t", str(total), "-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-ar", "48000", "-ac", "2", "-movflags", "+faststart", "-metadata", "title=Lux MC Launcher", "-metadata", "comment=Interface gravada com dados de demonstracao; trilha original gerada para Lux MC", str(output)]
    run(command, WORK / f"{variant}-mux.log")
    srt = output.with_suffix(".srt")
    write_srt(srt, timeline)
    metadata = read_metadata(ffmpeg, ffprobe, output)
    expected_size = (1080, 1920) if vertical else (1920, 1080)
    video = next(stream for stream in metadata["streams"] if stream["codec_type"] == "video")
    assert (video["width"], video["height"]) == expected_size
    assert abs(float(metadata["format"]["duration"])-total) < .25
    assert any(stream["codec_type"] == "audio" for stream in metadata["streams"])
    sheet = contact_sheet(ffmpeg, output, total, vertical)
    audio = audio_quality(ffmpeg, output, variant)
    report = {"path": str(output), "subtitles": str(srt), "contactSheet": str(sheet), "durationSeconds": total, "sha256": hashlib.sha256(output.read_bytes()).hexdigest(), "narration": "Microsoft Daniel pt-BR" if voice_path else None, "music": "Original procedural composition; no third-party samples", "data": "Frontend real do launcher; fixture isolada de demonstracao", "metadata": metadata, "audioQuality": audio}
    (EXPORT / f"{variant}-verification.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2), flush=True)
    return report


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default=str(MEDIA / "captures" / "manifest.json"))
    parser.add_argument("--ffmpeg")
    parser.add_argument("--variant", choices=["shorts", "normal", "all"], default="all")
    parser.add_argument("--no-voice", action="store_true")
    parser.add_argument("--reuse", action="store_true")
    args = parser.parse_args()
    WORK.mkdir(parents=True, exist_ok=True)
    EXPORT.mkdir(parents=True, exist_ok=True)
    manifest_path = Path(args.manifest).resolve()
    manifest = json.loads(manifest_path.read_text(encoding="utf-8-sig"))
    ffmpeg, ffprobe = find_tools(args.ffmpeg)
    for variant in (["shorts", "normal"] if args.variant == "all" else [args.variant]):
        produce(ffmpeg, ffprobe, manifest_path, manifest, variant, not args.no_voice, args.reuse)


if __name__ == "__main__":
    main()
