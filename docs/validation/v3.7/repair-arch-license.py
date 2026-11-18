import hashlib
import io
import json
import pathlib
import re
import sys
import tarfile

import zstandard

source, destination, license_path, report_path = map(pathlib.Path, sys.argv[1:])
license_bytes = license_path.read_bytes()
original_bytes = source.read_bytes()
with zstandard.ZstdDecompressor().stream_reader(io.BytesIO(original_bytes)) as reader:
    archive_bytes = reader.read()
with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode='r:') as archive:
    members = archive.getmembers()
    content = {}
    for member in members:
        path = pathlib.PurePosixPath(member.name)
        assert not path.is_absolute() and '..' not in path.parts, member.name
        if member.isfile():
            content[member.name] = archive.extractfile(member).read()
    metadata = content['.PKGINFO'].decode()
    assert 'pkgver = 3.7.0-1\n' in metadata
    assert 'arch = x86_64\n' in metadata
    assert 'license = MIT\n' in metadata
    binary_hash = hashlib.sha256(content['usr/bin/luxmc']).hexdigest()
    license_name = 'usr/share/licenses/luxmc/LICENSE'
    assert license_name not in content
    installed_size = sum(len(value) for key, value in content.items() if key != '.PKGINFO') + len(license_bytes)
    metadata = metadata.replace('license = MIT\n', 'license = custom:Luxmc\n')
    metadata = re.sub(r'^size = \d+$', f'size = {installed_size}', metadata, flags=re.MULTILINE)
    content['.PKGINFO'] = metadata.encode()
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode='w:', format=tarfile.GNU_FORMAT) as updated:
        for member in members:
            if member.name == '.PKGINFO':
                member.size = len(content[member.name])
            updated.addfile(member, io.BytesIO(content[member.name]) if member.isfile() else None)
        names = {member.name.rstrip('/') for member in members}
        for directory in ['usr/share/licenses', 'usr/share/licenses/luxmc']:
            if directory not in names:
                item = tarfile.TarInfo(directory)
                item.type = tarfile.DIRTYPE
                item.mode = 0o755
                updated.addfile(item)
        item = tarfile.TarInfo(license_name)
        item.mode = 0o644
        item.size = len(license_bytes)
        updated.addfile(item, io.BytesIO(license_bytes))
patched = zstandard.ZstdCompressor(level=15, threads=2).compress(output.getvalue())
with zstandard.ZstdDecompressor().stream_reader(io.BytesIO(patched)) as reader:
    verification = reader.read()
with tarfile.open(fileobj=io.BytesIO(verification), mode='r:') as archive:
    for name, value in content.items():
        assert archive.extractfile(name).read() == value, name
    assert archive.extractfile(license_name).read() == license_bytes
    assert hashlib.sha256(archive.extractfile('usr/bin/luxmc').read()).hexdigest() == binary_hash
destination.parent.mkdir(parents=True, exist_ok=True)
destination.write_bytes(patched)
result = {'package': destination.name, 'version': '3.7.0-1', 'license': 'custom:Luxmc', 'licenseFileIncluded': True, 'allOtherFilesPreserved': True, 'binarySha256': binary_hash, 'originalSha256': hashlib.sha256(original_bytes).hexdigest(), 'sha256': hashlib.sha256(patched).hexdigest(), 'bytes': len(patched)}
report_path.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
print(json.dumps(result))
