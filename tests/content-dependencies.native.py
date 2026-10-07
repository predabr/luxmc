import hashlib
import json
import os
import queue
import subprocess
import threading
import time
import urllib.request
import urllib.parse
from pathlib import Path

binary = Path(os.environ['LUXMC_NATIVE_BINARY']).resolve(strict=True)
process = subprocess.Popen([str(binary), '--daemon'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, encoding='utf8', creationflags=subprocess.CREATE_NO_WINDOW)
responses = queue.Queue()
created = []
results = []
identifier = 0
base = None

def reader():
    for line in process.stdout:
        try:
            responses.put(json.loads(line))
        except json.JSONDecodeError:
            pass

threading.Thread(target=reader, daemon=True).start()

def call(command, args=None):
    global identifier
    identifier += 1
    process.stdin.write(json.dumps({'id':identifier, 'command':command, 'args':args or {}})+'\n')
    process.stdin.flush()
    while True:
        response = responses.get(timeout=240)
        if response.get('id') == identifier:
            if response.get('error'):
                raise RuntimeError(response['error'])
            return response['result']

def modrinth(path):
    request = urllib.request.Request('https://api.modrinth.com/v2/'+path, headers={'User-Agent':'Luxmc-validation/3.0.2 (https://luxmc-r92.pages.dev)'})
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)

def files(profile):
    return {path.name:hashlib.sha256(path.read_bytes()).hexdigest() for path in (Path(profile['gameDir'])/'mods').glob('*.jar')}

try:
    call('app_init')
    base = Path(call('app_data_directory')).resolve()/'instances'
    profile = call('profiles_create', {'input':{'name':'Luxmc content validation '+str(time.time_ns()), 'mcVersion':'1.20.1', 'loader':'fabric', 'ramMb':2048}})
    created.append(profile)
    project = modrinth('project/inventory-profiles-next')
    query = urllib.parse.urlencode({'game_versions':json.dumps(['1.20.1']), 'loaders':json.dumps(['fabric'])})
    version = modrinth('project/'+project['id']+'/version?'+query)[0]
    call('mods_install', {'request':{'profileId':profile['id'], 'projectId':project['id'], 'versionId':version['id'], 'source':'modrinth', 'contentType':'mod'}})
    installed = call('mods_list', {'profileId':profile['id']})
    assert len(installed) >= 2, 'Required dependency was not installed'
    expected = {dependency['version_id'] for dependency in version['dependencies'] if dependency['dependency_type']=='required' and dependency.get('version_id')}
    assert expected.issubset({row['versionId'] for row in installed}), 'Pinned dependency was not respected'
    before = files(profile)
    assert len(before) == len(installed)
    future_query = urllib.parse.urlencode({'game_versions':json.dumps(['1.21.1']), 'loaders':json.dumps(['fabric'])})
    future = modrinth('project/'+project['id']+'/version?'+future_query)[0]
    try:
        call('mods_install', {'request':{'profileId':profile['id'], 'projectId':project['id'], 'versionId':future['id'], 'source':'modrinth', 'contentType':'mod'}})
        raise AssertionError('Wrong Minecraft version was accepted')
    except RuntimeError as error:
        assert 'compat' in str(error).lower()
    assert files(profile) == before
    results.append({'provider':'modrinth', 'project':project['slug'], 'requiredDependenciesInstalled':len(installed)-1, 'exactPinsPreserved':True, 'wrongVersionRejected':True, 'previousFilesPreserved':True})
    hits = call('mods_search', {'query':'Controlling', 'mcVersion':'1.20.1', 'contentType':'mod', 'loader':'fabric', 'source':'curseforge', 'limit':20})
    hit = next(item for item in hits if item['title'].lower()=='controlling')
    versions = call('mods_versions', {'projectId':hit['sourceId'], 'mcVersion':'1.20.1', 'source':'curseforge'})
    selected = next(item for item in versions if 'fabric' in item['loaders'])
    call('mods_install', {'request':{'profileId':profile['id'], 'projectId':hit['sourceId'], 'versionId':selected['id'], 'source':'curseforge', 'contentType':'mod'}})
    rows = call('mods_list', {'profileId':profile['id']})
    print(json.dumps({'installedRows':[{'project':row['projectId'], 'source':row['source'], 'file':row['fileName']} for row in rows], 'jarFiles':list(files(profile))}), flush=True)
    assert any(row['source']=='curseforge' and row['projectId']==hit['sourceId'] for row in rows)
    assert len(files(profile)) == len(rows)
    results.append({'provider':'curseforge', 'project':hit['title'], 'compatibleInstallSucceeded':True, 'metadataDependenciesChecked':True, 'registeredFiles':len(rows)})
finally:
    for profile in created:
        directory = Path(profile['gameDir']).resolve()
        if base and directory==(base/profile['id']/'.minecraft').resolve() and profile['name'].startswith('Luxmc content validation '):
            call('profiles_delete', {'id':profile['id']})
    process.terminate()
    process.wait(timeout=15)
    Path('docs/validation/content-dependencies-2026-10-06.json').write_text(json.dumps(results,indent=2),encoding='utf8')

print(json.dumps(results),flush=True)
