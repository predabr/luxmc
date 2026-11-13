import io
import json
import time
import zipfile
from pathlib import Path

workspace = Path(__file__).resolve().parents[1]
root = Path.home() / 'AppData/Roaming/github/Luxmc/data/instances'
names = ['fabric.mod.json', 'quilt.mod.json', 'META-INF/mods.toml', 'META-INF/neoforge.mods.toml']

class CountedFile(io.RawIOBase):
    def __init__(self, path):
        self.file = path.open('rb', buffering=0)
        self.bytes = 0

    def readable(self):
        return True

    def seekable(self):
        return True

    def readinto(self, buffer):
        count = self.file.readinto(buffer)
        self.bytes += count
        return count

    def seek(self, offset, whence=io.SEEK_SET):
        return self.file.seek(offset, whence)

    def tell(self):
        return self.file.tell()

    def close(self):
        self.file.close()
        super().close()

def inspect(stream):
    with zipfile.ZipFile(stream) as archive:
        return {name: archive.read(name) for name in names if name in archive.namelist()}

files = sorted(path for path in root.glob('*/.minecraft/mods/*.jar') if 256 * 1024 <= path.stat().st_size <= 32 * 1024 * 1024)[:48]
if not files:
    files = sorted(path for path in root.glob('*/mods/*.jar') if 256 * 1024 <= path.stat().st_size <= 32 * 1024 * 1024)[:48]
assert files, 'No installed archives available for a read-only reference comparison'
before = [(path.stat().st_size, path.stat().st_mtime_ns) for path in files]
measurements = []
for iteration in range(3):
    result = {'iteration': iteration, 'archives': len(files)}
    expected = []
    started = time.perf_counter()
    whole_bytes = 0
    for path in files:
        with CountedFile(path) as file:
            data = file.read()
            expected.append(inspect(io.BytesIO(data)))
            whole_bytes += file.bytes
    result['wholeFileSeconds'] = time.perf_counter() - started
    result['wholeFileBytesRead'] = whole_bytes
    started = time.perf_counter()
    sparse_bytes = 0
    for index, path in enumerate(files):
        with CountedFile(path) as file:
            with io.BufferedReader(file, buffer_size=65536) as buffered:
                assert inspect(buffered) == expected[index]
                sparse_bytes += file.bytes
    result['sparseSeconds'] = time.perf_counter() - started
    result['sparseBytesRead'] = sparse_bytes
    measurements.append(result)
assert before == [(path.stat().st_size, path.stat().st_mtime_ns) for path in files]
output = {'kind': 'Python reference comparison of archive I/O strategies; not a Rust or Minecraft launch benchmark', 'readOnly': True, 'metadataEquivalent': True, 'measurements': measurements}
(workspace / 'docs/validation/performance-oct11/archive-io-reference.json').write_text(json.dumps(output, indent=2), encoding='utf8')
print(json.dumps(output))
