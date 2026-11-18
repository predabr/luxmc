# Scroll e conteúdo das instâncias — 2 de outubro de 2026

A tela de conteúdo recebeu navegação por tipo, busca, contadores, importação local, acesso à pasta e instalação pelo catálogo com a instância, a versão e o loader corretos. A referência visual foi a gestão de projetos da [Modrinth App](https://modrinth.com/app).

O launcher mantém a rolagem nativa. O observador de scroll é passivo e pausa temporariamente wallpapers e animações decorativas, mantendo o último quadro. Cards fora da área visível usam `content-visibility`; a lista compartilhada usa chaves estáveis, renderização limitada à área visível e correção da posição ao filtrar. Os botões compartilhados usam transições de cor.

Foram corrigidos cliques interceptados por linhas vizinhas, menus cortados no final da lista, resultados antigos de buscas concorrentes, paginação sem retorno ao topo, instalação acionada pelo teclado abrindo detalhes e tratamento de mods como modpacks por palavras no título. Congelar versões agora persiste por instância e impede sua atualização em lote. A inferência inicial de skins respeita o modelo selecionado; aplicar a aparência continua sendo uma ação explícita.

O backend extrai miniaturas de até 64 pixels, limita a memória do cache e invalida entradas quando os arquivos mudam. Os ícones são gravados no SQLite em uma transação. Os arquivos JAR e as skins originais são preservados.

| Verificação | Resultado |
| --- | --- |
| `pnpm check` | Zero erros e avisos |
| `pnpm test:run` | 76 testes passaram |
| `pnpm test:website` | 15 testes passaram |
| `cargo check --locked` | Passou |
| `cargo test --lib --locked` | 86 passaram; um teste de relay público ignorado |
| `python3 tests/installers.test.py` | Seis testes passaram |
| `pnpm test:browser:content` | Catálogo, 1.000 mods, filtros, menus, congelamento, ações em lote, shaders, criação NeoForge, laboratório e links passaram |
| Wallpaper no Chromium | Três loops, quadro preservado durante seek, biblioteca preservada |
| Importação de skins | PNG original preservado, 60.728 pixels coloridos e modelo persistido após reload |
| `pnpm tauri build --no-bundle` | Executável final compilado com sucesso |
| Conteúdo nativo final | 1.000 mods: 177 ms na primeira leitura e 35 ms com cache; JARs preservados, ativação/desativação e shaders passaram |
| Laboratório nativo final | Diagnóstico, restauração reversível de cápsula e importação de benchmark passaram |
| WebKitGTK | 1.000 mods, 12 linhas renderizadas, último item acessível, sem overflow horizontal; p95 de quadros de 45 ms |

No Chromium, o percentil 95 dos quadros do catálogo ficou próximo de 19 ms nesta máquina. As larguras de 960 e 1440 pixels foram verificadas, inclusive com paths Windows contendo espaços. Os testes de navegador usam respostas controladas; os testes Rust exercitam o backend. Isso não equivale a um teste de Minecraft ou do instalador em um PC Windows.

A medição WebKitGTK usa a interface real com respostas controladas e uma amostra de 60 quadros. O p95 de 45 ms não representa uma garantia de 60 FPS em todo hardware. O teste nativo usa o executável final e arquivos reais em diretórios temporários isolados, sem tocar nos dados pessoais do launcher.

A integração contínua agora instala Chromium e executa a suíte de conteúdo em Linux e Windows, além dos testes Rust de biblioteca nos dois sistemas. A compilação nativa Windows liga explicitamente PSAPI. O instalador NSIS, o WebView2, o lançamento real do Minecraft, drivers, contas Microsoft e atualizações no Windows ainda precisam de execução nessa plataforma. Nenhuma execução remota do CI nem publicação foi feita nesta sessão.

As alterações locais preexistentes foram preservadas. O executável instalado em `/usr/bin/luxmc` continua sendo o de 28 de setembro; para usar esta revisão, execute o novo binário ou instale um pacote reconstruído.

Executável validado: `src-tauri/target/release/luxmc`. Para abrir a revisão corrigida a partir da raiz do projeto: `./src-tauri/target/release/luxmc`.

Capturas: [mods em 1440 pixels](../visual/2026-10-02/instance-mods-wide.png), [mods em 960 pixels](../visual/2026-10-02/instance-mods-small.png), [shaders](../visual/2026-10-02/instance-shaders-wide.png).

Reproduzir os testes de navegador: `pnpm exec playwright-core install --with-deps chromium` e `pnpm test:browser:content`. Para o teste nativo isolado no Linux: `pnpm tauri build --no-bundle` e `node tests/content.native.mjs`. Para WebKitGTK com PyGObject: `/usr/bin/python3 tests/webkit-content.py`, com o servidor Vite na porta 1420.
