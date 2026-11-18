import json
import os
import queue
import subprocess
import threading
import time
from pathlib import Path

root = Path('docs/validation')
binary = os.environ.get('LUXMC_NATIVE_BINARY') or str(Path(os.environ['LOCALAPPDATA']) / 'Luxmc-build/target/release/luxmc.exe')
process = subprocess.Popen([binary, '--daemon'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, encoding='utf8', creationflags=subprocess.CREATE_NO_WINDOW)
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
        response = responses.get(timeout=240)
        if response.get('id') == identifier:
            if response.get('error'):
                raise RuntimeError(response['error'])
            return response['result']

try:
    initialized = call(1, 'app_init')
    profile = next(profile for profile in initialized['profiles'] if profile['mcVersion'] == '1.8.9' and profile['loader'] == 'vanilla')
    identity = initialized['account']['id']
    print(json.dumps({'profileId': profile['id'], 'version': profile['mcVersion'], 'loader': profile['loader']}), flush=True)
    started = time.monotonic()
    request = {'versionId': '1.8.9', 'profileId': profile['id'], 'accountId': identity, 'enableVulkan': False}
    result = call(2, 'launch_game', {'request': request, **request, 'version_id': '1.8.9', 'profile_id': profile['id'], 'account_id': identity, 'enable_vulkan': False})
    report = {'version': '1.8.9', 'profileId': profile['id'], 'result': result, 'preparationSeconds': round(time.monotonic() - started, 2)}
    (root / 'legacy-startup-before.json').write_text(json.dumps(report, indent=2), encoding='utf8')
    print(json.dumps(report), flush=True)
    for _ in range(180):
        if (root / 'legacy-startup-stop').exists():
            break
        time.sleep(1)
except Exception as error:
    print(type(error).__name__ + ': ' + str(error), flush=True)
finally:
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
