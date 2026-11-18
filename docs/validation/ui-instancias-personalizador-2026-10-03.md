# Interface e personalizador — 3 de outubro de 2026

Esta primeira implementação foi revisada para reunir toda a instância em uma página e manter o personagem animando durante a rolagem. Consulte a [revisão completa](revisao-instancia-completa-2026-10-03.md) para o estado atual.

O gerenciamento da instância concentra os ajustes em uma página: geral, instalação, otimização, janela, Java e memória, scripts e laboratório. Conteúdo, mundos, capturas e arquivos continuam na navegação da instância. O laboratório mantém diagnóstico reversível, cápsulas e benchmarks. RAM mínima e máxima, presets JVM e seleção de Java estão no mesmo painel. O salvamento usa uma cópia do perfil e dos valores enviados, bloqueia operações repetidas, atualiza os ajustes locais e permite tentar novamente após erro.

A referência foi o [componente de botões do Modrinth](https://github.com/modrinth/code/blob/main/packages/ui/src/components/base/buttons/ButtonFrame.vue) e seu [renderizador de aparência](https://github.com/modrinth/code/blob/main/packages/ui/src/components/skin/SkinPreviewRendererImpl.vue). O Luxmc usa seus componentes Svelte e seus tokens de tema. Foram padronizados 334 botões de ação. Botões com texto recebem largura adequada; controles de seleção, switches e cards mantêm seus formatos e estados. A regra global que substituía qualquer cor de botão no hover foi removida. O componente compartilhado oferece estados de carregamento, teclado, desativação e tipo de botão explícito.

A prévia usa texturas sRGB e materiais que preservam as cores do PNG. As capas receberam o mesmo tratamento e as capas geradas têm todas as faces UV preenchidas. Falhas removem a capa anterior da prévia e exibem um estado de recuperação. Os PNGs de skin importados permanecem intactos. A prévia tem limite de resolução de 2x, animação a até 60 FPS, pausa durante rolagem e zoom com Ctrl + roda; a roda normal rola a página.

A lista de mods usa linhas fixas de 76 pixels, dez linhas na amostra WebKit e sem medição individual de altura. A atividade de rolagem é observada com eventos passivos e pausa wallpapers, prévias, pulsos decorativos e desfoque da barra lateral. Os efeitos retornam após a rolagem. As capturas estão em [docs/visual/2026-10-03](../visual/2026-10-03/).

Os testes de interface usam respostas controladas do backend. Foram verificados 1.000 mods, filtros, congelamento de versões, ações em lote, shaders, argumentos enviados ao catálogo, configurações salvas, falha de salvamento e nova tentativa em 1440 e 960 pixels. Também foram verificados criação NeoForge, laboratório, links, importação de skin, correspondência de cores dos pixels, as 12 capas incluídas, recuperação de capa com falha e rolagem sobre a prévia. O teste de wallpaper verificou três loops e quadro preservado durante seek.

A versão instalada em `/usr/bin/luxmc` é de 28 de setembro de 2026. A revisão compilada deve ser aberta pelo executável deste checkout. Testes locais com respostas controladas não validam contas Microsoft reais, serviços remotos, Minecraft em execução ou Windows.

As ferramentas incluem acesso aos editores de controles e configuração, jukebox, exportação e backups. Os editores carregaram os dados da instância correta e a jukebox abriu e fechou nas duas larguras testadas. A versão de produção passou pelos mesmos fluxos de conteúdo, ajustes e ferramentas.

| Verificação | Resultado |
| --- | --- |
| `pnpm check` final | Zero erros e avisos |
| `pnpm test:run` | 76 testes passaram |
| Suíte de navegador | Catálogo, conteúdo, NeoForge, laboratório, links, skin e capas passaram |
| Configurações e ferramentas | Passaram em desenvolvimento e produção, em 1440 e 960 pixels |
| PNG e renderização | 60.647 pixels opacos amostrados correspondem às cores do PNG original; aplicação preserva o arquivo |
| Capas incluídas | 12 renderizadas; recuperação de erro passou |
| Wallpaper | Três loops, quadro preservado durante seek, biblioteca preservada |
| `git diff --check` | Passou |
| `pnpm tauri build --no-bundle` | Compilação release otimizada concluída; executável em `src-tauri/target/release/luxmc` |

No Chromium, a última amostra do catálogo teve p95 de 17,6 ms. No WebKit de produção, a medição final sem compilação concorrente teve p95 de 28 ms na rolagem contínua e 52 ms no teste extremo de saltos de 1.500 pixels por quadro. Durante a compilação, as amostras variaram entre 28 e 37 ms e entre 57 e 64 ms, respectivamente. Esses resultados não sustentam uma garantia de 60 FPS constantes. O teste WebKit usa um contexto efêmero para isolar cache e dados. A lista manteve dez linhas renderizadas, o último dos 1.000 mods acessível e ausência de overflow horizontal.

Para abrir esta revisão, feche qualquer versão anterior e execute `./src-tauri/target/release/luxmc` na raiz do checkout. Esta compilação não substituiu `/usr/bin/luxmc` nem gerou novos instaladores.
