import subprocess,os,json,urllib.request
creds=subprocess.run(['git','credential','fill'],input='protocol=https\nhost=github.com\n\n',text=True,capture_output=True,env={**os.environ,'GIT_TERMINAL_PROMPT':'0','GCM_INTERACTIVE':'Never'},check=True).stdout
secret=dict(line.split('=',1) for line in creds.splitlines() if '=' in line)['password']
def api(path):
 path += ('&' if '?' in path else '?')+'fresh='+str(__import__('time').time())
 with urllib.request.urlopen(urllib.request.Request('https://api.github.com/repos/predabr/luxmc/'+path,headers={'Authorization':'Bearer '+secret,'Accept':'application/vnd.github+json','User-Agent':'Luxmc-release-check'})) as r:return json.load(r)
runs=api('actions/runs?per_page=8')['workflow_runs']
for r in runs:
 if (r['name']=='Release' and r['head_sha']==subprocess.check_output(['git','rev-parse','refs/tags/v3.2.0'],text=True).strip()) or (r['name']=='CI' and r['head_sha']==subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()):
  print(json.dumps({k:r[k] for k in ['id','name','status','conclusion','html_url','head_sha']}))
  print(json.dumps([{'name':j['name'],'status':j['status'],'conclusion':j['conclusion'],'step':next((s['name'] for s in j['steps'] if s['status']=='in_progress'),None)} for j in api('actions/runs/'+str(r['id'])+'/jobs')['jobs']]))
