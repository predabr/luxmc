import re

with open('src/lib/components/visuals/Cutscene.svelte', 'r') as f:
    content = f.read()

# Fix background fragments (restore to original)
content = re.sub(
    r'const aspect = \(imageSource\.width \|\| 1024\) \/ \(imageSource\.height \|\| 1024\);\n\s+const thickness = 0\.45;\n\s+const ROWS = 45;\n\s+const COLS = 45;\n\s+const intactGeom = new THREE\.PlaneGeometry\(width, height\);',
    'const ROWS = 6;\n\t\t\tconst COLS = 6;\n\t\t\tconst size = 4.4;\n\t\t\tconst thickness = 0.12;\n\n\t\t\tconst intactGeom = new THREE.PlaneGeometry(size, size);',
    content
)

# Fix shattered logo
content = re.sub(
    r'function createShatteredLogo\(imageSource: HTMLImageElement\) {\n\t\t\tconst size = 14;\n\t\t\tconst thickness = 0\.45;',
    'function createShatteredLogo(imageSource: HTMLImageElement) {\n\t\t\tconst aspect = (imageSource.width || 1024) / (imageSource.height || 1024);\n\t\t\tconst width = aspect >= 1 ? 16 : 16 * aspect;\n\t\t\tconst height = aspect >= 1 ? 16 / aspect : 16;\n\t\t\tconst thickness = 0.45;',
    content
)

# Replace all `size` references in createShatteredLogo grid with `width` and `height`
content = content.replace(
    'let gx = -size / 2 + (c / COLS) * size;\n\t\t\t\t\tlet gy = -size / 2 + (r / ROWS) * size;',
    'let gx = -width / 2 + (c / COLS) * width;\n\t\t\t\t\tlet gy = -height / 2 + (r / ROWS) * height;'
)
content = content.replace(
    'gx += (Math.random() - 0.5) * (size / COLS) * 0.72;\n\t\t\t\t\t\tgy += (Math.random() - 0.5) * (size / ROWS) * 0.72;',
    'gx += (Math.random() - 0.5) * (width / COLS) * 0.72;\n\t\t\t\t\t\tgy += (Math.random() - 0.5) * (height / ROWS) * 0.72;'
)
content = content.replace(
    'x: (tl.x + tr.x + br.x + bl.x) / 4 + (Math.random() - 0.5) * (size / COLS) * 0.35,\n\t\t\t\t\t\ty: (tl.y + tr.y + br.y + bl.y) / 4 + (Math.random() - 0.5) * (size / ROWS) * 0.35',
    'x: (tl.x + tr.x + br.x + bl.x) / 4 + (Math.random() - 0.5) * (width / COLS) * 0.35,\n\t\t\t\t\t\ty: (tl.y + tr.y + br.y + bl.y) / 4 + (Math.random() - 0.5) * (height / ROWS) * 0.35'
)
content = content.replace(
    'const nA = { x: (pA.x + size / 2) / size, y: (pA.y + size / 2) / size };\n\t\t\t\t\t\tconst nB = { x: (pB.x + size / 2) / size, y: (pB.y + size / 2) / size };\n\t\t\t\t\t\tconst nC = { x: (pC.x + size / 2) / size, y: (pC.y + size / 2) / size };\n\t\t\t\t\t\tconst nCenter = { x: (cx + size / 2) / size, y: (cy + size / 2) / size };',
    'const nA = { x: (pA.x + width / 2) / width, y: (pA.y + height / 2) / height };\n\t\t\t\t\t\tconst nB = { x: (pB.x + width / 2) / width, y: (pB.y + height / 2) / height };\n\t\t\t\t\t\tconst nC = { x: (pC.x + width / 2) / width, y: (pC.y + height / 2) / height };\n\t\t\t\t\t\tconst nCenter = { x: (cx + width / 2) / width, y: (cy + height / 2) / height };'
)

# UVs
content = content.replace(
    '(p1.x + size / 2) / size, (p1.y + size / 2) / size,\n\t\t\t\t\t\t\t(p2.x + size / 2) / size, (p2.y + size / 2) / size,\n\t\t\t\t\t\t\t(p3.x + size / 2) / size, (p3.y + size / 2) / size,\n\t\t\t\t\t\t\t// Back UVs (mirrored)\n\t\t\t\t\t\t\t(p1.x + size / 2) / size, (p1.y + size / 2) / size,\n\t\t\t\t\t\t\t(p3.x + size / 2) / size, (p3.y + size / 2) / size,\n\t\t\t\t\t\t\t(p2.x + size / 2) / size, (p2.y + size / 2) / size,',
    '(p1.x + width / 2) / width, (p1.y + height / 2) / height,\n\t\t\t\t\t\t\t(p2.x + width / 2) / width, (p2.y + height / 2) / height,\n\t\t\t\t\t\t\t(p3.x + width / 2) / width, (p3.y + height / 2) / height,\n\t\t\t\t\t\t\t// Back UVs (mirrored)\n\t\t\t\t\t\t\t(p1.x + width / 2) / width, (p1.y + height / 2) / height,\n\t\t\t\t\t\t\t(p3.x + width / 2) / width, (p3.y + height / 2) / height,\n\t\t\t\t\t\t\t(p2.x + width / 2) / width, (p2.y + height / 2) / height,'
)

with open('src/lib/components/visuals/Cutscene.svelte', 'w') as f:
    f.write(content)
