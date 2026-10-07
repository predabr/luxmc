import json
import os
import queue
import subprocess
import threading
from pathlib import Path

binary = Path(os.environ['LOCALAPPDATA']) / 'Luxmc-build/target/release/luxmc.exe'
process = subprocess.Popen([str(binary), '--daemon'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, encoding='utf-8', creationflags=subprocess.CREATE_NO_WINDOW)
responses = queue.Queue()

def read():
    for line in process.stdout:
        try:
            responses.put(json.loads(line))
        except json.JSONDecodeError:
            pass

threading.Thread(target=read, daemon=True).start()

def call(identifier, command, args=None):
    process.stdin.write(json.dumps({'id': identifier, 'command': command, 'args': args or {}}) + '\n')
    process.stdin.flush()
    while True:
        response = responses.get(timeout=60)
        if response.get('id') == identifier:
            if response.get('error'):
                raise RuntimeError(response['error'])
            return response.get('result')

try:
    results = call(1, 'mods_search', {'query': 'Vulkan Optimized', 'mcVersion': '', 'limit': 10, 'offset': 0, 'contentType': 'modpack'})
    pack = next(item for item in results if item['title'].strip().lower() == 'vulkan optimized')
    assert pack['iconUrl'].startswith('https://')
    evidence = {'title': pack['title'], 'source': pack['source'], 'projectId': pack['sourceId'], 'icon': pack['iconUrl'], 'nativeCatalogVerified': True}
    Path('docs/validation/final-polish-artwork-live.json').write_text(json.dumps(evidence, indent=2), encoding='utf-8')
    print(json.dumps(evidence))
finally:
    process.terminate()
    process.wait(timeout=10)
