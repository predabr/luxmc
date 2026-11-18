from pathlib import Path
import os,json,subprocess,threading,queue,sqlite3
binary=Path(os.environ['LOCALAPPDATA'])/'Luxmc/luxmc.exe'
auth=json.loads((Path(os.environ['APPDATA'])/'io.github.luxmc.Luxmc/auth.json').read_text())['current_account']
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
    accounts=call(3,'auth_accounts',{})
    initialization=call(4,'app_init',{})
    profiles=call(1,'profiles_list',{})
    call(2,'auth_save_appearance',{'uuid':auth['id'],'skinUrl':auth['skinUrl'],'variant':str(auth.get('skinVariant') or 'classic').lower(),'capeUrl':auth.get('capeUrl')})
    result={'nativeDbInitialized':True,'accountsDecoded':len(accounts),'appInit':'passed','profilesReturned':len(profiles),'nativeSaveAppearance':'passed'}
    print(json.dumps(result))
    Path('docs/validation/skin-db-native-verify.json').write_text(json.dumps(result,indent=2))
finally:
    process.stdin.close()
    try:process.wait(timeout=5)
    except subprocess.TimeoutExpired:process.terminate();process.wait(timeout=5)
connection=sqlite3.connect(Path(os.environ['APPDATA'])/'github/Luxmc/data/luxmc.db')
assert connection.execute('PRAGMA integrity_check').fetchall()==[('ok',)]
connection.close()
print('Integrity after native skin save: ok')

