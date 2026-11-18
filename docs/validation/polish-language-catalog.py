from pathlib import Path
import json,re,html

root=Path('src/lib/i18n')
pt=json.loads((root/'pt-BR.json').read_text(encoding='utf8'))
overrides=json.loads(Path('docs/validation/language-overrides.json').read_text(encoding='utf8'))
overrides.update({
    'Seu amigo cola o convite no Luxmc e entra pelo Multijogador.': ['Your friend pastes the invitation into Luxmc and joins through Multiplayer.', 'Tu amigo pega la invitación en Luxmc y entra desde Multijugador.'],
    'O Luxmc encontra a porta LAN ou você informa a porta exibida no jogo.': ['Luxmc detects the LAN port, or you can enter the port shown in the game.', 'Luxmc detecta el puerto LAN, o puedes introducir el puerto que aparece en el juego.'],
    'Conta Microsoft: após a sincronização, sua skin fica no perfil oficial de Minecraft e pode ser vista por outros jogadores, inclusive em outros launchers. Capas importadas são locais; somente capas oficiais da conta têm visibilidade universal.': ['Microsoft account: after syncing, your skin is saved to your official Minecraft profile and other players can see it, including through other launchers. Imported capes are local; only official account capes are visible to everyone.', 'Cuenta Microsoft: tras sincronizar, tu skin se guarda en tu perfil oficial de Minecraft y otros jugadores pueden verla, incluso desde otros launchers. Las capas importadas son locales; solo las capas oficiales de la cuenta son visibles para todos.'],
    'Aparência salva e skin sincronizada com Minecraft. Outros jogadores verão sua skin oficial.': ['Appearance saved and skin synced with Minecraft. Other players will see your official skin.', 'Apariencia guardada y skin sincronizada con Minecraft. Otros jugadores verán tu skin oficial.'],
    'Aparência salva no launcher. Falha na sincronização Microsoft: {{arg0}}. Clique em aplicar para tentar novamente.': ['Appearance saved in the launcher. Microsoft sync failed: {{arg0}}. Click Apply to try again.', 'Apariencia guardada en el launcher. Falló la sincronización con Microsoft: {{arg0}}. Pulsa Aplicar para intentarlo de nuevo.'],
    'Skin e capa salvas. Serão usadas no Minecraft aberto pelo Luxmc.': ['Skin and cape saved. They will be used when Minecraft is launched through Luxmc.', 'Skin y capa guardadas. Se usarán al abrir Minecraft desde Luxmc.'],
    'Mundo adicionado à lista de servidores desta instância como “Luxmc · Mundo de amigo”. Abra novamente a tela Multijogador para atualizar a lista.': ['World added to this instance’s server list as “Luxmc · Friend’s world”. Reopen the Multiplayer screen to refresh the list.', 'Mundo añadido a la lista de servidores de esta instancia como “Luxmc · Mundo de un amigo”. Vuelve a abrir Multijugador para actualizar la lista.'],
    'O Luxmc varre seu computador procurando instâncias do': ['Luxmc scans your computer for instances from', 'Luxmc busca en tu equipo instancias de'],
    'Sua aparência está aplicada.': ['Your appearance is applied.', 'Tu apariencia está aplicada.'],
    'Expulsar': ['Kick', 'Expulsar'],
    'Aplicar e sincronizar': ['Apply and sync', 'Aplicar y sincronizar'],
    'Aplicar e sincronizar com Microsoft': ['Apply and sync with Microsoft', 'Aplicar y sincronizar con Microsoft'],
    'Escolha de Capa HD': ['Choose an HD cape', 'Elegir una capa HD'],
    'Arquivo .PNG local': ['Local PNG file', 'Archivo PNG local']
})
def polish(source,value,language):
    value=html.unescape(value).replace('▁',' ').strip()
    if language=='es':
        value=re.sub(r'\bhabitaciones\b','salas',value,flags=re.I)
        value=re.sub(r'\bhabitación\b','sala',value,flags=re.I)
        value=re.sub(r'\bMultiplayer\b','Multijugador',value)
        if re.search(r'\bskin',source,re.I):value=re.sub(r'\bpieles\b','skins',re.sub(r'\bpiel\b','skin',value,flags=re.I),flags=re.I)
        if re.search(r'\bloader',source,re.I):value=re.sub(r'\bcargadores\b','loaders',re.sub(r'\bcargador\b','loader',value,flags=re.I),flags=re.I)
        if 'instância' in source.lower() and 'instancia' not in value.lower():value=re.sub(r'\bcasos\b','instancias',value,flags=re.I)
    if re.search(r'\bporta',source,re.I):
        value=re.sub(r'\bdoors\b','ports',re.sub(r'\bdoor\b','port',value,flags=re.I),flags=re.I) if language=='en' else re.sub(r'\bpuertas\b','puertos',re.sub(r'\bpuerta\b','puerto',value,flags=re.I),flags=re.I)
    return re.match(r'^\s*',source).group()+value+re.search(r'\s*$',source).group()
for language,index in [('en',0),('es',1)]:
    data=json.loads((root/f'{language}.json').read_text(encoding='utf8'))
    for key,source in pt['ui'].items():
        data['ui'][key]=overrides.get(source,[polish(source,data['ui'][key],language)]*2)[index]
    (root/f'{language}.json').write_text(json.dumps(data,ensure_ascii=False,indent='\t')+'\n',encoding='utf8')
for key,source in pt['ui'].items():pt['ui'][key]=html.unescape(source)
(root/'pt-BR.json').write_text(json.dumps(pt,ensure_ascii=False,indent='\t')+'\n',encoding='utf8')
for file in Path('src').rglob('*.svelte'):
    source=file.read_text(encoding='utf8')
    updated=re.sub(r'([\w-]+)="\{(uiText\("[^"\n]+"\))\}"',r'\1={\2}',source)
    if updated!=source:file.write_text(updated,encoding='utf8')
for file in list(Path('src').rglob('*.svelte'))+list(Path('src').rglob('*.ts')):
    if 'i18n' in file.parts or '.test.' in file.name:continue
    source=file.read_text(encoding='utf8')
    updated=re.sub(r'(\.toLocale(?:DateString|TimeString|String)\()([\'"])pt-BR\2',r'\1currentUiLocale()',source)
    if updated!=source:
        if file.suffix=='.svelte':
            if 'translateUi as uiText' in updated:updated=updated.replace('translateUi as uiText','translateUi as uiText, currentUiLocale',1)
            else:updated=re.sub(r'(<script[^>]*>)',r'\1\nimport { currentUiLocale } from "$lib/i18n/useTranslation.svelte";',updated,count=1)
        else:updated='import { currentUiLocale } from "$lib/i18n/useTranslation.svelte";\n'+updated
        file.write_text(updated,encoding='utf8')
