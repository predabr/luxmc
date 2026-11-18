from pathlib import Path
import os,gzip,re,json
folder=Path(os.environ['APPDATA'])/'github/Luxmc/data/instances/5118cc6c-169d-4e51-bed0-23caa762ee78/.minecraft/logs'
for path in folder.iterdir():
    if path.suffix=='.gz':
        text=gzip.open(path,'rt',errors='replace').read()
    else:
        text=path.read_text(errors='replace')
    versions=re.findall(r'(?:Minecraft version|Loading Minecraft|Minecraft Version)[: ]+([\w.\-]+)',text)
    versionArgs=re.findall(r'--version[= ]+([\w.\-]+)',text)
    print(json.dumps({'file':path.name,'versions':versions[:3],'versionArgs':versionArgs[:3]}))
