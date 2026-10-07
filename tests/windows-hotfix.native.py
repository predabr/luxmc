import json
import os
import queue
import subprocess
import threading
from pathlib import Path

binary = Path(os.environ['LOCALAPPDATA']) / 'Luxmc-build/target/release/luxmc.exe'
process = subprocess.Popen([str(binary), '--daemon'], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL, text=True, encoding='utf8', creationflags=subprocess.CREATE_NO_WINDOW)
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
        response = responses.get(timeout=100)
        if response.get('id') == identifier:
            if response.get('error'):
                raise RuntimeError(response['error'])
            return response.get('result')

try:
    specs = call(1, 'get_system_specs')
    assert specs['launcherVersion'] == '3.0.2'
    initialized = call(2, 'app_init')
    accounts = call(3, 'auth_accounts')
    active = initialized.get('account')
    appearance_saved = False
    if active:
        normalized = str(active['uuid']).replace('-', '').lower()
        selected = next(row for row in accounts if str(row['uuid']).replace('-', '').lower() == normalized)
        if selected.get('skinUrl'):
            call(4, 'auth_save_appearance', {'uuid': normalized.upper(), 'skinUrl': selected['skinUrl'],
                'variant': selected.get('skinVariant') or 'classic', 'capeUrl': selected.get('capeUrl')})
            appearance_saved = True
    settings = call(5, 'settings_get')
    profile_items = call(6, 'mods_search', {'query': "Farmer's Delight Cutting Compat", 'mcVersion': '',
        'contentType': 'datapack', 'source': 'curseforge', 'limit': 5})
    datapack = next(item for item in profile_items if 'Cutting Compat' in item['title'])
    latest = call(7, 'mods_versions', {'projectId': datapack['sourceId'], 'mcVersion': '26.3', 'source': 'curseforge'})
    supported = call(8, 'mods_versions', {'projectId': datapack['sourceId'], 'mcVersion': '1.21.1', 'source': 'curseforge'})
    assert len(latest) == 0
    assert len(supported) > 0
    result = {'version': specs['launcherVersion'], 'appInit': 'passed', 'accountsDecoded': len(accounts),
        'profilesAvailable': len(initialized.get('profiles', [])), 'appearanceSavedWithNormalizedUuid': appearance_saved,
        'activeAccountPreferencePreserved': bool(settings and settings.get('activeAccountId')),
        'datapackFilesFor26_3': len(latest), 'datapackFilesFor1_21_1': len(supported)}
    print(json.dumps(result))
    Path('docs/validation/windows-hotfix-native-2026-10-06.json').write_text(json.dumps(result, indent=2), encoding='utf8')
finally:
    process.stdin.close()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.terminate()
        process.wait(timeout=5)
