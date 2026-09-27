# Luxmc — implementação do plano mestre de 26/09/2026

## Escopo e preservação

Implementação local do documento enviado pelo usuário. Nenhum push, publicação, tag remota ou exclusão de instância foi realizado. Foram mantidos os controles de aparência, modelos clássico/fino, capas, configurações, modos de exibição do catálogo, notícias do launcher e changelog.

O checkout já continha outras alterações. Este relatório descreve o trabalho desta revisão; não atribui a ela toda a diferença acumulada no Git.

## 1. Texturas, skins salvas e avatar

- Validação das dimensões de skins modernas, legadas e HD, e capas Minecraft/OptiFine proporcionais.
- Diferenciação entre formatos: 64×32 pode ser tanto skin legada quanto capa. A decisão usa também o nome do arquivo e a distribuição de transparência no atlas UV. Não se trata toda imagem 2:1 como capa.
- Importação identificada como capa alimenta a seleção de capa personalizada e informa o usuário. Não é adicionada às skins salvas.
- A seleção de itens salvos também valida a textura, permitindo corrigir capas importadas anteriormente no lugar errado.
- Steve e Alex deixam de ser inseridos automaticamente nas skins salvas. Na hidratação, os identificadores e nomes especificados no pedido são filtrados. Quando há alteração, o valor original é preservado na chave `luxmc_saved_skins_before_cleanup` antes da limpeza.
- Detecção do modelo fino em skins modernas; skins legadas não consultam regiões inexistentes da metade inferior. A escolha manual fica persistida no item salvo.
- Aplicação explícita mantida: selecionar é um rascunho; apenas “Aplicar skin e capa” persiste a aparência.
- O avatar usa o rosto padrão do modelo escolhido quando recebe dimensões inválidas ou um UV puro de capa. A aplicação corporal também rejeita capas.

Arquivos: `textureImage.ts`, `skins/+page.svelte`, `SkinViewer3D.svelte`.

Limite: uma textura opaca de 64×32 com nome genérico pode ser ambígua. Nenhuma heurística de dimensões identifica com certeza toda capa arbitrária sem também rejeitar skins legadas legítimas.

## 2. Visualizador 3D

- Física e animação suspensas durante carregamento de skin; reinício do relógio e da referência de rotação ao retomar.
- Delta limitado a 50 ms, verificação de valores finitos, amortecimento exponencial e limites de rotação.
- Orientação da capa mantida em `Math.PI + yaw`, apenas quando a capa está visível.
- Normalização angular sem laços potencialmente infinitos quando valores inválidos chegam à rotação.
- Pausa por visibilidade do documento, interseção com a tela, jogo em execução e modo de desempenho; descarte dos recursos no encerramento do componente.

## 3. Entrada e lançamento de instâncias

- Removidos o `pkill -f` genérico antes de lançar e a espera obrigatória de 500 ms. O lançamento não encerra outras JVMs por correspondência textual.
- A abertura dos detalhes carrega a versão instalada e a lista de mods. Recursos, shaders, datapacks, mundos, capturas, árvore de arquivos e detecção de Java são carregados conforme a aba utilizada.
- Cache por instância/seção e compartilhamento de requisições em andamento.
- Controle de geração para impedir que respostas de uma instância anterior substituam o conteúdo atual.
- Busca local com debounce de 150 ms; verificação automática do Shield e enriquecimento dos metadados adiados para não disputar a abertura da tela.
- Atualizar continua disponível e invalida o cache das demais seções.

## 4. Wallpapers e fluidez

- Preservação do caminho local original e compatibilidade com URLs antigas de assets e do servidor de mídia.
- Resolução específica para imagem e vídeo, incluindo fallback para asset local se a reprodução pelo servidor de mídia falhar.
- Mensagem de erro quando não é possível reproduzir o vídeo.
- Sincronização da classe de wallpaper na inicialização; reabrir respeita um fundo padrão escolhido, mesmo que haja um wallpaper personalizado salvo.
- Fundo principal persistente; painéis translúcidos sem impor desfoque individual a todos os botões.
- Desenho direto em canvas persistente, callbacks de novos quadros quando disponíveis, fallback por RAF, pausa e retomada, loop nativo e recuperação de reprodução paralisada.
- A versão encontrada durante a revisão criava o canvas somente na prévia, mas exigia sua existência para inicializar o vídeo principal. Esse caminho foi corrigido.

A comparação de desempenho da etapa inicial está em `2026-09-26-fluidez-cpu.md`. Ela mostrou redução de CPU, mas não comprovou redução de RSS. Não há alegação de “zero vazamentos” ou estabilidade por várias horas.

## 5. Notícias reais

O endereço antigo `https://launchercontent.mojang.com/news.json` retornou material cuja data mais recente era janeiro de 2024. A integração usa o feed oficial ativo `https://launchercontent.mojang.com/v2/news.json`, consultado e validado em 26/09/2026.

- Comando Rust assíncrono `minecraft_news`, também exposto no daemon, com timeout e limite de resposta.
- Wrapper tipado no frontend; validação de links HTTPS oficiais, datas, duplicatas e ordenação.
- Cache por 30 minutos, requisição compartilhada e fallback local de publicações reais, sempre mantendo as datas originais.
- Atualização manual e indicação clara de modo offline.
- Notícias do Minecraft na página e na barra lateral; notas do Luxmc e changelog mantidos em suas abas.
- Títulos e resumos recebidos são tratados como texto, sem execução de HTML do feed.

## 6. Listas e ciclo de vida

- Catálogo virtualizado nos modos grade e lista, com medição dos itens para suportar alturas diferentes.
- Logs virtualizados com medição dinâmica, evitando sobreposição de mensagens longas.
- Adaptador usa runes para o estado de apresentação, sem criar novos stores legados.
- Limite de até 20 linhas virtuais montadas. Isso se refere aos itens, não à soma de todos os elementos internos de cada card. Um card contém diversos nós; limitar a página inteira a 20 nós destruiria seus controles.
- O teste percorreu 200 resultados e 500 logs, incluindo os últimos itens.
- `holdUntilCrawlEnd` e as dependências pré-otimizadas do Vite foram preservados.

## Verificações

Evidências em `/home/pedro/.cache/luxmc-validation/2026-09-26-performance/`:

- `master-check.log`: Svelte/TypeScript com zero erros e zero avisos.
- `master-tests.log`: 59 testes frontend aprovados.
- `master-rust.log`: `cargo check --locked` aprovado.
- `master-rust-tests.log`: 66 testes Rust aprovados, um ignorado.
- `master-browser.log`: 16 verificações de interação aprovadas no WebKitGTK, com IPC simulado e componentes reais.
- `news-native.json`: comando nativo consultou o feed real e retornou 100 entradas; maior data 26/09/2026.
- `master-release.log`: build de produção e executável Tauri.
- `master-wallpaper.log`: verificação prolongada de quadros, loops, pausa inesperada e pausa durante jogo.

Os testes de interação usam uma conta de teste e não modificam a conta Microsoft do usuário. Esta revisão não repete uma partida completa nos modpacks, não certifica Windows e não garante compatibilidade com todos os codecs instalados no sistema.

### Resultado do wallpaper na revisão final

O teste de 180 segundos passou com 34 amostras, 34 imagens distintas e média de 27,6 quadros desenhados por segundo, incluindo as pausas provocadas. A skin permaneceu visível, a análise amostral não detectou ruído e a retomada depois de pausa inesperada e de jogo foi confirmada. Isso valida o período testado, sem extrapolar para sessões de várias horas.

### Observação do checkout

A verificação de espaços em branco dos arquivos desta revisão passou. A verificação global apontou uma linha vazia adicional já existente ao final de `src/lib/stores/account.svelte.ts`, fora das alterações desta revisão.

O caminho alternativo foi exercitado separadamente por 150 segundos, forçando indisponibilidade do servidor local e ausência de `requestVideoFrameCallback`: passou com 28 amostras, 26 imagens distintas e média de 30,1 desenhos por segundo. A prévia usou o poster em cache, ficou pausada e não definiu uma fonte de vídeo adicional. Log: `master-wallpaper-fallback.log`.

### Entrega local

O build de produção terminou com sucesso em 7 min 44 s. O executável `src-tauri/target/release/luxmc` foi aberto e a página inicial foi inspecionada visualmente com o wallpaper do usuário, as duas instâncias atuais e seus controles. Captura: `master-launcher.png`. O launcher ficou aberto para teste.

Na amostra final de 30 segundos, o launcher e seus dois processos auxiliares somaram 73,33% de um núcleo de CPU e aproximadamente 658,7 MiB de RSS. O valor é uma observação da tela inicial com vídeo 1080p ativo; a biblioteca mudou desde a primeira medição, portanto não é uma comparação controlada adicional. Os testes anteriores controlados demonstraram a redução descrita no relatório de fluidez. Ainda existe custo relevante de decodificação e composição do vídeo; não se promete consumo nulo nem desaparecimento de todo engasgo.

O modo `pnpm tauri dev` também foi compilado e iniciado durante a verificação. Não foi iniciada uma partida real nos modpacks nesta revisão.
