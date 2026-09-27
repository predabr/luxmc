# Auditoria de capas, prévia 3D e interface

Revisão posterior: [correções após uso real](2026-09-24-correcoes-em-uso-real.md). O diagnóstico das faces da capa e a estratégia de wallpaper deste documento foram substituídos após os testes do usuário.

## Diagnóstico antes das alterações

Foram inspecionados o estado Git, AGENTS.md, dependências, AppearanceAgent.java, auth_save_appearance, a normalização Rust de capas, capeTextures.ts, SkinViewer3D, os containers do layout, as duas barras laterais, os dois sistemas de tradução e as telas de amigos/configurações. As alterações anteriores e o arquivo snake.html foram preservados.

- O agente Java entrega os bytes do PNG normalizado sem inverter coordenadas. O gerador de capas aplicava um espelhamento horizontal adicional na face externa (x=12..21, y=1..16), e a miniatura aplicava outro. A correção pertence ao gerador, não a um novo flip indiscriminado no servidor Java.
- A troca de skin ocultava o canvas, interrompendo a prévia antes de terminar o carregamento. Erros de formato depois de carregar a imagem não percorriam toda a cadeia de fallback. O ResizeObserver alterava o tamanho sem redesenhar, deixando o canvas vazio quando a animação estava pausada.
- A altura do layout dependia de h-full numa cadeia sem altura explícita no HTML. A lista de notícias tinha max-height de 340 px.
- A aba P2P renderizava o painel do túnel embarcado junto dos cards antigos de UPnP/entrada direta.
- O vídeo ficava visível no seeked, sem confirmar a apresentação de um frame decodificado. A prévia nas configurações mostrava apenas um ícone para vídeos.
- O português cobria as chaves inglesas existentes, mas faltavam 15 chaves usadas nas telas e 43 chaves existentes no espanhol. Havia também rótulos ingleses escritos diretamente no HTML.

## Alterações aplicadas

### Capas e avatar

O gerador agora mantém o atlas canônico sem o espelhamento extra. A miniatura usa a mesma orientação. Uma migração por comparação exata de pixels reconhece apenas capas geradas pela versão anterior e restaura a orientação ao carregar a seleção: não espelha automaticamente PNGs oficiais/importados nem interfere em regiões de outras faces/elytra. A normalização mantém as coordenadas proporcionais das capas HD e formatos OptiFine.

A seleção de capas está visível na tela principal, com miniaturas, estado selecionado, opção sem capa e importação rápida de PNG. A capa importada também recebe miniatura recortada da face externa. Skin/capa continuam sendo salvas juntas no SQLite pelo comando existente. O avatar passa a ser recortado da textura efetivamente escolhida, incluindo a camada externa da cabeça, antes de concluir o salvamento.

Fonte de verificação do mapa UV: implementação instalada de skinview3d em node_modules/skinview3d/libs/model.js, CapeObject e setCapeUVs. [Código do projeto skinview3d](https://github.com/bs-community/skinview3d/blob/master/src/model.ts). A orientação final em um mod que substitua o renderer ainda exige observação dentro desse jogo; nenhum novo flip foi colocado no agente Java.

### Prévia e layout

A seleção mantém a instância SkinViewer e seu canvas. O carregamento aguarda loadSkin e percorre URL, cache/base64 e textura local, inclusive quando a falha é de formato. Seleções canceladas não aplicam resultados atrasados. Resize e interação da câmera redesenham mesmo com animação suspensa. RAF, visibilidade, cancelamento e descarte de recursos continuam ativos.

O layout e a barra esquerda usam altura explícita da viewport, com navegação rolável e rodapé fixado pelo flexbox. Notícias usam o espaço restante com rolagem própria. Filtros mantêm fundo transparente, superfície translúcida e foco de teclado no contorno externo, sem o retângulo interno deformado. O helper de botões passou a usar os tokens do tema em vez de cores hexadecimais fixas.

### Multiplayer e vídeo

A aba P2P contém dois cards: hospedar e entrar. A hospedagem detecta LAN e só pede a porta quando não consegue determinar um único mundo. O convite continua privado e válido por uma hora. Entrar conecta o túnel e inicia a instância ativa em uma ação. Se o lançamento falhar, a sessão conectada permite tentar novamente ou sair. A lista social e as outras funcionalidades de amigos permanecem nas suas abas; comandos de compatibilidade continuam no backend.

VideoWallpaper mantém o primeiro frame no canvas e só revela o vídeo com requestVideoFrameCallback (com fallback para browsers sem essa API). O reinício é controlado pelo ended, com ocultação perto do final, em waiting e seeking. Cancelamento invalida callbacks de fontes anteriores. A prévia das configurações usa o mesmo componente em modo estático, carregando um frame sem reproduzir continuamente o vídeo.

### Português

Foram preenchidas as chaves ausentes e corrigidos rótulos da biblioteca, skins, notícias e estados de carregamento. As telas de capturas agora resolvem todas as chaves utilizadas. O atributo lang acompanha a escolha de idioma. Nomes de projetos, jogadores, marcas e termos técnicos permanecem próprios. Um teste verifica todas as chaves literais usadas em Svelte/TS, cobertura dos outros dicionários e compatibilidade dos parâmetros de interpolação. Os idiomas inglês e espanhol foram preservados.

## Validação

- pnpm check: zero erros e avisos.
- pnpm test:run: 52 testes aprovados.
- tests/audit.browser.cjs: canvas preservado durante trocas de skin, avatar em base64, contexto WebGL perdido/restaurado, fallback remoto, resize com animação pausada, sidebars em duas alturas, notícias até o rodapé, filtros transparentes, dois cards P2P, hospedagem/encerramento e conexão seguida de lançamento. IPC simulado para as ações da interface; a rede Rust não foi alterada nesta revisão.
- Capas: recuperação do atlas gerado antigo, preservação de atlas correto e coordenadas de uma textura HD assimétrica.
- tests/wallpaper.browser.cjs: primeiro frame RGBA [0,0,253,255], três reinícios reais, ocultação durante seek e pausa ao iniciar o jogo.

Os testes de navegador usam Chromium. Não representam garantia absoluta para todo codec/GStreamer/WebKitGTK, nem substituem a conferência visual da capa no Minecraft que o usuário está jogando. O código completo foi aplicado aos arquivos do projeto, sem placeholders.

## Código completo no projeto

- [Atlas e migração de capas](../../src/lib/utils/capeTextures.ts)
- [Normalização e avatar](../../src/lib/utils/textureImage.ts)
- [Prévia 3D](../../src/lib/components/ui/SkinViewer3D.svelte)
- [Tela de skins e capas](../../src/routes/skins/+page.svelte)
- [Layout](../../src/routes/+layout.svelte), [barra esquerda](../../src/lib/components/layout/Sidebar.svelte) e [barra direita](../../src/lib/components/layout/RightSidebar.svelte)
- [Tela de amigos](../../src/routes/friends/+page.svelte) e [painel do túnel](../../src/lib/components/friends/MeshPanel.svelte)
- [Wallpaper de vídeo](../../src/lib/components/visuals/VideoWallpaper.svelte) e [configuração com prévia](../../src/lib/components/settings/ThemeSection.svelte)
- [Filtros](../../src/lib/components/ui/GlassSelect.svelte), [botões](../../src/lib/components/ui/button.ts) e [estilos](../../src/app.css)
- [Português](../../src/lib/i18n/pt-BR.json) e [verificação de traduções](../../src/lib/i18n/i18n.test.ts)

## Retomada em 24/09/2026

A verificação de interface passou novamente, incluindo quatro seleções alternadas de Alex/Steve, aguardando cada chamada de salvamento antes da próxima seleção. A checagem Svelte teve zero erros/avisos e os 52 testes unitários passaram. O teste de vídeo atingiu o limite de tempo numa execução concorrente com a geração do frontend; repetido depois dessa etapa, completou três loops, preservou o primeiro frame e passou sem erros de página.

O teste de estabilidade repetiu oito ciclos de navegação e observou mais 125 segundos, com coleta de lixo antes das medições: heap de 23.422.624 para 26.241.924 bytes, nove salvamentos locais e nenhum erro de página. Passou no limite de crescimento definido pelo teste; não comprova ausência de vazamentos em sessões prolongadas.

Build nativo concluído: `pnpm tauri build --no-bundle`, perfil release otimizado, compilação Rust em 7m05s. Executável: `src-tauri/target/release/luxmc`. O frontend de produção foi gerado pelo mesmo comando. `git diff --check` também passou.
