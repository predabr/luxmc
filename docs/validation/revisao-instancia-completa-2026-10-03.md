# Instância completa, instalador e fluidez — revisão de 3 de outubro de 2026

A interface final usa uma única barra de abas: Mods, Pacotes de recursos, Shaders, Datapacks, Mundos, Screenshots, Arquivos e Configurações. Apenas o painel selecionado é montado. A navegação por teclado usa setas, Home e End. A versão com todas as seções empilhadas foi substituída a pedido do usuário. Configurações e ferramentas ficam na própria aba de Configurações, com o laboratório expansível. Os dados dos painéis são carregados quando necessários. Capturas finais: [abas](../visual/2026-10-03-abas/). A correção foi recompilada com `pnpm tauri build --no-bundle` em 8 min 01 s e a janela do executável atualizado foi aberta.

A mensagem “Nenhum mod instalado nesta instância” foi removida. Recursos, shaders e datapacks têm ações próprias para explorar, abrir pasta, importar e excluir, com o tipo correto enviado ao backend. A lista de mods permanece virtualizada. A coluna de ações tem espaço para os botões e um interruptor de 44 pixels; o ícone de excluir tem 16 pixels e um alvo de 40 pixels. O filtro pixelado foi retirado dos ícones de mods. O backend prioriza a imagem do próprio JAR, escolhe a resolução adequada nos metadados e normaliza caminhos. As imagens são limitadas a 128 pixels, com redução para 64 quando necessário para manter o tamanho do transporte. Os JARs permanecem intactos.

O catálogo renderiza apenas as linhas próximas da área visível e usa a rolagem principal, sem outro painel de rolagem. Ícones pequenos não são esticados como banners. O componente de imagem aplica suas dimensões ao contêiner e mantém proporções. A prévia do personagem continua animando durante a rolagem; wallpapers decorativos ainda podem pausar. O Linux deixou de forçar `WEBKIT_DISABLE_DMABUF_RENDERER=1`; opções de ambiente e o modo explícito de renderização por software continuam disponíveis. O DMA-BUF é o caminho padrão descrito na [documentação gráfica do WebKit](https://docs.webkit.org/Ports/WebKitGTK%20and%20WPE%20WebKit/Graphics.html).

O instalador recebeu cabeçalho com identidade do pack, nome da instância, presets e seletor de memória, resumo durante a instalação, etapas derivadas do status, percentual e cancelamento. O estado de cancelamento é limpo após a tarefa encerrar, permitindo nova tentativa. O progresso usa os eventos reais do backend; os testes de interface fornecem eventos controlados.

As configurações receberam seletores com borda, fundo e seta, sem caracteres de seta dentro dos valores. Foi removida a regra global que apagava os estilos dos seletores. Os interruptores usam dimensões e estados coerentes. O seletor compartilhado recebeu seta, marcação do item escolhido e menu opaco. A ação que apenas mostrava uma mensagem de exclusão agora chama a API tipada de exclusão, com confirmação e bloqueio de repetição/jogo ativo.

O fechamento normal não registra um interceptor JavaScript. A interceptação existe apenas quando há jogo em execução e o aviso está habilitado. Cancelar mantém a janela; aprovar solicita sua destruição. A capacidade `core:window:allow-destroy` foi habilitada apenas para a janela `main`: a capacidade padrão anterior não autorizava essa operação. Isso remove tanto a dependência da interface em fechamento normal quanto a recusa da operação após confirmação.

## Validação

- `pnpm check`: zero erros e avisos.
- `cargo check`: passou.
- Testes unitários frontend: 76 passaram.
- Testes Rust de imagens, invalidação de cache e cancelamento: quatro passaram.
- Suíte de navegador: catálogo, instâncias em 1440/960 pixels, NeoForge, laboratório, links, skins e capas passaram.
- Instâncias: uma única barra com oito abas e apenas um painel montado; seleção por clique e teclado passou em 960/1440 pixels. 1.000 mods continuam acessíveis, filtros e ações em lote funcionam, configurações têm recuperação de erro e ferramentas abrem.
- Instalador: nome e 6 GB de RAM, progresso de 6%, cancelamento e retorno ao estado editável passaram.
- Fechamento: nenhum interceptor em repouso; rejeitar e aprovar o aviso com jogo ativo passaram com API controlada.
- Personalizador: as 12 capas, recuperação de erro, PNG original e animação durante a rolagem passaram. A amostra de produção teve 60.917 pixels opacos com cores correspondentes ao PNG original.
- Produção: catálogo/instalador/configurações e instâncias passaram nas duas larguras.
- Executável nativo: 1.000 mods carregados em 593,8 ms na primeira leitura e 91,7 ms com cache; ícones limitados, JARs originais preservados, alternância de mods e shaders passaram com dados isolados.
- Fechamento nativo: janela real do executável release encerrou em 19,9 ms, com código zero e dados isolados. O teste solicita fechamento gracioso do PID próprio usando a API Lua do Hyprland 0.56; não encerra outras janelas.
- WebKit sem compilação concorrente: catálogo com P95 de 17 ms, até seis cards montados e último pack acessível; lista de 1.000 mods com P95 de 18 ms na rolagem regular e 33 ms em saltos de 1.500 pixels por quadro. Nenhum transbordamento horizontal. São medições locais com conteúdo controlado, não garantia para todo hardware ou pack remoto.
- `pnpm tauri build --no-bundle`: release otimizado passou em 8 min 14 s. Executável: `src-tauri/target/release/luxmc`. O binário instalado em `/usr/bin` não foi substituído.

Capturas: [revisão visual](../visual/2026-10-03-revisao/). Os testes de interface usam respostas controladas do backend e não equivalem a instalar um pack remoto completo ou autenticar uma conta Microsoft real. O aviso de fechamento com Minecraft realmente em execução permanece sem validação nativa; aceitar/cancelar foram verificados com API controlada.
