from pathlib import Path
import json,re,time,urllib.request,urllib.parse

path=Path('docs/validation/ui-translation-work.json')
entries=json.loads(path.read_text(encoding='utf8'))
for language in ['en','es']:
    pending=[entry for entry in entries if not entry.get(language)]
    batches=[]
    batch=[]
    length=0
    for entry in pending:
        if length+len(entry['source'])>2800 and batch:
            batches.append(batch);batch=[];length=0
        batch.append(entry);length+=len(entry['source'])+25
    if batch:batches.append(batch)
    for index,batch in enumerate(batches):
        original='\n'.join(f'[LUXMC{i:04d}] '+re.sub(r'{{arg(\d+)}}',r'ZXARG\1ZX',entry['source']) for i,entry in enumerate(batch))
        query=urllib.parse.urlencode({'client':'gtx','sl':'pt','tl':language,'dt':'t','q':original})
        for attempt in range(3):
            try:
                request=urllib.request.Request('https://translate.googleapis.com/translate_a/single?'+query,headers={'User-Agent':'Mozilla/5.0'})
                with urllib.request.urlopen(request,timeout=25) as response: data=json.load(response)
                translated=''.join(part[0] for part in data[0] if part[0])
                pieces=re.findall(r'\[LUXMC\s*(\d{4})\]\s*(.*?)(?=\[LUXMC\s*\d{4}\]|$)',translated,re.S)
                if len(pieces)!=len(batch):raise ValueError('Translation markers missing')
                for number,value in pieces:
                    entry=batch[int(number)]
                    value=re.sub(r'ZXARG\s*(\d+)\s*ZX',r'{{arg\1}}',value.strip(),flags=re.I)
                    original_params=sorted(re.findall(r'{{[^}]+}}',entry['source']))
                    if sorted(re.findall(r'{{[^}]+}}',value))!=original_params:raise ValueError('Translation parameters changed')
                    leading=re.match(r'^\s*',entry['source']).group()
                    trailing=re.search(r'\s*$',entry['source']).group()
                    entry[language]=leading+value+trailing
                path.write_text(json.dumps(entries,ensure_ascii=False,indent=2),encoding='utf8')
                print(f'{language}: {index+1}/{len(batches)}',flush=True)
                break
            except Exception as error:
                if attempt==2:raise
                time.sleep(2*(attempt+1))
