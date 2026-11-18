from pathlib import Path
from datetime import datetime,timezone
import os,sqlite3,json
root=Path(Path('docs/validation/skin-db-recovery-path.txt').read_text().strip())
base=Path(os.environ['APPDATA'])/'github/Luxmc/data'
c=sqlite3.connect(root/'recovered.db')
c.execute('PRAGMA foreign_keys=ON')
now=datetime.now(timezone.utc).isoformat()
profiles=[]
for folder in (base/'instances').iterdir():
    game=folder/'.minecraft'
    pack=game/'modrinth.index.json'
    if pack.exists():
        data=json.loads(pack.read_text()); deps=data['dependencies'];name=data['name']; version=deps['minecraft'];loader='fabric';loader_version=deps.get('fabric-loader')
    elif folder.name=='5118cc6c-169d-4e51-bed0-23caa762ee78':
        name='Vanilla 1.8.9';version='1.8.9';loader='vanilla';loader_version=None
    else:continue
    count=len(list((game/'mods').glob('*.jar')))
    c.execute('INSERT OR IGNORE INTO profiles(id,name,mc_version,loader,loader_version,game_dir,created_at,updated_at,ram_mb,mod_count) VALUES (?,?,?,?,?,?,?,?,?,?)',(folder.name,name,version,loader,loader_version,str(game),now,now,4096,count))
    profiles.append({'name':name,'version':version,'mods':count})
auth=json.loads((Path(os.environ['APPDATA'])/'io.github.luxmc.Luxmc/auth.json').read_text())['current_account']
assert c.execute('SELECT 1 FROM accounts WHERE id=?',(auth['id'],)).fetchone()
expiry=auth.get('expiresAt')
expiry=datetime.fromtimestamp(expiry/1000 if abs(expiry)>100000000000 else expiry,timezone.utc).isoformat() if expiry else None
c.execute('UPDATE accounts SET skin_url=?,skin_variant=?,cape_url=?,access_token=?,expires_at=?,updated_at=? WHERE id=?',(auth.get('skinUrl'),str(auth.get('skinVariant') or 'classic').lower(),auth.get('capeUrl'),auth.get('minecraftToken'),expiry,now,auth['id']))
c.commit()
assert c.execute('PRAGMA integrity_check').fetchall()==[('ok',)]
assert not c.execute('PRAGMA foreign_key_check').fetchall()
c.close()
(root/'restored-profiles.json').write_text(json.dumps(profiles,indent=2))
print(json.dumps({'profiles':profiles,'accountRecovered':True,'integrity':'ok','foreignKeys':'ok'}))
