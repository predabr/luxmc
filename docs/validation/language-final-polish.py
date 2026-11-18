from pathlib import Path
import json,re
root=Path('src/lib/i18n')
pt=json.loads((root/'pt-BR.json').read_text(encoding='utf8'))
for language in ['en','es']:
    path=root/f'{language}.json'
    data=json.loads(path.read_text(encoding='utf8'))
    for key,value in data['ui'].items():
        source=pt['ui'][key]
        if re.search(r'\bcapa',source,re.I) and not re.search(r'modpack|imagem de capa|foto de capa|cobertura',source,re.I):
            if language=='en':value=re.sub(r'\bCovers\b','Capes',re.sub(r'\bcovers\b','capes',re.sub(r'\bCover\b','Cape',re.sub(r'\bcover\b','cape',value))))
            else:value=re.sub(r'\bportadas\b','capas',re.sub(r'\bportada\b','capa',value,flags=re.I),flags=re.I)
        data['ui'][key]=value
    data['app']['tagline']='Minecraft launcher for Windows and Linux' if language=='en' else 'Launcher de Minecraft para Windows y Linux'
    data['app']['cutscene']['readySubtitle']='Your next Minecraft adventure' if language=='en' else 'Tu próxima aventura en Minecraft'
    path.write_text(json.dumps(data,ensure_ascii=False,indent='\t')+'\n',encoding='utf8')
pt['app']['tagline']='Launcher de Minecraft para Windows e Linux'
pt['app']['cutscene']['readySubtitle']='Sua próxima aventura no Minecraft'
(root/'pt-BR.json').write_text(json.dumps(pt,ensure_ascii=False,indent='\t')+'\n',encoding='utf8')
path=Path('src/lib/components/instances/JukeboxModal.svelte')
source=path.read_text(encoding='utf8')
source=re.sub(r'title: uiText\("ui\.([^"]+)"\)',lambda match:'title: '+json.dumps(pt['ui'][match[1]],ensure_ascii=False),source)
path.write_text(source,encoding='utf8')
