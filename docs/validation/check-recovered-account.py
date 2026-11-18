from pathlib import Path
import os,sqlite3,json
root=Path(Path('docs/validation/skin-db-recovery-path.txt').read_text().strip())
c=sqlite3.connect(root/'recovered.db')
account=json.loads((Path(os.environ['APPDATA'])/'io.github.luxmc.Luxmc/auth.json').read_text())['current_account']
print({'currentAccountKeys':list(account) if isinstance(account,dict) else type(account).__name__,'databaseAccountMatches':bool(c.execute('SELECT 1 FROM accounts WHERE id=?',(account.get('id'),)).fetchone()) if isinstance(account,dict) else False})
print('foreignKeyProblems',len(c.execute('PRAGMA foreign_key_check').fetchall()))
c.close()
