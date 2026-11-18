from pathlib import Path
import os,json,re,sqlite3
root=Path(Path('docs/validation/skin-db-recovery-path.txt').read_text().strip())
c=sqlite3.connect(root/'recovered.db')
print('versions columns', c.execute('PRAGMA table_info(mc_versions)').fetchall())
base=Path(os.environ['APPDATA'])/'github/Luxmc/data/instances/5118cc6c-169d-4e51-bed0-23caa762ee78/.minecraft'
for path in (base/'logs').iterdir():
    if path.suffix=='.gz':continue
    text=path.read_text(errors='replace')
    print(json.dumps({'file':path.name,'length':len(text),'numericVersionMentions':sorted(set(re.findall(r'(?<![\w])(?:26\.[234]|1\.2[01]\.\d+)(?![\w])',text)))[:10]}))
print('saves', len(list((base/'saves').glob('*/level.dat'))))
c.close()
