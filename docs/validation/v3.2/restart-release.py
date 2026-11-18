import subprocess,os,json,urllib.request,urllib.error
s=subprocess.run(['git','credential','fill'],input='protocol=https\nhost=github.com\n\n',text=True,capture_output=True,env={**os.environ,'GIT_TERMINAL_PROMPT':'0','GCM_INTERACTIVE':'Never'},check=True).stdout
token=dict(l.split('=',1) for l in s.splitlines() if '=' in l)['password']
def api(path,method,data=None):
 req=urllib.request.Request('https://api.github.com/repos/predabr/luxmc/'+path,data=None if data is None else json.dumps(data).encode(),headers={'Authorization':'Bearer '+token,'Accept':'application/vnd.github+json','User-Agent':'Luxmc-publisher','Content-Type':'application/json'},method=method)
 try:
  with urllib.request.urlopen(req) as r:return r.status,r.read().decode()
 except urllib.error.HTTPError as e:return e.code,e.read().decode()
sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
print('Tag update',api('git/refs/tags/v3.2.0','PATCH',{'sha':sha,'force':False}))
print('Cancel superseded draft build',api('actions/runs/37708684446/cancel','POST'))
print('Build corrected draft',api('actions/workflows/release.yml/dispatches','POST',{'ref':'main','inputs':{'tag':'v3.2.0','reuse_assets':False}}))
