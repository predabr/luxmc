# Navegação, capas e modpacks — 03/10/2026

## Alterações

- Removidos os atalhos adicionais de hospedagem e diagnóstico da barra lateral. Hospedagem continua acessível em Amigos; diagnóstico continua na busca de comandos.
- Removido o banner com imagem da página inicial do launcher. Personalizador preservado.
- Catálogo reutiliza consultas por até dois minutos, reúne requisições simultâneas iguais e mantém até 48 resultados em cache. Falhas não entram no cache.
- Trocas de filtros e categorias não aguardam o debounce de digitação. A capa original aparece enquanto o banner maior carrega.
- Site recebeu profundidade em camadas, reflexos e iluminação acompanhando o ponteiro. Movimento respeita preferência reduzida e pausa quando a página fica oculta.

## Erro de inicialização reproduzido

Foi usada uma cópia dos 43 arquivos de mods do Fabulously Optimized, com Minecraft 26.3, Fabric 0.19.5 e Java 25. Nenhum mundo foi copiado ou aberto. O teste chamou a inicialização do Fabric e terminou antes de executar o método principal do Minecraft, com credenciais fictícias.

Antes da correção, o teste reproduziu os erros de conversão de classes de ModernFix e Sodium. A virtualização de AppData no Windows apresentava as mesmas bibliotecas por dois caminhos: o caminho nominal de AppData e o caminho físico dentro de Packages. O Fabric carregava classes equivalentes em carregadores diferentes.

O launcher agora resolve os caminhos físicos das bibliotecas, remove entradas duplicadas e converte o prefixo estendido do Windows em caminhos utilizáveis pelo Java. A normalização ocorre antes de montar os argumentos de inicialização, em todas as rotas de lançamento. Mods, configurações e mundos do usuário não foram alterados.

Com apenas o classpath normalizado, o mesmo teste carregou os entrypoints e terminou com `LUXMC_PREFLIGHT_OK`. Isso verifica a falha reproduzida; não equivale a testar renderização, jogo completo ou todos os modpacks disponíveis.

## Evidências

- `navigation-covers-check.log`: Svelte/TypeScript sem erros ou avisos.
- `navigation-cache-tests.log`: três testes de cache, concorrência, falhas e expiração passaram.
- `navigation-covers-browser.log`, `navigation-windows-browser.log`, `navigation-hosting-browser.log`: catálogo, navegação, hospedagem e armazenamento passaram nas verificações de navegador.
- `navigation-site-browser.log`: efeitos, redução de movimento e layout responsivo verificados.
- `navigation-build.log`: frontend compilado.
- `fabric-preflight-before.log`: reprodução da falha original.
- `fabric-preflight-classpath-only.log`: inicialização do Fabric passou com apenas a correção dos caminhos das bibliotecas.
- `modpack-classpath-check.log`: Rust compilou com três avisos existentes.
- `modpack-classpath-tests.log`: três testes passaram, cobrindo aliases, caminhos Windows e preservação de entradas ausentes.
- `navigation-modpack-windows-build.log`: executável Windows compilado com o frontend incorporado.
- `navigation-native-verify.log`: aplicativo aberto, comandos nativos funcionando e armazenamento contabilizando 905.832.768 bytes, sem avisos de leitura.
- `navigation-native-ui.log`: tela inicial sem banner e sem os dois atalhos adicionais na barra lateral.

Executável aberto para teste: `release-windows/Luxmc-2.0.2-corrigido.exe`. A janela respondeu com título Luxmc. Captura: `docs/visual/2026-10-03-hosting/windows-corrigido.png`.

## Ideia de servidores pagos

Somente proposta para discussão; nenhum serviço pago, cobrança ou provisionamento foi implementado nesta alteração.
