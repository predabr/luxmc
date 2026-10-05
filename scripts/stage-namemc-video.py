from pathlib import Path
from datetime import datetime, timezone
import json
import os
import subprocess
from PIL import Image, ImageDraw, ImageFont, ImageEnhance, ImageFilter

root = Path(__file__).resolve().parent.parent
captures = root / 'docs/media/2026-10-05/captures'
native = captures / 'namemc-native-gallery.mp4'
ffmpeg = Path(os.environ['LOCALAPPDATA']) / 'Luxmc-video-tools/imageio/imageio_ffmpeg/binaries/ffmpeg-win-x86_64-v7.1.exe'
background = Image.open(captures / 'skins-preview.png').convert('RGBA')
background = ImageEnhance.Brightness(background).enhance(.36)
shadow = Image.new('RGBA', background.size)
ImageDraw.Draw(shadow).rounded_rectangle((608, 70, 1312, 1010), 28, fill=(4, 8, 16, 220))
background.alpha_composite(shadow.filter(ImageFilter.GaussianBlur(22)))
draw = ImageDraw.Draw(background)
draw.rounded_rectangle((620, 82, 1300, 998), 24, fill=(15, 20, 29, 255), outline=(67, 95, 137, 255), width=2)
fonts = Path(os.environ['WINDIR']) / 'Fonts'
draw.text((642, 100), 'NameMC', font=ImageFont.truetype(str(fonts / 'seguisb.ttf'), 25), fill=(229, 238, 251, 255))
draw.text((1115, 110), 'CATÁLOGO DE SKINS', font=ImageFont.truetype(str(fonts / 'segoeui.ttf'), 12), fill=(132, 165, 211, 255))
draw.line((642, 139, 1278, 139), fill=(54, 77, 113, 255), width=1)
draw.text((642, 973), 'Galeria pública · Captura nativa', font=ImageFont.truetype(str(fonts / 'segoeui.ttf'), 13), fill=(142, 164, 195, 255))
background_path = captures / 'namemc-stage-background.png'
background.convert('RGB').save(background_path)
output = captures / 'namemc-native-stage.mp4'
command = [str(ffmpeg), '-y', '-hide_banner', '-loglevel', 'error', '-loop', '1', '-framerate', '30', '-i', str(background_path), '-i', str(native), '-filter_complex', '[1:v]scale=656:824,setsar=1[gallery];[0:v][gallery]overlay=632:144:shortest=1,format=yuv420p[out]', '-map', '[out]', '-t', '12', '-an', '-r', '30', '-c:v', 'libx264', '-preset', 'veryfast', '-crf', '18', '-threads', '2', '-filter_complex_threads', '2', '-movflags', '+faststart', str(output)]
subprocess.run(command, check=True, creationflags=subprocess.CREATE_NO_WINDOW)
manifest_path = captures / 'manifest.json'
manifest = json.loads(manifest_path.read_text(encoding='utf-8-sig'))
shot = next(shot for shot in manifest['shots'] if shot['id'] == 'namemc')
shot.update(path=output.name, start=0, productionFocus=[570, 45, 780, 980], recordedAt=datetime.now(timezone.utc).isoformat(), source='Public NameMC gallery recorded from the installed native window; cropped to exclude ads and composited over isolated demo UI')
manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding='utf8')
print(f'Native NameMC gallery staged: {output}')
