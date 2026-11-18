from pathlib import Path
import os,json,subprocess,threading,queue,sqlite3
binary=Path(os.environ['LOCALAPPDATA'])/'Luxmc/luxmc.exe'
database=Path(os.environ['APPDATA'])/'github/Luxmc/data/luxmc.db'
backup=Path(os.environ['LOCALAPPDATA'])/'Luxmc-build/recovery/artwork-before-repair.db'
backup.parent.mkdir(parents=True,exist_ok=True)
with sqlite3.connect(database) as source, sqlite3.connect(backup) as target:
    source.backup(target)
process=subprocess.Popen([str(binary),'--daemon'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True,encoding='utf8',creationflags=subprocess.CREATE_NO_WINDOW)
responses=queue.Queue()
def read():
    for line in process.stdout:
        try: responses.put(json.loads(line))
        except json.JSONDecodeError: pass
threading.Thread(target=read,daemon=True).start()
def call(number,command,args):
    process.stdin.write(json.dumps({'id':number,'command':command,'args':args})+'\n');process.stdin.flush()
    while True:
        response=responses.get(timeout=30)
        if response.get('id')==number:
            if response.get('error'):raise RuntimeError(response['error'])
            return response.get('result')
try:
    profiles=call(1,'profiles_list',{})
    matched=[]
    for profile in profiles:
        manifest=Path(profile['gameDir'])/'modrinth.index.json'
        if not manifest.exists():continue
        if json.loads(manifest.read_text(encoding='utf8')).get('name')!='Vanilla Perfected':continue
        assert profile['name']=='Vanilla Perfected'
        updated=call(2,'profiles_update',{'input':{'id':profile['id'],'icon':'/modpack_vanilla_perfected_icon.webp'}})
        for key,value in profile.items():
            if key not in ('icon','updatedAt'):assert updated[key]==value,key
        assert updated['icon']=='/modpack_vanilla_perfected_icon.webp'
        matched.append(profile['id'])
    assert len(matched)==1
    persisted=call(3,'profiles_list',{})
    assert next(profile for profile in persisted if profile['id']==matched[0])['icon']=='/modpack_vanilla_perfected_icon.webp'
    result={'restoredProfiles':len(matched),'otherProfileFieldsPreserved':True,'persistedIcon':'/modpack_vanilla_perfected_icon.webp','backup':str(backup)}
    Path('docs/validation/modpack-artwork-native-verify.json').write_text(json.dumps(result,indent=2))
    print(json.dumps(result))
finally:
    process.stdin.close()
    try:process.wait(timeout=5)
    except subprocess.TimeoutExpired:process.terminate();process.wait(timeout=5)
with sqlite3.connect(database) as connection:
    assert connection.execute('PRAGMA integrity_check').fetchall()==[('ok',)]
print('Integrity after artwork repair: ok')
