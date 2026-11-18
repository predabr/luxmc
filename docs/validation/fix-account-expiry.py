from pathlib import Path
from datetime import datetime,timezone
import os,sqlite3,json
base=Path(os.environ['APPDATA'])/'github/Luxmc/data/luxmc.db'
backup=Path(os.environ['LOCALAPPDATA'])/'Luxmc-build/recovery'/datetime.now().strftime('%Y%m%d-%H%M%S-expiry')
backup.mkdir(parents=True)
c=sqlite3.connect(base)
saved=sqlite3.connect(backup/'before-expiry-fix.db')
c.backup(saved)
saved.close()
count=0
for identity,expiry in c.execute('SELECT id,expires_at FROM accounts').fetchall():
    if expiry is not None and str(expiry).lstrip('-').isdigit():
        epoch=int(expiry)
        normalized=datetime.fromtimestamp(epoch/1000 if abs(epoch)>100000000000 else epoch,timezone.utc).isoformat()
        c.execute('UPDATE accounts SET expires_at=? WHERE id=?',(normalized,identity))
        count+=1
c.commit()
assert c.execute('PRAGMA integrity_check').fetchall()==[('ok',)]
c.close()
print(json.dumps({'datesCorrected':count,'integrity':'ok','backup':str(backup)}))
