import json
import os
import queue
import re
import subprocess
import threading
from pathlib import Path

binary = Path(os.environ['LUXMC_NATIVE_BINARY']).resolve(strict=True)
output = Path(os.environ['LUXMC_VALIDATION_OUTPUT']).resolve()
process = subprocess.Popen([str(binary), '--daemon'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, encoding='utf8', creationflags=subprocess.CREATE_NO_WINDOW)
responses = queue.Queue()
identifier = 0

def read():
    for line in process.stdout:
        try:
            responses.put(json.loads(line))
        except json.JSONDecodeError:
            pass

threading.Thread(target=read, daemon=True).start()

def call(command, args=None):
    global identifier
    identifier += 1
    process.stdin.write(json.dumps({'id': identifier, 'command': command, 'args': args or {}}) + '\n')
    process.stdin.flush()
    while True:
        response = responses.get(timeout=120)
        if response.get('id') == identifier:
            if response.get('error'):
                raise RuntimeError(response['error'])
            return response['result']

try:
    initialized = call('app_init')
    account = initialized['account']
    status = call('host_world', {'identity': {'username': account['username'], 'uuid': account['uuid'], 'avatarUrl': None}})
    current = call('tunnel_status')
    assert current and not current['worldReady']
    code = current['roomCode']
    assert code and re.fullmatch(r'LUXMC1-\d{16}', code)
    call('stop_session')
    assert call('tunnel_status') is None
    receipt = {'initialized': True, 'roomBeforeGame': True, 'waitingForLan': True, 'sessionStopped': True, 'inviteLength': len(code)}
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(receipt, indent=2), encoding='utf8')
    print(json.dumps(receipt), flush=True)
finally:
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=10)
