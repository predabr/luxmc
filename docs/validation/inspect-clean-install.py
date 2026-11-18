import json
import os
import pathlib
import sqlite3

root = pathlib.Path(os.environ["APPDATA"]) / "github/Luxmc/data"
database = root / "luxmc.db"
if database.exists():
    with sqlite3.connect("file:" + database.as_posix() + "?mode=ro", uri=True) as connection:
        names = [row[0] for row in connection.execute("SELECT name FROM sqlite_master WHERE type=?", ("table",))]
        print(json.dumps({"profileTables": [name for name in names if "profile" in name or "instance" in name]}))
        if "profiles" in names:
            columns = [row[1] for row in connection.execute("PRAGMA table_info(profiles)")]
            if "game_dir" in columns:
                print(json.dumps({"gameDirectories": [row[0] for row in connection.execute("SELECT game_dir FROM profiles")]}))
print(json.dumps({"dataChildren": [path.name for path in root.iterdir()] if root.exists() else []}))
