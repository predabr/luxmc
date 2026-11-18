from pathlib import Path
import os, json, re
base=Path(os.environ['APPDATA'])/'github/Luxmc/data/instances'
for folder in base.iterdir():
    game=folder/'.minecraft'
    pack=game/'modrinth.index.json'
    info={'id':folder.name,'mods':len(list((game/'mods').glob('*.jar')))}
    if pack.exists():
        value=json.loads(pack.read_text()); info.update(name=value.get('name'),dependencies=value.get('dependencies'))
    log=game/'logs/latest.log'
    if log.exists():
        text=log.read_text(errors='replace')
        info['versions']=re.findall(r'Loading Minecraft ([\w.\-]+)(?: with ([\w ]+) ([\w.\-]+))?',text)[:3]
    print(json.dumps(info))
config=Path(os.environ['APPDATA'])/'io.github.luxmc.Luxmc/auth.json'
value=json.loads(config.read_text()); print('auth-store keys', list(value))
