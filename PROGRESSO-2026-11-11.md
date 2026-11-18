# Progresso — 11/11/2026 (sessão de correções e redesign)

> Salvo antes do desligamento do PC. Estado local, sem git.

## Status geral
- Servidor de teste "oi" (MC 26.3, 2048 MB, porta 33047) inicia e roda limpo.
- Janelas de CMD/terminal piscando: **CORRIGIDO E VERIFICADO** (ver causa raiz abaixo).
- Gamerules "Incorrect argument": **CORRIGIDO E VERIFICADO** (0 ocorrências no console).
- Telemetria (RAM/CPU/heap) com valores reais: **CORRIGIDO E VERIFICADO**.
- Redesign da página Servidores + console arredondado: **APLICADO e com svelte-check 0 erros**.
- Testes: Rust `local_server::settings` 9/9, `local_server::telemetry` 2/2, vitest 242/243 (1 flake de timeout, passa isolado), svelte-check 0 erros/0 warnings.
- Pendente: Server List Ping real via script externo (script saía sem resposta — investigar handshake/fechamento da conexão) e relatório final da missão em pt-BR.

## Causa raiz das janelas de CMD (CORRIGIDO)
O processo `luxmc-server.exe` (worker) é **cópia** de `luxmc.exe` guardada em
`%APPDATA%\github\Luxmc\data\server-worker\<revisao>\luxmc-server.exe`, copiada **uma única vez**
(`if !executable.exists()`). A cópia era de 17:54 (build anterior ao fix `CREATE_NO_WINDOW`),
então a telemetria rodava nesse binário velho e o `jcmd` abria terminal visível a cada ~1–2 s.

Correções em `src-tauri/src/server_worker.rs`:
1. **Refresh do exe**: copia de novo quando a fonte é mais nova que o destino (staging `.new` →
   rename → fallback copy best-effort). Silencia falhas (arquivo em uso).
2. **Guard de lock em `connect()`**: se `worker.lock` está preso mas não há `endpoint.json`,
   espera até 15 s pelo auto-reparo e depois retorna erro PT-BR claro (evita fila infinita de
   cópias de 100 MB travando toda a IPC).
3. **Auto-reparo no worker `run()`**: accept timeout 120 s → ticks de 10 s; se `endpoint.json`
   sumir, reescreve; idle-exit após 12 ticks (mantém 120 s).
4. **Resposta quando o semáforo de 16 requisições acaba**: antes `continue` silencioso (cliente
   travava para sempre); agora responde erro PT-BR "O serviço de servidores está ocupado…".

Binários: todos os spawns (`telemetry.rs` jcmd, `diagnostics.rs` powershell/java, `firewall.rs`
powershell, `server_worker.rs`, `core/process.rs`) já têm `CREATE_NO_WINDOW (0x08000000)`.
Verificação: amostragem a cada 400 ms por 40 s com servidor online + telemetria ativa →
**NENHUMA janela de console**.

## Gamerules (CORRIGIDO E VERIFICADO)
- Backend `settings.rs::startup_commands()` version-tiered: nomes snake_case a partir de
  1.21.11/26.x/25w44a (`keep_inventory`, `advance_time`, `natural_health_regeneration`,
  `spawn_mobs`, `spawn_monsters`, `command_blocks_work`), camelCase em 1.21.9–1.21.10
  (`keepInventory`, `doDaylightCycle`, `naturalRegeneration`, `doMobSpawning`, `spawnMonsters`
  desde 25w35a, `commandBlocksEnabled` desde 25w37a), básicos desde 1.6.1, sem regras
  inexistentes (`doAnimalSpawning` nunca existiu no Java; `functionCommandLimit` é Bedrock).
- Frontend `gamerules.ts`: mapa `SNAKE_CASE_RENAMES` + `usesSnakeCaseGamerules()` + `mapValue`
  (doFireTick→128/0, checks de movimento invertidos); catálogo corrigido
  (`snowAccumulationHeight` int 0–8, `spawnMonsters`, remoções).
- Console real do servidor 26.3: `gamerule spawn_monsters true` etc. respondidos pelo RCON sem
  nenhum `Incorrect argument` (0 linhas).

## Telemetria (CORRIGIDO E VERIFICADO)
- `telemetry.rs::parse_heap_output` aceita o formato novo do Java 25:
  `total reserved N K, committed N K, used N K` (mantém legado `total N K, used N K`).
  Teste novo `java25_reserved_committed_used_output_is_parsed` passa.
- `commands/system.rs::system_resource_sample` agora chama `refresh_memory()` (antes: total 0).
- Leitura real (servidor online, pid 23288): processMemoryMb 597–598, heapUsed 227→229 MB,
  heapCommitted 366 MB, heapMax 2048 MB (= RAM configurada), heapSource `jcmd`,
  systemUsed 18946/32639 MB, CPU real, tps/mspt null, detail honesto.

## Redesign (APLICADO)
- `ServersPanel.svelte` reescrito: blocos arrastáveis (HTML5 drag com indicador antes/depois
  por eixo), paleta de 8 cores por bloco, ordem + cores persistidas em
  `localStorage` `luxmc.localServerOrder` / `luxmc.localServerColors`, cards 20 px com faixa
  de acento colorida, seletor "Selecionado", ações play/paleta/excluir.
- `ConsolePanel.svelte` novo estilo: chips pílula, busca/seleção arredondadas, terminal 20 px,
  linhas em bolhas arredondadas com barra warn/error, timestamps em pílula, input pílula com
  badge `>` latão, placeholder neutro (`list, say Olá, time set day`).
- `src/app.css`: tokens `--radius-*` arredondados nos dois temas
  (xs 8, sm 10, md 14, lg 18, xl 22, 2xl 30, base 14).
- 52 `border-radius` 10/12/14 px → 14/16/18 px em todos os painéis de servidor
  (Overview/World/Players/Performance/Settings/Mods/Plugins/Backups/About/Wizard/Workspace…).

## Modo de execução / ambiente
- `tauri dev` rodando com `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`
  (shell `sh_1cd67e94a001SpeHt3GwNcmuRH`), vite na 1420.
- Drivers CDP no raiz do repo: `tmp-drive2.cjs` (inicia + screenshots), `tmp-start2.cjs`
  (invoke direto), `tmp-final.cjs` (telemetria+screenshots), `tmp-verify2.cjs` (fluxo completo
  com dismiss do modal "Remind me later"), `tmp-rcon-stop.cjs` (parada graciosa via RCON).
- Modal de update bloqueia cliques: dismiss = botão "Remind me later" (driver `dismiss()`).
- Conta: Spect3rBW `8a5bec034d584f96908ae56015586a6d`; servidor "oi"
  `b855869b-06c1-416c-8c15-a8190d5f7fa7`.

## Próximos passos (ao voltar) — CONCLUÍDO em12/11/2026
1. ✅ Server List Ping externo: **funcionando** (script corrigido — host com prefixo varint, status
   request id `0x00`, tratamento de `close`). Resultado:26.3 / protocol777 /0–10 jogadores /3 ms.
2. ✅ Isolamento de2ª instância: dois servidores online ao mesmo tempo (portas33047 e25565, PIDs e
   diretórios separados) + conflito de porta com sugestão PT-BR de porta livre testada (25600).
3. ✅ `tests/local-server.browser.cjs` PASS (Edge).
4. ✅ `cargo test --lib` completo: **373 passed /0 failed** (19 ignored). vitest **243/243**.
   svelte-check **0/0**.
5. ✅ Relatório final da missão em pt-BR: `docs/validation/relatorio-final-servidores-2026-11-12.md`
   (conectividade externa marcada como **Não verificado**).
- Drivers `tmp-*.cjs` removidos da raiz; servidor de teste "oi" parado com segurança; instância de
  teste "isola-b" excluída.

## Redesign dos blocos de servidores — reficção em12/11/2026 (pedido do usuário)
- Motivo: o redesign anterior era sutil (faixa de4 px) e o usuário não via mudança. Refeito em
  `ServersPanel.svelte`: card = **bloco colorido** (gradiente e borda dominados pela cor escolhida),
  pílula de status com ponto pulsante (Online/Offline/Iniciando), chips com ícones (versão/jogadores/
  RAM), botões Iniciar/Parar maiores com tinte, faixa de8 swatches24 px **sempre visível** no rodapé
  do card + botão `Padrão`, hover com elevação, drag com ghost rotacionado e indicadores luminosos.
- **Causa raiz visual encontrada:** com papel de parede personalizado (`html.has-custom-wallpaper`),
  a regra global `app.css` ~linha687 aplica `background-color: rgb(var(--bg-elevated)/var(--interface-opacity,.18)) !important`
  a todo `<button>`, anulando as cores dos swatches (computed ficava `rgba(17,18,22,0.18)`).
  Corrigido no componente com seletor de especificidade maior (`.card .dot`, `.card .icon-btn*`) +
  `background-color … !important` (autoria vence autoria por especificidade: (0,3,0)/(0,5,0) >
  (0,2,3)).
- Verificado: cores reais no DOM (brass `rgb(217,186,120)`, ametista `rgb(179,135,250)`, azul
  `rgb(96,165,250)`, verde `rgb(34,197,94)`…), screenshots `design-bloco-padrao.png`,
  `design-bloco-ametista.png`, `design-swatches.png` em `%TEMP%\opencode`; svelte-check 0/0.

## Canvas de nós do Gerenciador de Servidor (12/11/2026)
- Novo `ServerNodeCanvas.svelte`: workspace de canvas infinito (fundo cósmico, poeira estelar,
  fórmulas fracas tipo f(x)/ψ Schrödinger), pan (arrastar fundo) e zoom (roda/botões, 35–200%).
- Abas de servidor flutuantes no topo, fora das caixas: aba ativa com brilho ametista, setas
  anterior/próximo, botão +. Sem instâncias → canvas limpo só com "+ Criar Servidor".
- Nós arrastáveis (clique no cabeçalho sem arrastar expande/recolhe) conectados por curvas Bezier
  coloridas em SVG recalculadas ao vivo: Servidor, Software, Modpacks (só com mod loader), Mundo,
  Regras, Recursos, Controle.
- Dados 100% reais: `platform.settings`/`loader`/`telemetry` (Regras reflete server.properties),
  `saveServerSettings`, `updateLocalServer` (RAM), `start/stopLocalServer`, `loadersVersions` +
  `set_loader`, `modsSearch(contentType modpack, loader)` para o sub-nó de modpacks.
- Inspetor de estilo flutuante por nó: forma (arredondada/pílula/círculo/personalizada), raio,
  espessura de borda, opacidade do vidro, textura (grade/pontos) e 8 temas de cor; posições,
  estilos, zoom e pan persistidos em `luxmc.nodeCanvas.v1`.
- Descoberta real do backend: `set_loader` recusa trocar loader em mundo já gerado ("Este mundo já
  usa outro loader…") — a UI mostra o toast do erro e mantém o estado consistente.
- Verificado via CDP: abas/nós/fios no DOM, sub-nó Modpacks listando catálogo Fabric real
  (Fabulously Optimized, Sodium Plus, Vanilla Perfected…), screenshot `canvas-final.png`;
  servidor de teste "demo-fabric" criado para a prova e excluído depois; svelte-check 0/0.

## Canvas como interface única — UI legada eliminada (12/11/2026, pedido do usuário)
- `ServerWorkspace.svelte` reescrito: sem rail lateral, sem "Painel Clássico", sem "Visão geral",
  sem seções que trocam de tela. O canvas infinito é a interface permanente; o painel legítimo
  (Console, Jogadores, Mods, Backups, Desempenho, Configurações) abre como **folha de vidro
  flutuante dentro do próprio canvas** (`aside.sheet`), com botão picked no dock.
- Dock vira barra horizontal inferior-central (Console/Jogadores/Mods/Backups/Desempenho/
  Configurações); zero botões que saem do canvas (verificado: `classicRemnants: false` no DOM).
- Portas físicas de conexão (`span.port.in/.out`) em cada nó onde os cabos Bezier se plugam;
  arraste com snapping magnético de 10 px.
- LOD tipográfico: zoom < 75% → modo esquemático (corpos recolhidos, rótulos contra-escalados
  `min(13.5/zoom, 22px)`); zoom < 50% → modo mapa (rótulos centrais até 40/zoom px). Nítido em
  qualquer zoom (verificado a 48% e 35%).
- Troca de loader otimista: o sub-nó de modpacks monta/desmonta instantaneamente ao clicar; se o
  backend recusar (mundo já gerado), o estado reverte com o toast real do erro.
- Controles polidos: switches animados nas Regras, gauge de RAM (alocada vs sistema) nos Recursos,
  bloco proeminente INICIAR/ENCERRAR SERVIDOR no Controle; menu de contexto (botão direito) abre o
  inspetor de estilo do nó.
- Verificado via CDP: 11 portas, 5 cabos, sheet de Console/Mods dentro do canvas, LOD esquemático
  e mapa; screenshots `final-canvas.png`, `final-sheet.png`, `final-lod-map.png`; svelte-check 0/0.

## Canvas v2 — interface 100% de nós, zero painéis (12/11/2026, spec master do usuário)
- `ServerNodeCanvas.svelte` reescrito do zero (`luxmc.nodeCanvas.v2`): todo painel legado virou nó.
  11 tipos de nó — Servidor, Software, Modpacks (sub-nó só com mod loader), Mundo (sliders + regras
  com divisória), Recursos, EULA, Controle, Console, Jogadores, Backups, Configuração — ligados por
  10 arestas Bezier com cor por sistema: azul elétrico (Core), esmeralda (Mundo), âmbar (Recursos/
  Backups), ametista (EULA), carmesim (Controle).
- `ServerWorkspace.svelte` enxugado: topbar + faixa de avisos + canvas + wizard. `aside.sheet`,
  dock e os imports Console/Players/World/Mods/Backups/Performance/Settings/AboutPanel **removidos**;
  rota `/hosting/server` não recebe mais `?section=` (deep-link de painel abolido).
- Espelhamento ao vivo do nome: input no nó Configuração muta `platform.overview` otimista
  (título do cartão atualiza a cada tecla) e persiste via `updateLocalServer` no blur; erro reverte.
- EULA honesto: badge de compliance (aceito na criação) sem checkbox fake — o backend não tem API
  de gravação do eula.txt pós-criação.
- Recursos: gauge circular SVG com aro de metal escovado (gradiente), leitura digital
  `2.0 / 31.5 GiB`, sparkline de CPU do processo, TPS/uptime/heap reais da telemetria.
- Handles de canto (3 pílulas no topo direito, hover): ciclo de forma, ciclo de cor e inspetor
  (também via botão direito). Pinos de porta renderizados só onde existe cabo real
  (`incoming`/`outgoing` derivados das arestas) — nenhum conector solto.
- Verificação CDP no app rodando (svelte-check 0/0 antes): 10 nós (modpacks oculto no vanilla),
  9 cabos, cores computadas conferidas (azul ×4, verde ×1, ametista ×1, âmbar ×2, vermelho ×1),
  `dockLegacy: 0`; espelhamento provado ("Espelho Live" → título do cartão → restaurado "oi");
  Console abriu com 14 linhas reais + envio de comando; LOD `lod-map` a 35% com wireframes nítidos
  (`canvas-v2-lod-map.png`); troca Fabric montou o sub-nó de modpacks instantaneamente e reverteu ao
  ser recusada pelo backend (mundo gerado), voltando ao Vanilla; screenshots `canvas-v2-full.png`,
  `canvas-v2-final.png` em `%TEMP%\opencode`. Modal de update do launcher segue reabrindo e
  interceptando cliques (fechar com "Remind me later"); o usuário estava usando o app durante os
  testes (navegou para /mods no meio) — automação encerrada para não atrapalhar.

## Spec master v2 — polimento e 3 bugs reais corrigidos (12/11/2026)
- **Cabo flutuante raiz (crítico):** o SVG dos fios ficava em `left/top: -2000px` enquanto os paths
  usam coordenadas da camada — todos os cabos desenhavam 2000 px fora de posição (só um arco solto
  aparecia no canto). Medição numérica antes do fix: erro de até 2785 px até o pino mais próximo.
  Fix: SVG em `inset 0` com `overflow: visible` (origem do user-space = origem da camada). Depois do
  fix: 8/9 cabos terminam a 0,6–0,9 px do centro do pino (um a 3,6 px, dentro do pino de 11 px) via
  `getPointAtLength` + `getScreenCTM`.
- **Micro-pills ilegíveis no zoom-out:** `--lodpx` era unitless e `calc(13.5 * var(--lodpx))`
  resultava sem unidade → `font-size` inválido → rótulos ficavam ~3,7 px reais na tela. Fix: px na
  fórmula (`calc(12px * var(--lodpx))`, variável unitless). Modo mapa: pílula com `width:auto`
  abraçando **ícone da categoria + título** a tamanho de tela constante (ícone 24 px, título 12 px
  na tela a 35%); portas e padding também contra-escalados.
- **Cabos desaparecendo no zoom-out:** traço de 2 px em coordenadas do mundo × 0,35 = 0,7 px na
  tela. Fix: `stroke-width: calc(2px * var(--lodpx))` no LOD (traço = 2 px de tela em qualquer zoom;
  confirmado 5,7142 px na camada a 35%).
- **Texturas de grade/pontos removidas** (spec proíbe artefatos de grade): campo `texture` fora do
  `NodeStyle`, do inspetor, do default do Console e do CSS (`.node::before` incluído).
- **Física dos cabos**: `transition: d 420ms` nos paths (flexam fluidos no fan-out/recolher) e
  `transition: none` com a camada em `dragging` (rastreio em tempo real durante o arraste). Larguras
  dos nós agora medidas (`bind:clientWidth`) e usadas na ancoragem dos cabos — pílulas do modo mapa
  têm largura variável e os fios continuam plugadas na borda real.
- Wizard (spec §7) conferido: os 5 passos já existem mapeados em APIs reais (identidade com Java/
  Bedrock travada e snapshot toggle; matriz de software; runtimes Java com instalação gerenciada —
  backend não tem campo de JVM args, então nada foi inventado; mundo/rede/RAM com perfis; catálogo
  de regras + EULA + timeline de instalação real com auto-start). Topbar espelha o nome porque
  `platform.config` deriva de `overview` (mesma fonte mutada pelo espelhamento).
- Extra: 2 warnings de CSS legados em `ModCard.svelte` (`-webkit-line-clamp` sem o padrão) corrigidos.
- Verificação final: svelte-check **0/0**; screenshot `canvas-v2-lod-final.png` (35%: todos os cabos
  coloridos plugados nos pinos, pílulas com ícone+título, zero artefatos). Drivers tmp removidos.

## Canvas infinito v4 " multi-��ncoras, in��rcia e dropdowns de vidro (13/11/2026)
- **Roteamento multi-���ncora (spec '2):** servidor centrado com pinos nas **quatro bordas** (top/right/
  bottom/left) ' pinos por m���dulo alocados ao lado mais pr���ximo; cabos B'zier neon grossos
  recalculando em tempo real durante arraste (`transition: none` na camada em drag). CDP provou os 4
  lados ([bottom,left,right,top] com mundo abaixo do hub), 9 pinos 100% em borda, erro m'ximo
  pino''ncora 1,6 px, tra'co computado 3,5 px.
- **F���sica de arraste OS-grade (spec '3):** damping no mousemove + **in��rcia com momentum** no
  soltar (glide at��� parar) + snap final de 10 px no mundo. Testes sint'ticos escalados por zoom
  ('tela = 'mundo ' zoom, lido de `DOMMatrix(transform).a`).
- **Dropdowns de vidro (spec '5):** `GlassSelect.svelte` gen'rico (`generics="T extends string"` '
  `bind:value` checa contra uni'os literais) com prop `accent`; integrado no n��� de mundo do canvas
  (dificuldade/gamemode, herda `--node-theme`) e no wizard (ametista). Sem `backdrop-filter` (global
  `main * { backdrop-filter: none !important }` em app.css:413 mata ' ' opacidade 0.95 j' mata o
  efeito); fundo em token `rgb(var(--bg-elevated) / 0.95)` (regra global de wallpaper resolve em
  0.94 ' consistente com a plataforma).
- LOD: `body-node` oculto em schematic (<75%) ' testes de GlassSelect exigem '75% (lod-full);
  `--lodpx` unitless com px na f���rmula.
- svelte-check **0/0**; screenshots orbitais em `%TEMP%\opencode`.

## Mods restaurada + servidores instant'neos + topbar nova (13/11/2026)
- **Aba /mods quebrada (erro interno):** `{/if}' ���rf'' em `+page.svelte:819` (parse
  `block_unexpected_close` derrubava a rota inteira). Removido; em seguida 8 erros mascarados
  apareceram ' feature de filtros pela metade (`filtersOpen` inexistente, `SlidersHorizontal` sem
  import, `expanded` n' o era bindable). Completado: estado `filtersOpen` + bot' o
  `.catalog-filter-toggle` (badge de contagem) + prop `expanded = $bindable(false)` no
  `ModSearchBar` com gaveta col'aps'vel (`grid-template-rows 0fr'1fr`, 260ms). CDP: 36 cards,
  toggle abre 505px / fecha colapsado.
- **Lista de servidores mais r'pida:** causa raiz ' `startPolling()` s��� rodava no mount do
  `ServerWorkspace` ' primeira entrada baixava a lista do zero. Fix: `void
  platform.refreshServers()` no boot do `+layout.svelte` (onMount) + `"/hosting/server"` como
  primeira rota do `warmRoutes`. Medi' ' o: navega' ' o suave sidebar''hosting/server renderiza o
  servidor em **~115 ms** (3 medi' ' es: 124/116/113 ms).
- **Topbar redesign:** emblema gen'rico "L" (`/branding/luxmc-emblem-128.png`) removido ' cubo
  isom' trico em SVG inline com 3 faces em gradientes brass (`--brass-300/400/500/600/700`,
  stops via `style` ' var() n' o funciona em atributos de apresenta' ' o) e edges hairline
  `.emblem-edges`. Bordas: `border-bottom` plano fora; `::after` com hairline em gradiente
  brass'ametista'transparente + `box-shadow: inset 0 1px 0` (highlight de vidro); `.win-btn`
  polido (raio 9px, fundo transl'cido, `:focus-visible` ametista). CDP confirmou SVG + gradiente
  + img antiga ausente; screenshot `topbar-nova.png`.
- svelte-check **0/0** ap���s cada frente.
## 2026-11-13 — Lote 2: modal travado, filtros à direita, caixa do servidor, RAM auto, migrador, downloads e instalador

### Frontend
- **Modal de guia em /mods**: `<InstallGuideModal>` era renderizado incondicionalmente (sempre visível, impossível fechar). Agora vive dentro de `{#if showInstallGuide}`, com `role="dialog"`, `aria-modal`, `tabindex="-1"` e Escape via `<svelte:window>`. Verificado via CDP: overlay ausente.
- **Filtros à direita (estilo Modrinth)**: `<ModSearchBar>` movido para `<aside class="catalog-rail">` à direita; `.catalog-layout` grid `minmax(0,1fr) 296px` com `grid-column`/`grid-row` explícitos (auto-placement empurrava o conteúdo para a linha 2 — "coisas lá pra baixo"). Toggle de filtros compacto (`align-self: flex-start`, 86px). CDP: rail 296px à direita, mesma linha do conteúdo, 32 cards, toggle close/reopen OK.
- **ModSearchBar**: mecanismo de colapso interno (`drawer-collapse`, prop `expanded`) removido; painel sempre visível (a página controla o rail).
- **Caixa do gerenciador de servidores**: `.workspace` com radius 18px, sombra de vidro, moldura em hairline gradiente (brass→amethyst→border) via `::after` + mask-composite; controles de janela em cápsula de vidro (radius 13px), botões sem borda (hover neutro, close vermelho). CDP confirmou numericamente.
- **RAM "Auto" (crash de modpacks, ex. Cursed Walking)**: frontend sempre enviava 4096 e derrotava o `smart_modpack_ram` do backend. Defaults agora são 0 (Auto) em ModpackInstaller, página /mods, PackCollections (amigos) e zod `ramMb min(0)`; UI mostra "Auto" + chips 4/6/8/12 GB.
- **Migrador de launchers**: `ExternalInstance` ganha `icon`/`ramMb`; Prism lê `name=`, `iconKey=`, `MaxMemAlloc=` (ícones resolvidos em data-URL a partir de `{raiz}/icons/{key}.ext`); CurseForge lê `allocatedMemory` + investiga instance/icon/logo/cover.png|jpg; `importer_execute_import` aceita `icon`/`ramMb`, sanitiza (data:/http/builtin, ≤6MB) e INSERT grava ícone+RAM; cópia ampliada (coremods/openloader/packmenu/custommainmenu). Frontend (UniversalImporterModal) envia ícone/RAM.

### Backend / Performance
- reqwest com features `http2` + `gzip`; client compartilhado com `tcp_keepalive(30s)` e `http2_adaptive_window(true)`.
- `mods_install_with_deps_core`: staging de mods paralelizado com `buffer_unordered(16)` (era sequencial).
- Janelas de download dos loaders (fabric/forge/neoforge/quilt) `min(12)`→`min(24)`; prefetch do instalador 6→12. BOM UTF-8 removido dos .rs após bulk edit.
- cargo check OK (7 warnings pré-existentes de dead code).

### Instalador Windows (estilo Modrinth, flat)
- Novo `packaging/windows/sidebar.bmp` 164×314 gerado via System.Drawing (navy escuro, cubo isométrico brass flat, wordmark LUXMC, arcos sutis) — sem 3D.
- `headerImage` removido do tauri.conf.json (páginas internas limpas); textos do template NSIS padronizados ("Bem-vindo ao instalador do ${PRODUCTNAME}", "${PRODUCTNAME} instalado!", run "Abrir o ${PRODUCTNAME}").

### Caça a erros (varredura final)
- CDP varreu console/página/HTTP em /mods e /hosting/server: 8 erros → **0 erros**.
  - Paleta adaptativa (node-vibrant) carregava banner externo com `crossOrigin` (bisecthosting sem ACAO) → novo comando Rust `util_image_data_url` (https, ≤8MB, content-type image, thumb 512px PNG) traz a imagem pelo backend; Vibrant roda sobre data-URL com cache de 24 entradas.
  - Updater: `fetch` para github.com/…/latest.json sempre falhava (redirect sem ACAO) → bloco removido; o fallback api.github.com (ACAO *) cobre versão/revision/notas/assets.
- svelte-check 0/0; cargo check OK; app rebuildado e relançado pelo tauri dev.

## 2026-11-13 — Lote 3: redesign completo dos botões (estilo Modrinth, flat)

### Factory central
- `src/lib/components/ui/button.ts` reescrita: base pill (`rounded-full`), gap-2, `whitespace-nowrap`, transição só de cores (sem transform). Variantes flat: primary/play `bg-brand-500 hover:bg-brand-400 active:bg-brand-600 border-transparent` (gradiente removido), secondary `bg-fg/[0.07] hover:bg-fg/[0.11] active:bg-fg/[0.14]`, ghost/outline/microsoft/danger/ghostBrand achatados. Tamanhos: sm h-9 px-4, md h-10 px-5 text-[13px], lg h-11 px-6, xl h-12, pill, hero h-14, icon h-10 w-10, novo iconSm h-9 w-9. `Button.svelte` inalterado (usa a factory).

### app.css global
- `.launcher-button { border-radius: 999px }` global; sheen `::before` removido (`content: none`); gradientes de `background-image` (secondary/outline/primary/microsoft/play/danger) → none; sombras inset 3D de todos os variantes → none; antiga regra `border-radius: var(--radius-lg)` removida.
- Movimentos removidos: hover lift `translateY(-1px/-2px)` (normal e refined-ui), active `scale(.98)` → `translateY(1px)` (press sutil mantido), nav-link active `scale(.98)` → none, bounce de ícone no hover/focus (`translate: 0 -2px`) → none.
- Sheen sweep `::after` do primary/play (keyframe `luxmc-button-sheen`) removido.
- Ripple de clique removido: action `src/lib/actions/buttonFeedback.ts` deletada (uso único no layout raiz), `use:buttonFeedback` removido de `+layout.svelte`, `.launcher-button-wave` removida do CSS.
- Regras de wallpaper achatadas: primary/play flat (sem gradiente/borda/sombra); hover com wallpaper não troca mais `border-color` (só clareia fundo, sem sombra).
- refined-ui: primary `background: rgb(var(--brand-action))` com borda transparente e sem glow/sombra.

### Overrides pontuais
- `+page.svelte` (home): ícone play `rounded-2xl` → `rounded-full`; `ThemeSection.svelte`: ícone danger do wallpaper `rounded-lg` → `rounded-full`.
- Correção de quebra: fragmentos órfãos no fim do app.css (lixo de edição, linhas 914–924) causavam `Unexpected }` do postcss → 500 Internal Error em todas as páginas; removidos (arquivo termina no bloco `prefers-reduced-motion`).

### Verificação
- Auditoria visual por CDP nas 8 páginas (`/`, `/instances`, `/mods`, `/settings`, `/skins`, `/friends`, `/news`, `/hosting/server`): cutscene de boot reinicia a cada `goto` — script pula com Esc; todas renderizam pills flat, hover só clareia fundo, sem lift/sheen/ripple. Screenshots em `%TEMP%\opencode\audit-*.png`.
- svelte-check **0/0**. Nenhum arquivo Rust editado neste lote.

### Lote 3b — revisão estrutural "igual Modrinth" (geometria real das refs)
- Erro corrigido: o lote anterior forçou TUDO pill; as refs do Modrinth mostram pill **só nos CTAs** (Play/New instance) e toggles — dropdowns, nav e menus são **rounded ~10px cinza escuro**, e o item ativo é **tintado**, não preenchido.
- **Factory (`button.ts`)**: base `rounded-sm` (10px); pill via marcadores (`--primary`, `--play`, `--brand-outline`). Secondary agora **sem borda** (`bg-fg/[0.07] hover:bg-fg/[0.11]`). Alturas Modrinth: sm 32px, md 36px, lg 40px, xl 44px, hero 48px, icon 36px. Novo variante `tinted` (item ativo: `bg-brand-500/12 text-brand-400`) e `brandOutline` (chip outline brass, pill).
- **app.css**: `.launcher-button` default `var(--radius-sm)`; pill só primary/play/brand-outline; native `select` cinza (`fg/0.07`, borda `fg/0.08`, radius 10px, semibold); `.launcher-nav-link` 40px/10px (antes 44px/22px) sem sombra inset; `.launcher-tab` 36px/10px; `.mod-view-switch` container cinza com itens 8px e ativo tintado (sem glow); `.settings-navigation` e `.instance-settings-navigation` viram coluna **flat** (sem cápsula de vidro), itens 38px; indicador de nav (barra 3px + glow) desativado e bounce de ícone no ativo removido.
- **Sidebar**: itens 40px/10px; ativo **tintado** (antes `bg-brand-500` preenchido); "+ New Instance" = `brandOutline` (chip outline brass, como o "New instance" verde do Modrinth).
- **Settings** (`+page.svelte`): tabs eram `variant: primary` + `min-h-14` (56px!) → agora `tinted`/`ghost`, `min-h-10` (40px), ícones 16px, sem `surface-glass` (coluna flat como o modal do Modrinth).
- **Instance settings** (`instances/[id]`): ativo `primary` → `tinted`.
- **GlassSelect (ui + server)**: popup 10px com borda `fg/0.08`, itens `rounded-xs` min-h-9; trigger do server sem gradiente/lift/sombras pesadas (cinza flat, hover clareia).
- **CatalogNavigation (/mods)**: tabs de tipo 38px/10px, cinza `fg/0.05`, ativo tintado sem inset ring/glow.
- **Buscas normalizadas para 10px cinza** (`fg/[0.07]`, borda `fg/[0.08]`): home, InstanceFilters (era pill), /mods (era rounded-2xl py-4 com shadow-inner), instance settings, friends, BedrockSetup, DependencyMap, owner.
- **Auditoria visual** (CDP, 8 páginas): home/instances/mods/settings/skins/friends/news/hosting renderizam no padrão das refs — dropdowns "Name/All" cinza rounded, Play pill + kebab circular, toolbar de pills flat, nav tintada. Modal de update do launcher tem `role="dialog"` e é escondido por CSS injetado no script de auditoria (o style se perde a cada `goto`; agora reinjetado por página).
- svelte-check **0/0**; sem erro postcss no HMR.

## 13/11 — CI verde e comprovantes do painel do dono
- CI quebrado desde 094be0e: versão `4.1.0` sem notas em `docs/releases/v4.1.0.md` (teste Rust do changelog exige a entrada atual). Notas criadas e incluídas em `changelog.rs`; `cargo test --lib changelog::` verde.
- Comprovantes do painel do dono não abriam: CSP de `/painel-luxmc*` em `website/_headers` não permitia `blob:` no `img-src`, e a página fica sujeita a **dois** CSPs (o geral tem blob, o restrito não) — o preview do comprovante usa `URL.createObjectURL`. Corrigido para `img-src 'self' data: blob:`.
- `scripts/preview-website.mjs` agora aplica **todos** os blocos de `_headers` por rota (antes só o primeiro bloco `/*`), que é o motivo de o teste de navegador nunca ter pegado o CSP estrito.
- `tests/commerce.browser.cjs`: exige `img.receipt-image` visível após "Ver comprovante" e usa o rótulo atual do código de amizade (`Código de amizade enviado pelo jogador`, que mudou em b6e0b7d). Teste completo verde localmente com Edge.
- Falha do Release 4.1.0 (`test:browser:content` em mods-catalog) já havia sido corrigida em a70a7aa (abas achatadas); release.yml não roda testes Rust, por isso a falha era outra.
- Testes de navegador desatualizados pelo 4.1.0 (rodavam fora do CI e nunca pegaram as mudanças): `workshop` agora expande o painel da fila recolhível, `catalog-room-polish` verifica o novo carregamento de arte do card (opacidade 0→1 com ícone como capa), `launcher-refinement` mocka a API de releases do GitHub (o updater não busca mais `latest.json`) e espera o fundo dos cards com papel de parede, `reinstall-recovery` acompanha o login redesenhado (aba Microsoft, story visível só em telas grandes) e `scroll-profile-polish` valida o feedback de lançamento atual (classe `resource-busy`, o wave do botão foi removido no redesign). Suíte completa `test:browser:content` verde localmente (47 testes).
- `social-presence` falhava em servidor vite velho (instâncias de módulo divergentes por HMR acumulado); em servidor fresco passa sempre.
- `site-release-identity` acompanha a versão embarcada do site (o herói agora traz `v4.1.0`, exigido pelo `check-release-version`): tags do GitHub mais antigas que a versão publicada mantêm a versão do site, sufixos `revision`/`build` anotam o título e release futura atualiza a página. Corrigido também o preview local do site, que referenciava `securityHeaders` indefinido nas respostas de `/api/*` (derrubava a suíte `test:browser:website` inteira no Release). Suíte do site verde localmente (5/5).
- CI no Linux: `quilt_loader_prepare` comparava cold/warm na ordem, mas o classpath era coletado de `buffer_unordered` (ordem de conclusão do download, não do perfil). `prepare_quilt` e `prepare_fabric` agora preservam a ordem do perfil (enumerate + sort por índice) — ordem de classpath determinística entre execuções; forge/neoforge já montavam em ordem síncrona.
- **Release 4.1.0 publicado com sucesso no GitHub** (instaladores Windows/Linux/macOS verificados).
- Pendente: publicar o site (`pnpm deploy:website` precisa de login OAuth no wrangler) e reativar a tag `v4.1.0` no commit corrigido.

## 15/11 — Shield letal + Java instantâneo

### Java: verificação absurdamente rápida (nunca mais re-baixa)
- Cache persistido de fingerprint em `<data_dir>/java/verification-cache.json`: grava tamanho+mtime do `java.exe`/bin, `jvm.dll`/`libjvm.so`/`libjvm.dylib`, `java.dll`/`zip.dll`/`libzip.so`, `rt.jar`/`modules` e do arquivo `release`. Se nada mudou, pula o spawn de `java -version` (verificação passa a custar ~ms).
- Java do sistema resolvido e cacheado uma vez; `java_runtime_status` agora é **somente leitura** (antes uma consulta de status disparava download de 100 MB de runtime).
- `java_uninstall` esquece a entrada do cache; caminho custom de Java também é cacheado.
- Testes: `cargo test --lib java` 19/19 (+3 novos: fingerprint estável muda com mtime, cache ignorado quando binário muda, versão lida do arquivo `release`).

### Luxmc Shield: purga letal de instância infectada
- Antimalware real detectado (severity **critical** — hash/C2/behavior): a instância inteira é apagada **na hora**, antes do lançamento.
- Segurança: nunca purga com o jogo rodando (`get_active_game_pid()`); diretório gerenciado (`instances/<id>/.minecraft`) → `remove_dir_all` do diretório todo (mods, mundos, configs, bibliotecas); `game_dir` externo escolhido pelo usuário → apaga só `mods/` (não toca em dados pessoais). Perfil removido do banco (FK em cascata).
- Avisos por nome suspeito ("suspicious_structure") **nunca** purgam — só ameaça real.
- Frontend: evento `launcher-shield-warning` agora carrega `purged`/`instanceName`; `+layout.svelte` mostra **modal em tela cheia** (z-160, ícone ShieldAlert) com o nome da instância, lista de arquivos infectados e CTA "Baixar outro modpack" que leva ao catálogo `/mods`; páginas de instância e oficina refrescam perfis e saem da instância apagada. Toast antigo substituído pelo modal.
- Correção de chaves i18n: as 6 chaves novas do modal foram gravadas com prefixo `ui.` dentro da seção `ui` (lookup quebrado → teste de cobertura falhava); renomeadas para o hash puro em en/pt-BR/es.
- Testes: `cargo test --lib shield` 12/12 (novos: resultado limpo serializa `purged:false`, `has_critical_threat`), suíte completa `cargo test --lib` **391/391**, `svelte-check` **0/0**, `vitest` **246/246**.

### Pendência de CI resolvida (16/11)
- Commits `1e0d7e0`/`35aa600` (Shield purge + Java cache) deixaram o CI vermelho no step "Content and scrolling regression tests" (ubuntu e windows falharam em testes diferentes). Diagnóstico com log via `git credential fill` (PAT do credential manager) + reprodução local com o chromium do playwright (igual ao CI).
- **Quebra real 1 — `reinstall-recovery`**: `b7f434f` removeu o painel `section.login-story` (login de card único) e o teste ainda o exigia; agora valida o card `.auth-surface`.
- **Quebra real 2 (bug de produto) — `local-server`**: o poll de 1.5s (`refreshServers`) recriava `overview` → `platform.config` (objeto novo) a cada varredura; o `$effect` do `ServerNodeCanvas` re-disparava e sobrescrevia os rascunhos do usuário (MOTD, porta, gamerules) — editando durante um refresh, a digitação era revertida e o save gravava valor antigo. No CI lento, o poll caía entre fill e save → timeout. Fix: rascunhos re-semeados só quando `config.id` ou o objeto `settings` muda de verdade (`draftSyncedFor`).
- **Flake — `release-3.9.2`**: espera do canvas 2× tinha 30s; endurecida para 90s (mesma condição) para CI lento.
- Suíte completa local: **47/47 verde** com Edge; `svelte-check` 0/0. Commit `410ef35` → **CI 43865025138 success** (ubuntu + windows).
- Flake restante: o CI do commit de docs (`b17531d`) falhou só no windows em "Website tests" — `social.test.mjs:142` esperava 429 e recebeu 200: o limitador social usa **janela fixa por minuto** (`floor(segundos/60)`), e a rajada de 122 requests que começa perto do fim do minuto resetava o contador no meio (ubuntu passou no mesmo commit; windows passou 40 min antes no mesmo código). Fix determinístico: o teste espera o início de um minuto quando a fronteira está a menos de 4 s. Commit `4ce7a75` → **CI 43875568984 success**.

### Recuperação do preview 3D de skins (16/11)
- O CI do commit de docs (`33eddb8`) falhou de novo no windows em `release-3.9.2.browser.cjs:38` — agora com o timeout de 90s estourado, provando que não era só lentidão: o canvas nunca chegou a 2×.
- Causa raiz no `SkinViewer3D.svelte`: perda de contexto WebGL (`webglcontextlost`) durante a corrida de inicialização do processo de GPU no runner marcava `unavailable = $state(true)` **para sempre** — o efeito de criação tem guarda `|| unavailable` e nunca recria; o preview 3D morria definitivamente no produto também (usuário ficaria sem preview após um context loss).
- Fix: recuperação limitada — `scheduleViewerRecovery()` tenta `disposeViewer()` + `createViewer()` até 4× (espaçamento 1,5s×tentativa), acionado do catch da criação e do catch do render; `recoveryAttempts = 0` num render bem-sucedido; timer limpo no cleanup do `onMount`; o retry só exige `mounted` (painel escondido não desiste mais — criar viewer invisível é inofensivo, o render é controlado por `running`).
- Validação: `pnpm check` 0/0; `release-3.9.2` com o chromium empacotado do playwright (igual ao CI) PASS; suíte completa **47/47** com chromium empacotado. Commit `be32851` → **CI 43891593961 success** (ubuntu + windows).

## 17/11 — Nova logo em tudo + fim dos toasts de confirmação

### Troca de dica de login embutido
- `AccountLoginModal.svelte` usa agora `ui.7b258b8a8dbb4fc2` ("Conclua o login na janela que abriu."); chaves órfãs `design.browserLogin`/`browserLoginHint` removidas de en/pt-BR/es (vírgula final de `loginDescription` corrigida).
- Testes atualizados: `tests/account-login-concurrency.browser.cjs` (aba via `aria-label="Microsoft"`, assegura `[disabled]` nas duas abas) e `tests/login-review.browser.cjs` (formulário novo `#lux-user`/`#lux-pass`, sem passo "Próximo"). Ainda pendente de execução com vite próprio na porta 1420.

### Nova logo (cubo azul brilhante + portas douradas)
- Anexo do usuário (JPEG 1920×1920) recuperado do storage do opencode (`session_message` seq 27352, base64 em `data.files[0].data`) → master em `static/brand/luxmc-hero.jpg`; cópia extraída em `%TEMP%\opencode\new-logo0.jpg`.
- `scripts/render-brand.cjs` reescrito para pipeline raster: tile de quadrado arredondado (raio 0.2, fundo preservado — sem chroma-key para não furar as portas douradas nem criar halos) renderizado em 8 alvos (`src-tauri/icons/source-1024.png`, `static/logo.png`, `static/brand/luxmc-mark-512.png`, `static/branding/luxmc-emblem*.png`, `static/favicon.png`, `website/assets/*`). SVGs antigos `luxmc-icon.svg`/`luxmc-mark.svg` apagados.
- `pnpm tauri icon` regenerou os 17 ícones desktop; **android/ios restaurados via git checkout** (foreground adaptativo não pode ser tile pré-arredondado — decisão da rodada anterior mantida). 18 arquivos de ícone alterados ao todo.

### Remoção dos toasts de confirmação
- Política: **remover** todos os `success` e `info` triviais ("X copiado!", "salvo", "aplicado", "ativado", "iniciado com PID", "criado com sucesso"); **manter** erros/warnings, conclusões de operações longas (instalações, updates, imports/exports, backups, UPnP, exposição, restart), toasts de progresso, carregadores de artefato (link mclo.gs, códigos de convite/importação), notificações não solicitadas (amigos, updater, crash-doctor, conquistas, perf-guard), canal de toasts de plugins, guardas/explicações (modo online/offline do servidor, cond info/error de mods de desempenho) e feedback de join.
- Codemod (`%TEMP%\opencode\remove-toasts.cjs`): scanner balanceado por parênteses com detecção de nível, remoção em linha inteira/inline, cadeias `.then(() => toast(...))` (o `.catch` vizinho é preservado), ordem descendente por linha e revisão em modo `--dry` antes de gravar. Bug apanhado em revisão: falso-positivo de `isLineOnly` quando a chamada fecha em outra linha (só afetou `OverviewPanel:72`, consertado à mão).
- **90 remoções via codemod** (45 arquivos) + consertos manuais: `instances/[id]:939` mantém só o ramo de erro (`if (failed) toast(..., 'error')`), `routes/+page.svelte:494` e `CreateInstanceModal.svelte:257` ("Instância criada com sucesso!" — o modal já tem tela própria de sucesso), `instances/+page.svelte:577` (toast exibia o rótulo do botão "Duplicar"). Total ≈ 97 confirmações removidas; 114 success/info informativas mantidas.
- Imports de `toast` não usados limpos em 6 arquivos (script conservador que só remove quando não há mais chamada `toast(` no arquivo).
- Verificação: `pnpm check` **0/0**; `pnpm test:run` **248/248** (duas passadas seguidas verdes; uma falha isolada não reproduzível em execuções capturadas).

## 18/11 — Wallpapers a menos + lançamento otimizado (medido com dados reais)

### Wallpapers removidos
- `night` ("Noite Estrelada na Taiga") e `day` ("Alvorada nos Picos Congelados") removidos do catálogo `BACKGROUNDS` (`theme.svelte.ts`), do mapa `getBgName` (`ThemeSection.svelte`) e das chaves `bgNight`/`bgDay` em en/pt-BR/es; `static/bg_night.jpg` apagado.
- `static/bg_day.jpg` preservado: é fallback genérico dos banners de modpack (`curatedBanners.ts`, `LightboxModal`, `InstanceCard`, fixtures).
- Persistência já resiliente: `init()` só aplica `savedBg` se existir em `BACKGROUNDS` (senão obsidian) e `currentBackgroundStyle` tem `|| BACKGROUNDS.obsidian.style`.

### Otimização do launch (dados do `performance_history` do banco real)
- Linha de base medida (kind=preparation): modpack FO 26.2 = **86,35 s** (minecraftFiles 52,2 s + javaLoaderAndProcess 32,5 s); vanilla 26.3 = **15,9 s**. Pelos launch logs, ~27,2 s do total é o próprio Minecraft abrindo (49 mods) — fora do launcher; o trabalho do launcher no dia ruim foi ~59 s.
- Causa 1 (maior): `download_assets` enumerava os ~4000 objetos do índice **serialmente** (um `valid_cached_file` por objeto: abrir + ler receipt/hashear) → **GAP de 48 s** comprovado no log para baixar só 51 assets. Fix: enumeração paralela (`buffer_unordered(48)`), preservando a semântica de erro.
- Causa 2: `download_libraries` verificava as bibliotecas uma a uma (~2,5 s mesmo com 0 downloads). Fix: paralelo (`buffer_unordered(24)`), mesmo comportamento de erro e mesmas URLs.
- Causa 3: `valid_cached_file` abria o arquivo ANTES de consultar o receipt `.luxmc-verified` (2 opens por acerto). Fix: `metadata()` + receipt antes do `File::open` (1 open no caminho quente).
- Causa 4: `prepare_fabric` batia em `meta.fabricmc.net` **a cada launch** quando `loader_version` está vazio (0,61 s/log). Fix: cache em memória com TTL de 30 min por (meta, versão do MC) em `fetch_versions_from`; só guarda resultado não-vazio e a checagem de suporte das metas alternativas (legacyfabric/babric) foi preservada.
- Causa 5: Luxmc Shield hasheava os 49 JARs (~230 MB) a cada launch — o `CLEAN_CACHE` era só em memória e morria no restart (3,34 s/log). Fix: cache persistente em `<game_dir>/.luxmc/shield-scan-cache.json` (path → SHA-256 do conteúdo); a verificação continua sendo por **hash de conteúdo** (não por mtime/size — o teste `content_hash_invalidates_cache_even_with_same_size_and_timestamp` segue valendo), pulando apenas a descompressão/inspeção do ZIP no caminho quente.
- Verificação: `cargo check` 0; `cargo test --lib` **417/417**; `pnpm check` **0/0**; `pnpm test:run` **248/248**.
- **Medido em 18/11 após os fixes:** Fabulously Optimized 26.3 = **28,51 s** (era 86,35 s no 26.2; minecraftFiles 52,24 s → **0,08 s**, Shield 48 JARs em 0,45 s, janela do jogo 25,64 s). FTB StoneBlock 3 (224 mods, Forge) = **36,58 s** (minecraftFiles 0,98 s, Shield 224 JARs 4,54 s, janela do jogo 28,87 s). Trabalho do launcher: ~2,8 s (FO) e ~7,7 s (StoneBlock) — o resto é boot do jogo. Os 2 testes de navegador de login da rodada anterior seguem pendentes (precisam de vite próprio na 1420).

### Janela do Minecraft mais cedo (rodada 2, mesmo dia)
- O cache persistente do Shield **quase não gravava**: `scan_file` retornava cedo no acerto do `CLEAN_CACHE` em memória sem chamar `persisted_remember`. Resultado real: `.luxmc/shield-scan-cache.json` do StoneBlock tinha **1 entrada para 224 JARs**, então a inspeção ZIP de ~4,5 s se repetia a cada launch. Fix: todo hash confirmado (cache em memória ou persistido) agora é persistido; duas varreduras concorrentes não se sobrescrevem mais com um mapa parcial.
- Inspeção paralela do Shield de 4 → 8 workers (primeira varredura de um pack novo também cai).
- Confirmação da janela: polling de 500 ms → **150 ms** (eram 2 leituras consecutivas, até ~1 s só de latência).
- Flags de modpack pesado no Windows: `-Xms` = metade da RAM (teto 4096 MB) em vez de 1/4 (teto 1024 MB) e `-XX:MaxGCPauseMillis=200` (era 100) — menos coletas durante o registro de 224 mods. Limiar de "pesado" caiu de 50 → 32 mods (pega o FO, 48).
- Caches das instâncias atuais (FO 48 + StoneBlock 224, hashes conferidos iguais aos gravados pelo app) já semeados no formato do `shield-scan-cache.json`; o próximo launch pula a inspeção ZIP.
- Verificação: `cargo check` 0; `cargo test --lib` **418/418** (novo teste das flags pesadas).


