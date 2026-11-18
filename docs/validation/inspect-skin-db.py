import sqlite3, os, json
from pathlib import Path
path = Path(os.environ['APPDATA']) / 'github/Luxmc/data/luxmc.db'
connection = sqlite3.connect(path.as_uri() + '?mode=ro', uri=True)
checks = {}
for query in ['PRAGMA integrity_check', "SELECT name FROM sqlite_master WHERE type='table'"]:
    try:
        checks[query] = connection.execute(query).fetchall()
    except Exception as error:
        checks[query] = str(error)
print(json.dumps(checks))
connection.close()
