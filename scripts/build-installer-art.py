from pathlib import Path
from PIL import Image

root = Path(__file__).resolve().parents[1]
output = root / 'packaging' / 'windows'
for name, size in [('sidebar', (164, 314)), ('header', (150, 57))]:
    source = output / f'{name}-preview.png'
    if not source.exists():
        raise SystemExit('Renderize primeiro com node scripts/render-installer-art.cjs')
    with Image.open(source) as image:
        image.convert('RGB').resize(size, Image.Resampling.LANCZOS).save(output / f'{name}.bmp')
