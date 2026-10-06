import ctypes
import json
import os
import queue
import re
import subprocess
import threading
import time
from pathlib import Path

binary = Path(os.environ['LUXMC_NATIVE_BINARY']).resolve(strict=True)
output = Path(os.environ['LUXMC_VALIDATION_OUTPUT']).resolve()
output.parent.mkdir(parents=True, exist_ok=True)
process = subprocess.Popen([str(binary), '--daemon'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, encoding='utf8', creationflags=subprocess.CREATE_NO_WINDOW)
responses = queue.Queue()
results = []
created = []
owned_pid = None
identifier = 0
managed_base = None
cases = set(filter(None, os.environ.get('LUXMC_VALIDATION_CASES', '').split(',')))
kernel = ctypes.WinDLL('kernel32', use_last_error=True)
kernel.OpenProcess.restype = ctypes.c_void_p
kernel.CloseHandle.argtypes = [ctypes.c_void_p]
kernel.GetExitCodeProcess.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ulong)]

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
    deadline = time.monotonic() + 900
    while time.monotonic() < deadline:
        try:
            response = responses.get(timeout=min(10, deadline - time.monotonic()))
        except queue.Empty:
            if process.poll() is not None:
                raise RuntimeError('Native daemon exited')
            continue
        if response.get('id') == identifier:
            if response.get('error'):
                raise RuntimeError(response['error'])
            return response['result']
    raise TimeoutError(command)

def alive(pid):
    handle = kernel.OpenProcess(0x1000, False, pid)
    if not handle:
        return False
    code = ctypes.c_ulong()
    try:
        return bool(kernel.GetExitCodeProcess(handle, ctypes.byref(code))) and code.value == 259
    finally:
        kernel.CloseHandle(handle)

def stop_owned():
    global owned_pid
    if owned_pid and alive(owned_pid):
        subprocess.run(['taskkill', '/PID', str(owned_pid), '/T', '/F'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, creationflags=subprocess.CREATE_NO_WINDOW)
    owned_pid = None
    time.sleep(2)

def save():
    output.write_text(json.dumps(results, indent=2), encoding='utf8')

def launch(profile, account_id):
    global owned_pid
    print(json.dumps({'testing': profile['name'], 'version': profile['mcVersion'], 'loader': profile['loader']}), flush=True)
    started = time.monotonic()
    report = {'name': profile['name'], 'profileId': profile['id'], 'version': profile['mcVersion'], 'loader': profile['loader'], 'loaderVersion': profile.get('loaderVersion')}
    try:
        result = call('launch_game', {'request': {'versionId': profile['mcVersion'], 'profileId': profile['id'], 'accountId': account_id, 'enableVulkan': False}})
        owned_pid = result['pid']
        report['preparationSeconds'] = round(time.monotonic() - started, 2)
        print(json.dumps({'prepared': profile['name'], 'seconds': report['preparationSeconds']}), flush=True)
        deadline = time.monotonic() + 180
        content = ''
        while time.monotonic() < deadline:
            content = ''
            for name in ['latest.log', 'luxmc_game.log']:
                path = Path(profile['gameDir']) / 'logs' / name
                if path.exists() and path.stat().st_mtime >= time.time() - (time.monotonic() - started) - 2:
                    content += path.read_text(encoding='utf8', errors='replace')[-200000:]
            if not alive(owned_pid):
                report['status'] = 'exited_before_ready'
                errors = [line for line in content.splitlines() if any(marker in line for marker in ['Caused by:', 'Exception', 'Error:', '/FATAL]', 'GLFW error'])]
                report['errors'] = [re.sub(r'(?i)(access.?token|session id|authorization)[^\n]*', '[redacted]', line)[:500] for line in errors[-8:]]
                break
            atlas = bool(re.search(r'Created:.*(?:atlas|textures)|Created.*textures-atlas', content, re.I))
            sound = bool(re.search(r'Sound (?:engine|library|system)|OpenAL initialized', content, re.I))
            if atlas and sound:
                time.sleep(5)
                if alive(owned_pid):
                    report['status'] = 'resources_ready_process_alive'
                    report['readySeconds'] = round(time.monotonic() - started, 2)
                    break
            time.sleep(1)
        else:
            report['status'] = 'readiness_timeout'
        report['agentLoaded'] = 'Luxmc' in content
    except Exception as error:
        report['status'] = 'launch_failed'
        report['error'] = str(error)[:800]
    finally:
        stop_owned()
        results.append(report)
        save()
        print(json.dumps(report), flush=True)

try:
    initialized = call('app_init')
    managed_base = Path(call('app_data_directory')).resolve() / 'instances'
    account_id = initialized['account']['id']
    original = initialized['profiles']
    for profile in original:
        if (profile['mcVersion'] == '1.8.9' or profile['name'] in ['Vanilla Perfected', 'Vulkan Optimized']) and (not cases or profile['name'] in cases):
            launch(profile, account_id)
    for loader, version in [('vanilla', '26.3'), ('fabric', '1.20.1'), ('quilt', '1.20.1'), ('forge', '1.20.1'), ('neoforge', '1.21.1')]:
        if cases and loader not in cases:
            continue
        loader_version = None
        if loader != 'vanilla':
            versions = call('loaders_versions', {'loader': loader, 'gameVersion': version})['versions']
            if not versions:
                results.append({'loader': loader, 'version': version, 'status': 'no_loader_versions'})
                save()
                continue
            loader_version = next((item['id'] for item in versions if item['stable']), versions[0]['id'])
        profile = call('profiles_create', {'input': {'name': 'Luxmc validation ' + loader + ' ' + str(time.time_ns()), 'mcVersion': version, 'loader': loader, 'loaderVersion': loader_version, 'ramMb': 4096, 'autoOptimize': True, 'resolutionW': 960, 'resolutionH': 540}})
        created.append(profile)
        launch(profile, account_id)
finally:
    stop_owned()
    base = managed_base.resolve() if managed_base else None
    for profile in created:
        directory = Path(profile['gameDir']).resolve()
        if base and directory == (base / profile['id'] / '.minecraft').resolve() and directory.parent.name == profile['id'] and profile['name'].startswith('Luxmc validation '):
            call('profiles_delete', {'id': profile['id']})
    process.terminate()
    process.wait(timeout=10)
    save()

if any(item['status'] != 'resources_ready_process_alive' for item in results):
    raise SystemExit(1)
