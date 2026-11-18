from pathlib import Path
import sqlite3,json
root=Path(Path('docs/validation/skin-db-recovery-path.txt').read_text().strip())
c=sqlite3.connect(root/'recovered.db')
for table in ['accounts','profiles','launch_logs','launch_log_lines','app_settings']:
    print(table,json.dumps(c.execute('PRAGMA table_info('+table+')').fetchall()))
print('profile ids',c.execute('SELECT DISTINCT profile_id FROM launch_logs').fetchall())
c.close()
