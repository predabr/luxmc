import hashlib
import json
import os
import time
import urllib.request
from pathlib import Path

version = json.loads(Path('package.json').read_text(encoding='utf-8'))['version']
local = Path('release-windows/Lux MC Launcher.exe')
expected = hashlib.sha256(local.read_bytes()).hexdigest()
local_bytes = local.read_bytes()
url = 'https://luxmc-r92.pages.dev/download/windows?validation=' + str(time.time_ns())
headers = {'User-Agent':'Luxmc-delivery-validation/' + version,'Cache-Control':'no-cache'}
with urllib.request.urlopen(urllib.request.Request(url, headers=headers, method='HEAD'), timeout=60) as response:
    assert response.status == 200
    assert 'Lux MC Launcher.exe' in response.headers.get('Content-Disposition','')
with urllib.request.urlopen(urllib.request.Request(url, headers={**headers,'Range':'bytes=0-1'}), timeout=60) as response:
    assert response.status == 206
    assert response.read() == b'MZ'
for range_header, expected_bytes in [('bytes=1048576-1048831',local_bytes[1048576:1048832]),('bytes=-64',local_bytes[-64:])]:
    with urllib.request.urlopen(urllib.request.Request(url, headers={**headers,'Range':range_header}), timeout=60) as response:
        assert response.status == 206
        assert response.read() == expected_bytes
with urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=60) as response:
    status = response.status
    filename = response.headers.get('Content-Disposition','')
    actual = hashlib.sha256()
    length = 0
    while chunk := response.read(1024 * 1024):
        actual.update(chunk)
        length += len(chunk)
assert status == 200 and length == local.stat().st_size and actual.hexdigest() == expected
receipt = {'version':version,'status':status,'bytes':length,'sha256':expected,'filename':filename,'url':url.split('?')[0],'headPassed':True,'rangePassed':True,'deployment':os.environ.get('LUXMC_SITE_DEPLOYMENT'),'githubPublished':True}
Path('docs/validation/v3.1-site-installer.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8')
print(json.dumps(receipt))
