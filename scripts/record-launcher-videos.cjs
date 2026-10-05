const { chromium } = require('playwright-core');
const fs = require('node:fs');
const path = require('node:path');
const { setupLauncherDemo } = require('./launcher-video-fixture.cjs');

const directory = path.resolve('docs/media/2026-10-05/captures');
const base = 'http://127.0.0.1:1420';
const selected = new Set(process.argv.slice(2));
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
const shots = [
  { id: 'home', route: '/', title: 'Seu Minecraft, organizado', captions: ['Instâncias, versões e modpacks reunidos na biblioteca.', 'Escolha sua aventura sem perder o controle dos arquivos.'], shortDuration: 6, shortCaption: 'Sua biblioteca. Seus mods. Seu próximo mundo.', shortFocus: [95, 155, 1140, 800], action: async p => { await move(p, p.getByRole('button', { name: 'Expandir menu', exact: true })); if (await p.getByRole('button', { name: 'Expandir menu', exact: true }).count()) await p.getByRole('button', { name: 'Expandir menu', exact: true }).click(); await delay(1400); await move(p, p.locator('.home-instance-grid > div').first()); } },
  { id: 'library', route: '/instances', title: 'Informações claras para cada aventura', captions: ['Tempo jogado, mods e memória ficam visíveis em cada cartão.', 'Organize, personalize e escolha a instância que vai jogar.'], action: async p => { await move(p,p.getByRole('heading',{name:/Suas instâncias/}));await delay(1300);await p.mouse.move(1140,650,{steps:35}); } },
  { id: 'new-instance', route: '/instances', prepare: async p => { await p.locator('main').getByRole('button',{name:'Nova Instância',exact:true}).click(); await p.locator('#instance-name').waitFor(); }, title: 'Uma instância do seu jeito', captions: ['Comece com Vanilla ou escolha um loader compatível.', 'Defina nome, memória e versão antes de criar sua instância.'], focus: [490, 80, 970, 920], action: async p => { const name=p.locator('#instance-name'); await move(p, name); if(await name.count()) await name.fill('Minha próxima aventura'); await delay(1600); const fabric=p.getByRole('button', { name: 'Fabric', exact: true }); if(await fabric.count()) { await move(p,fabric); await fabric.click(); } } },
  { id: 'discover', route: '/mods', title: 'Explore mods e modpacks', captions: ['Descubra conteúdo de Modrinth e CurseForge.', 'Use filtros de versão e loader para encontrar opções compatíveis.'], shortDuration: 7, shortCaption: 'Modpacks, mods e conteúdo, com filtros para sua versão.', shortFocus: [85, 215, 1130, 800], action: async p => { const first=p.locator('.catalog-card').first(); await move(p,first); await delay(1000); await p.mouse.wheel(0,420); await delay(1600); await p.mouse.wheel(0,-420); } },
  { id: 'install-pack', route: '/mods', title: 'Escolha como instalar', captions: ['Abra o modpack e configure sua própria instância.', 'Selecione a versão disponível e a memória antes do download.'], focus: [410, 80, 1100, 930], prepare: async p => { const card=p.locator('.catalog-card').first(); await card.waitFor(); await card.getByRole('button', { name: /Criar|Instalar/ }).first().click(); }, action: async p => { const ram=p.getByRole('button', { name:'6 GB',exact:true }); if(await ram.count()){await move(p,ram);await ram.click();} await delay(1000); await move(p,p.getByRole('dialog').first()); } },
  { id: 'instance', route: '/instances/demo-vp', title: 'Cada instância tem seu espaço', captions: ['Veja versão, loader, memória e conteúdo instalado.', 'Mantenha seus modpacks separados e encontre as ferramentas da instância.'], shortFocus: [230, 100, 1050, 860], action: async p => { await move(p,p.getByRole('heading',{name:'Vanilla Perfected',exact:true}));await delay(1000);await p.mouse.wheel(0,350); } },
  { id: 'content', route: '/instances/demo-vp', title: 'Gerencie o conteúdo instalado', captions: ['Pesquise e organize mods sem sair da instância.', 'Recursos, shaders e datapacks têm suas próprias abas.'], prepare: async p => { await p.getByRole('tab',{name:'Mods',exact:true}).click(); await p.locator('main').evaluate(e=>e.scrollTop=470); }, action: async p => { await delay(900); await move(p,p.getByRole('tab',{name:'Shaders',exact:true}));await p.getByRole('tab',{name:'Shaders',exact:true}).click();await delay(1700); await p.getByRole('tab',{name:'Pacotes de recursos',exact:true}).click(); } },
  { id: 'tools', route: '/instances/demo-vp', title: 'Ferramentas, sem bagunça', captions: ['Controles, configuração, exportação e backups ficam reunidos.', 'Abra a ferramenta necessária e volte para sua instância.'], prepare: async p => { await p.getByRole('button',{name:'Ferramentas',exact:true}).click(); }, action: async p => { await move(p,p.locator('#instance-toolbox')); await delay(1400); const exportButton=p.getByRole('button',{name:/Exportar instância/}).first(); if(await exportButton.count())await move(p,exportButton); } },
  { id: 'skins', route: '/skins', title: 'Seu personagem, em 3D', captions: ['Visualize skins, movimento e capa no personalizador.', 'Importe sua textura e reaplique a aparência quando quiser.'], shortDuration: 8, shortCaption: 'Skins em 3D, com movimento e sua própria coleção.', shortFocus: [170, 270, 1000, 750], action: async p => { const run=p.getByRole('button',{name:'Correr',exact:true});await move(p,run);await run.click();await delay(1800);const walk=p.getByRole('button',{name:'Andar',exact:true});await move(p,walk);await walk.click();await delay(1400);const canvas=p.locator('[aria-label="Visualizador 3D de Skin"] canvas').first();const box=await canvas.boundingBox();if(box){await p.mouse.move(box.x+box.width*.5,box.y+box.height*.5,{steps:25});await p.mouse.down();await p.mouse.move(box.x+box.width*.76,box.y+box.height*.5,{steps:35});await p.mouse.up();} } },
  { id: 'namemc', route: '/skins', title: 'Encontre seu próximo visual', captions: ['O NameMC ajuda a descobrir e importar novas skins.', 'Use a seleção, um link de skin ou seu próprio arquivo PNG.'], shortDuration: 6, shortCaption: 'Encontre uma skin no NameMC e leve para sua coleção.', shortFocus: [410, 210, 1030, 740], prepare: async p => {await p.getByRole('button',{name:'Link da skin no NameMC',exact:true}).click();}, action: async p => { const input=p.getByRole('textbox').first();await move(p,input);await input.fill('https://namemc.com/skin/63455d7069b397c2');await delay(1700);const importButton=p.getByRole('button',{name:/Importar.*skin|Importar pelo link|Importar link/i}).last();if(await importButton.count()){await move(p,importButton);} } },
  { id: 'friends', route: '/friends', title: 'Sua turma, sempre por perto', captions: ['Veja quem está online e acompanhe a atividade dos amigos.', 'Seu perfil e código ficam ao lado da lista, com ações claras.'], shortDuration: 7, shortCaption: 'Amigos online, pedidos claros e convites no lugar certo.', shortFocus: [130, 105, 1100, 845], action: async p => {const online=p.getByRole('button',{name:/^Online/}).first();if(await online.count()){await move(p,online);await online.click();await delay(1800);const all=p.getByRole('button',{name:/^Todos/}).first();await move(p,all);await all.click();}} },
  { id: 'add-friend', route: '/friends?tab=add', title: 'Encontre o perfil certo', captions: ['Busque um jogador por nickname ou pelo código completo.', 'Confira o perfil antes de enviar o pedido de amizade.'], focus: [265, 195, 1110, 760], action: async p => {const input=p.getByRole('textbox',{name:/Nickname|jogador/}).first();await move(p,input);await input.fill('Alex');await delay(1600);const result=p.getByRole('button',{name:/Alex/}).first();if(await result.count())await move(p,result);} },
  { id: 'requests', route: '/friends?tab=pending', title: 'Convites fáceis de acompanhar', captions: ['Os pedidos recebidos mostram ações para aceitar ou recusar.', 'Convites enviados ficam identificados enquanto aguardam resposta.'], action: async p => {const accept=p.getByRole('button',{name:/Aceitar/}).first();if(await accept.count())await move(p,accept);} },
  { id: 'p2p', route: '/hosting', room: true, title: 'Uma sala para jogar juntos', captions: ['Participantes, convite e controles da sala ficam reunidos.', 'Use a mesma versão e mods compatíveis; esta sala é uma demonstração.'], shortDuration: 7, shortCaption: 'Sua sala, seus amigos. Demonstração do fluxo P2P.', shortFocus: [330, 160, 1120, 760], action: async p => {const members=p.locator('[data-room-member]');if(await members.count())await move(p,members.last());await delay(1700);const copy=p.getByRole('button',{name:'Copiar convite',exact:true});await move(p,copy);} },
  { id: 'settings', route: '/settings', title: 'Ajustes que você controla', captions: ['Resolução, comportamento do launcher e efeitos de som.', 'As preferências ficam organizadas por categoria e são persistentes.'], shortDuration: 6, shortCaption: 'Configurações organizadas para jogar do seu jeito.', shortFocus: [380, 130, 1090, 800], action: async p => { await move(p,p.getByRole('tab',{name:'Geral',exact:true})); await p.mouse.wheel(0,320); } },
  { id: 'appearance', route: '/settings', title: 'Seu launcher, seu estilo', captions: ['Escolha tema, cores, transparência e papel de parede.', 'Animações e efeitos podem acompanhar sua preferência de desempenho.'], shortDuration: 6, shortCaption: 'Cores, transparência, animações e wallpaper do seu jeito.', shortFocus: [430, 215, 1020, 760], prepare: async p => {await p.getByRole('tab',{name:/Aparência/}).click();}, action: async p => {await p.mouse.move(900,630,{steps:30});await p.mouse.wheel(0,460);await delay(1600);await p.mouse.wheel(0,-230);} },
  { id: 'java', route: '/settings', title: 'Preparação compatível com o jogo', captions: ['Gerencie o Java e a memória com opções compatíveis.', 'O launcher reaproveita bibliotecas preparadas para reduzir trabalho repetido.'], prepare: async p => {await p.getByRole('tab',{name:'Java',exact:true}).click();}, action: async p => {const auto=p.getByRole('button',{name:'Automático (recomendado)',exact:true});await move(p,auto);await p.mouse.wheel(0,270);} },
  { id: 'language', route: '/settings', title: 'Português, English, Español', captions: ['O idioma pode seguir o sistema ou uma escolha manual.', 'Português e espanhol são reconhecidos; outros idiomas usam inglês.'], prepare: async p => {await p.getByRole('tab',{name:'Idioma',exact:true}).click();}, action: async p => {await p.mouse.move(1000,550,{steps:35});} },
  { id: 'storage', route: '/settings', title: 'Saiba onde estão seus arquivos', captions: ['Confira o espaço usado por instâncias, Java e cache.', 'Encontre os dados do launcher e os atalhos de armazenamento.'], prepare: async p => {await p.getByRole('tab',{name:'Armazenamento',exact:true}).click();}, action: async p => {await p.mouse.move(920,700,{steps:30});await p.mouse.wheel(0,320);} },
  { id: 'login', route: '/', login: true, title: 'Sua conta, seu acesso', captions: ['Conecte Microsoft ou sua conta Luxmc para começar.', 'A autorização Microsoft acontece no navegador, sem informar a senha ao launcher.'], focus: [435, 165, 1070, 780], action: async p => {await move(p,p.locator('button.launcher-button--microsoft'));} },
  { id: 'worlds', route: '/instances/demo-vp', title: 'Seus mundos, organizados', captions: ['Encontre os mundos salvos dentro de cada instância.', 'Veja os detalhes do save e acesse as ferramentas de backup.'], prepare: async p => { await p.getByRole('tab',{name:'Mundos',exact:true}).click(); }, action: async p => { await move(p,p.locator('#instance-mundos')); await p.mouse.wheel(0,270); await delay(1500); await p.getByRole('button',{name:/^Backups de Saves/}).click(); } },
  { id: 'accounts', route: '/settings', title: 'Gerencie sua identidade', captions: ['Conta, perfil social e personalização ficam ao seu alcance.', 'Acesse o portal ou escolha a aparência da sua conta.'], prepare: async p => { await p.getByRole('tab',{name:'Contas',exact:true}).click(); }, action: async p => { await move(p,p.getByRole('heading',{name:'Gerenciamento de Contas',exact:true}));await p.mouse.wheel(0,180); } },
  { id: 'commands', route: '/settings', title: 'Atalhos para seu próprio fluxo', captions: ['Comandos personalizados podem acompanhar o início e o fim da sessão.', 'Configure esse comportamento por instância quando precisar.'], prepare: async p => { await p.getByRole('tab',{name:'Comandos',exact:true}).click(); }, action: async p => { await move(p,p.getByPlaceholder('ex: echo Iniciando Minecraft'));await delay(1500);await move(p,p.getByRole('button',{name:'Salvar comandos da instância',exact:true})); } },
  { id: 'privacy', route: '/settings', title: 'Escolha o que compartilhar', captions: ['Modo streamer, detalhes no Discord e preferências de privacidade.', 'Você controla as opções que fazem sentido para sua sessão.'], prepare: async p => { await p.getByRole('tab',{name:'Privacidade',exact:true}).click(); }, action: async p => { await move(p,p.getByRole('switch',{name:'Alternar Modo Streamer',exact:true}));await p.mouse.wheel(0,210); } },
  { id: 'installer', image: 'packaging/windows/sidebar-preview.png', title: 'Sua próxima aventura começa aqui', captions: ['Lux MC Launcher: biblioteca, personalização e amigos.', 'Instale, escolha uma instância e prepare sua próxima aventura.'], focus: [0, 0, 656, 1256] }
];

async function move(page, locator) {
  if (!(await locator.count())) return;
  const box=await locator.first().boundingBox();
  if(box)await page.mouse.move(box.x+box.width*.5,box.y+box.height*.5,{steps:35});
}

(async()=>{
  fs.mkdirSync(directory,{recursive:true});
  const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe'});
  const manifestPath=path.join(directory,'manifest.json');
  const manifest=fs.existsSync(manifestPath)?JSON.parse(fs.readFileSync(manifestPath,'utf8')):{shots:[]};
  try{
    for(const spec of shots){
      if(selected.size&&!selected.has(spec.id))continue;
      const normalDuration=spec.id==='installer'?8:spec.id==='skins'?14:10;
      const entry={id:spec.id,title:spec.title,normalTitle:spec.title,normalDuration,shortDuration:spec.shortDuration||0,shortCaption:spec.shortCaption||'',captions:spec.captions,shortFocus:spec.shortFocus||spec.focus||[230,170,1140,790],recordedAt:new Date().toISOString()};
      if(spec.image){
        const target=`${spec.id}.png`;
        fs.copyFileSync(spec.image,path.join(directory,target));
        Object.assign(entry,{path:target,type:'image',start:0});
      }else{
        const context=await browser.newContext({viewport:{width:1920,height:1080},recordVideo:{dir:path.join(directory,'raw'),size:{width:1920,height:1080}},locale:'pt-BR',reducedMotion:'no-preference'});
        const page=await context.newPage();
        const errors=[];
        page.on('pageerror',error=>errors.push(String(error)));
        const began=Date.now();
        await setupLauncherDemo(page,{login:!!spec.login,room:!!spec.room});
        await page.goto(base+spec.route,{waitUntil:'networkidle'});
        if (spec.login) await page.locator('button.launcher-button--microsoft').waitFor(); else await page.locator('main').waitFor();
        await page.waitForTimeout(1100);
        if(spec.prepare)await spec.prepare(page);
        await page.waitForTimeout(600);
        for(const toast of await page.locator('.fixed.right-4.top-4').getByRole('button',{name:'Fechar',exact:true}).all())await toast.click();
        await page.waitForTimeout(250);
        await page.addStyleTag({content:'.video-demo-cursor{position:fixed;width:16px;height:16px;border:2px solid rgb(var(--fg));border-radius:50%;background:rgb(var(--brand-500));box-shadow:0 0 0 5px rgb(var(--brand-500)/.16);pointer-events:none;z-index:2147483647;transform:translate(-50%,-50%);transition:width .12s,height .12s}.video-demo-label{position:fixed;right:18px;bottom:14px;z-index:2147483646;border:1px solid rgb(var(--border));background:rgb(var(--bg-elevated)/.88);color:rgb(var(--fg-muted));padding:5px 9px;border-radius:6px;font:600 11px system-ui;letter-spacing:.06em;pointer-events:none}'});
        await page.evaluate(()=>{const cursor=document.createElement('div');cursor.className='video-demo-cursor';cursor.style.left='960px';cursor.style.top='960px';document.body.append(cursor);document.addEventListener('mousemove',event=>{cursor.style.left=event.clientX+'px';cursor.style.top=event.clientY+'px';});const badge=document.createElement('div');badge.className='video-demo-label';badge.textContent='DEMONSTRAÇÃO · DADOS DE EXEMPLO';document.body.append(badge);});
        const start=(Date.now()-began)/1000;
        const captured=Date.now();
        await delay(800);
        if(spec.action)await spec.action(page);
        await delay(Math.max(0,(normalDuration+1)*1000-(Date.now()-captured)));
        await page.screenshot({path:path.join(directory,`${spec.id}-preview.png`)});
        const video=page.video();
        await context.close();
        await video.saveAs(path.join(directory,`${spec.id}.webm`));
        await video.delete();
        Object.assign(entry,{path:`${spec.id}.webm`,type:'video',start,errors});
        if(errors.length)throw Error(`${spec.id}: ${errors.join('; ')}`);
      }
      const existing=manifest.shots.findIndex(shot=>shot.id===entry.id);
      if(existing<0)manifest.shots.push(entry);else manifest.shots[existing]=entry;
      manifest.shots.sort((a,b)=>shots.findIndex(s=>s.id===a.id)-shots.findIndex(s=>s.id===b.id));
      fs.writeFileSync(manifestPath,JSON.stringify(manifest,null,2)+'\n');
      console.log(`Recorded ${entry.id}: ${entry.path}, ${normalDuration}s.`);
    }
  }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
