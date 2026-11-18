from pathlib import Path
import os, subprocess, sqlite3, json, hashlib
root = Path('docs/validation/skin-db-recovery-path.txt').read_text().strip()
root = Path(root)
cli = Path(os.environ['LOCALAPPDATA']) / 'Luxmc-build/sqlite-recovery-tools/sqlite3.exe'
archive = cli.parent / 'tools.zip'
assert hashlib.sha3_256(archive.read_bytes()).hexdigest() == '88b4659fe747896b853af10157316b4ade143553efb89c1c8ca7423a278dcc8b'
source = root / 'luxmc.db'
recovered = root / 'recovered.db'
with (root / 'recovery.sql').open('wb') as stream:
    result = subprocess.run([str(cli), str(source), '.recover --ignore-freelist'], stdout=stream, stderr=subprocess.PIPE, creationflags=subprocess.CREATE_NO_WINDOW)
print(json.dumps({'recoveryExit': result.returncode, 'diagnostic': result.stderr.decode(errors='replace')[:1000]}))
with (root / 'recovery.sql').open('rb') as stream:
    result = subprocess.run([str(cli), str(recovered)], stdin=stream, stdout=subprocess.PIPE, stderr=subprocess.PIPE, creationflags=subprocess.CREATE_NO_WINDOW)
print(json.dumps({'importExit': result.returncode, 'diagnostic': result.stderr.decode(errors='replace')[:1000]}))
connection = sqlite3.connect(recovered)
summary = {'integrity': connection.execute('PRAGMA integrity_check').fetchall(), 'tables': {}}
for (name,) in connection.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall():
    try:
        summary['tables'][name] = connection.execute('SELECT count(*) FROM "' + name.replace('"','""') + '"').fetchone()[0]
    except Exception as error:
        summary['tables'][name] = str(error)
connection.close()
(root / 'summary.json').write_text(json.dumps(summary, indent=2))
print(json.dumps(summary))
