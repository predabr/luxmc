import hashlib,pathlib
exec(open('docs/validation/v3.2/restart-release.py',encoding='utf-8-sig').read().split("sha=subprocess")[0])
meta=json.load(open('docs/validation/v3.2/release-draft.json',encoding='utf-8'))
a=next(x for x in meta['assets'] if x['name']=='Lux.MC.Launcher.exe')
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*args,**kwargs): return None
opener=urllib.request.build_opener(NoRedirect)
req=urllib.request.Request('https://api.github.com/repos/predabr/luxmc/releases/assets/'+str(a['id'])+'?fresh='+str(__import__('time').time()),headers={'Authorization':'Bearer '+token,'Accept':'application/octet-stream','User-Agent':'Luxmc-release-verifier'})
try:
 with opener.open(req) as r:data=r.read()
except urllib.error.HTTPError as e:
 if e.code!=302: raise
 with urllib.request.urlopen(urllib.request.Request(e.headers['Location'],headers={'User-Agent':'Luxmc-release-verifier'}),timeout=120) as r:data=r.read()
sha=hashlib.sha256(data).hexdigest()
assert len(data)==a['size'] and data[:2]==b'MZ' and 'sha256:'+sha==a['digest']
pathlib.Path('release-windows/GitHub-3.2.0.exe').write_bytes(data)
print(json.dumps({'bytes':len(data),'sha256':sha,'officialArtifact':True}))
