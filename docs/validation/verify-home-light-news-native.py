import json
import os
from pathlib import Path
import queue
import sqlite3
import struct
import subprocess
import threading
from urllib.parse import urlparse

binary = Path(os.environ['LOCALAPPDATA']) / 'Luxmc/luxmc.exe'
executable = binary.read_bytes()
offset = struct.unpack_from('<I', executable, 0x3c)[0]
assert struct.unpack_from('<H', executable, offset + 24 + 68)[0] == 2
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

def call(identifier, command):
    process.stdin.write(json.dumps({'id': identifier, 'command': command, 'args': {}}) + '\n')
    process.stdin.flush()
    while True:
        response = responses.get(timeout=30)
        if response.get('id') != identifier:
            continue
        if response.get('error'):
            raise RuntimeError(response['error'])
        return response['result']

try:
    initialized = call(1, 'app_init')
    feed = call(2, 'minecraft_news')
    entries = feed['entries']
    official = [entry for entry in entries if urlparse(entry.get('readMoreLink', '')).scheme == 'https'
        and urlparse(entry.get('readMoreLink', '')).hostname in ('www.minecraft.net', 'minecraft.net', 'aka.ms')]
    assert official
    database = Path(os.environ['APPDATA']) / 'github/Luxmc/data/luxmc.db'
    with sqlite3.connect(database.as_uri() + '?mode=ro', uri=True) as connection:
        integrity = connection.execute('PRAGMA quick_check').fetchone()[0]
    assert integrity == 'ok'
    report = {'guiSubsystem': True, 'profiles': len(initialized.get('profiles', [])),
        'accountAvailable': bool(initialized.get('account')), 'databaseIntegrity': integrity,
        'feedEntries': len(entries), 'officialNewsEntries': len(official), 'latestNewsDate': max(entry['date'] for entry in official),
        'officialNewsSource': 'https://launchercontent.mojang.com/v2/news.json'}
    Path('docs/validation/home-light-news-native.json').write_text(json.dumps(report, indent=2), encoding='utf8')
    print(json.dumps(report, indent=2))
finally:
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
