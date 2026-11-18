import json
import os
import sqlite3
from pathlib import Path

database = Path(os.environ['APPDATA']) / 'github/Luxmc/data/luxmc.db'
with sqlite3.connect(database.as_uri() + '?mode=ro', uri=True) as connection:
    rows = connection.execute('SELECT recorded_at,metrics FROM performance_history WHERE profile_id = ?', ('639d81a1-950d-4419-bdac-be0b3c8f25cb',)).fetchall()
    print(json.dumps([{'recordedAt': time, 'pid': json.loads(metrics).get('pid'), 'kind': json.loads(metrics).get('kind')} for time, metrics in rows]))
