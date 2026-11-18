from pathlib import Path
import base64
import json
import os
import queue
import sqlite3
import struct
import subprocess
import threading
import tempfile
import uuid

binary = Path(os.environ['LOCALAPPDATA']) / 'Luxmc/luxmc.exe'
executable = binary.read_bytes()
pe_offset = struct.unpack_from('<I', executable, 0x3c)[0]
assert executable[pe_offset:pe_offset + 4] == b'PE\0\0'
subsystem = struct.unpack_from('<H', executable, pe_offset + 24 + 68)[0]
assert subsystem == 2
process = subprocess.Popen([str(binary), '--daemon'], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL, text=True, encoding='utf8', creationflags=subprocess.CREATE_NO_WINDOW)
responses = queue.Queue()

def read_responses():
    for line in process.stdout:
        try:
            responses.put(json.loads(line))
        except json.JSONDecodeError:
            pass

threading.Thread(target=read_responses, daemon=True).start()

def call(identifier, command, arguments=None):
    process.stdin.write(json.dumps({'id': identifier, 'command': command, 'args': arguments or {}}) + '\n')
    process.stdin.flush()
    while True:
        response = responses.get(timeout=35)
        if response.get('id') == identifier:
            if response.get('error'):
                raise RuntimeError(response['error'])
            return response.get('result')

try:
    locale = call(1, 'app_system_locale')
    accounts = call(2, 'auth_accounts')
    initialized = call(3, 'app_init')
    profiles = call(4, 'profiles_list')
    skins = call(5, 'skins_list')
    imported = call(6, 'namemc_import_skin', {'url': 'https://namemc.com/skin/63455d7069b397c2'})
    assert imported['skinId'] == '63455d7069b397c2'
    prefix = 'data:image/png;base64,'
    assert imported['skinUrl'].startswith(prefix)
    png = base64.b64decode(imported['skinUrl'][len(prefix):], validate=True)
    assert png[:8] == b'\x89PNG\r\n\x1a\n'
    width = int.from_bytes(png[16:20], 'big')
    height = int.from_bytes(png[20:24], 'big')
    assert width >= 64 and height in (width, width // 2)
    temporary = Path(tempfile.gettempdir()) / f'Luxmc-wallpaper-native-{uuid.uuid4()}'
    temporary.mkdir()
    source = temporary / 'wallpaper.png'
    source.write_bytes(Path('static/alex.png').read_bytes())
    managed_directory = Path(os.environ['LOCALAPPDATA']) / 'github/Luxmc/data/wallpapers'
    previous = set(managed_directory.glob('*'))
    managed = None
    try:
        managed = Path(call(7, 'wallpaper_import', {'path': str(source)}))
        assert managed.parent.resolve() == managed_directory.resolve()
        assert managed != source
        assert Path(call(8, 'wallpaper_import', {'path': str(source)})) == managed
        source.unlink()
        assert managed.read_bytes() == Path('static/alex.png').read_bytes()
    finally:
        if source.exists(): source.unlink()
        if managed and managed not in previous and managed.parent.resolve() == managed_directory.resolve(): managed.unlink()
        temporary.rmdir()
    result = {'installedBinary': str(binary), 'systemLocale': locale,
        'accountsDecoded': len(accounts), 'profilesReturned': len(profiles), 'skinsReturned': len(skins),
        'appInit': bool(initialized), 'nameMcNativeImport': True, 'pngDimensions': [width, height],
        'windowsGuiSubsystem': subsystem, 'wallpaperManagedCopySurvivesSourceDeletion': True}
finally:
    process.stdin.close()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.terminate()
        process.wait(timeout=5)

database = Path(os.environ['APPDATA']) / 'github/Luxmc/data/luxmc.db'
connection = sqlite3.connect(f'file:{database.as_posix()}?mode=ro', uri=True)
assert connection.execute('PRAGMA integrity_check').fetchall() == [('ok',)]
connection.close()
result['databaseIntegrity'] = 'ok'
Path('docs/validation/redesign-native-verify.json').write_text(json.dumps(result, indent=2), encoding='utf8')
print(json.dumps(result))
