import struct,gzip,os,json,io
from pathlib import Path
class Nbt:
    def __init__(self,data): self.stream=io.BytesIO(data)
    def number(self,fmt): return struct.unpack('>'+fmt,self.stream.read(struct.calcsize('>'+fmt)))[0]
    def text(self): return self.stream.read(self.number('H')).decode('utf8',errors='replace')
    def value(self,tag):
        if tag in range(1,7):return self.number({1:'b',2:'h',3:'i',4:'q',5:'f',6:'d'}[tag])
        if tag==8:return self.text()
        if tag==9:
            kind=self.number('B');count=self.number('i'); assert 0<=count<=2000000
            return [self.value(kind) for _ in range(count)]
        if tag==10:
            result={}
            while True:
                kind=self.number('B')
                if not kind:break
                key=self.text();result[key]=self.value(kind)
            return result
        if tag in (7,11,12):
            count=self.number('i');assert 0<=count<=2000000
            self.stream.read(count*{7:1,11:4,12:8}[tag]);return None
        raise ValueError('NBT inválido')
    def root(self):
        kind=self.number('B');self.text();return self.value(kind)
if __name__=='__main__':
    base=Path(os.environ['APPDATA'])/'github/Luxmc/data/instances'
    for folder in base.iterdir():
        for level in (folder/'.minecraft/saves').glob('*/level.dat'):
            with gzip.open(level,'rb') as stream: value=Nbt(stream.read(32*1024*1024)).root()
            print(json.dumps({'instance':folder.name,'version':value.get('Data',{}).get('Version',{}).get('Name')}))
