# Luxmc 3.6 — revisão 3

Revisão de 10 de outubro de 2026. Este relatório diferencia verificações de código, simulações da interface e integrações reais. A aprovação de testes não representa garantia de que qualquer modpack, serviço externo ou computador será compatível.

## Correções

As salas usam o transporte QUIC do iroh, com conexão direta ou relay criptografado. O anúncio LAN e os proxies locais usam as interfaces IPv4 do computador, incluindo a interface física que o listener do Java utiliza. Os proxies aceitam apenas conexões originadas no próprio PC. A conexão é mantida ao navegar entre abas. A interface apresenta participantes, avatares, mundos, convite, renovação de convite e controle de entradas. O fluxo normal não baixa nem inicia o EasyTier.

Os painéis de configurações e conteúdo de instâncias ficam transparentes sobre wallpapers; os cartões internos preservam suas bordas arredondadas.

Cada instalação da fila recebe um identificador. Cancelamento, download, integridade, extração e conferência final compartilham a mesma operação. Um cancelamento antigo não cancela a próxima instalação. A conferência acontece antes de registrar a instância. Resultados tardios de uma instalação nova são removidos; instâncias já existentes não são apagadas. A auditoria de mods usa até quatro tarefas paralelas e verificações cooperativas de cancelamento.

O painel da comunidade exige a identidade autenticada do dono no servidor. Mostra perfis cadastrados, presença recente e pessoas jogando, com busca, paginação e tags públicas. As tags não concedem privilégios. As contagens online consideram presença recente; clientes sem conexão ou compartilhamento de presença não podem ser contabilizados como online.

O Shield passa a conferir o conteúdo em vez de confiar apenas no tamanho e horário do arquivo. A análise usa leitura por blocos, busca de padrões otimizada, até quatro tarefas paralelas, limites de descompressão e cache de conteúdo verificado. Corrigidos os dois padrões de webhook, incluindo classes maiores. Erros de leitura e JARs inválidos não são classificados como limpos. Indicadores críticos e verificações incompletas bloqueiam a abertura do jogo; sinais isolados geram avisos. Nenhum mod é apagado automaticamente. A oficina permite iniciar uma análise manual.

As assinaturas documentadas vêm da [investigação técnica do Fractureiser](https://github.com/trigram-mrp/fractureiser/blob/main/docs/tech.md). A análise local confere hashes e indicadores estáticos conhecidos; complementa o antivírus e não promete detectar toda ameaça nova ou ofuscada. Os testes usam apenas dados inertes em arquivos temporários, sem executar malware ou desativar o antivírus. A leitura dos JARs mantém até 16 MiB por tarefa em memória e evita abrir os cabeçalhos dos recursos que não são classes. Entradas duplicadas são inspecionadas individualmente para não ocultar código.

A descoberta de Java verifica o executável antes de indicá-lo como instalado e inclui Java 16 e 25. O reparo e gerenciamento de Java também ficam disponíveis na API do daemon. A remoção de um runtime é bloqueada durante uma partida. A proteção das pastas do sistema no Windows aceita variações de maiúsculas e minúsculas.

## Evidências

| Área | Verificação |
| --- | --- |
| Frontend | `pnpm check`: zero erros e avisos; 150 testes unitários aprovados após atualização das dependências |
| Backend | `cargo check` aprovado; 201 testes nativos aprovados, seis testes optativos ignorados na suíte padrão; integrações executadas separadamente |
| Site e contas | 48 testes aprovados; API de produção aceita a conta do dono e recusa requisição anônima; sem alteração de tags de pessoas reais |
| Metadados de atualização | 12 testes de versão, manifesto e assinatura aprovados |
| Telas | 16 suítes gerais e uma verificação adicional da seleção de Java 16/25: catálogo, mil mods, criação, laboratório, links, skins, Windows, sala/armazenamento, idiomas, wallpaper, exclusão/modal, Bedrock, contraste, animações, cancelamento/dono e oficina |
| Shield | Oito testes aprovados: marcadores inertes, webhooks, bloqueio crítico, JAR inválido, classe excessiva, alteração com mesmo tamanho/horário, classes duplicadas e leitura de recursos grandes |
| Pacotes instalados | Análise sem indicadores críticos em bibliotecas de 46, 73, 120 e 491 mods; arquivos preservados |
| Executável final Windows | Versão 3.6.0 conferida; Java 25 existente validado pela API real; remoção de versão inválida recusada; análise dos 491 mods em 2,407 s e segunda análise em 0,677 s, sem indicadores conhecidos |
| Pacote real | Arquivo do Fabulously Optimized obtido pelo provedor passou na validação de tamanho e hash nativa |
| Java 8 | Runtimes Mojang e Temurin baixados, conferidos e executados em pasta temporária |
| LAN | Listener padrão do Java 8 encontrou o anúncio multicast e transferiu dados pelo endereço anunciado |
| Relay e convite | Transferência com transporte direto desativado; convite curto publicado e resolvido no serviço de produção |

As simulações de telas utilizam respostas IPC controladas; não comprovam execução real do Bedrock, login Microsoft ou uma partida multiplayer. A transferência de rede utiliza um serviço de teste, sem modificar mundos do usuário. O teste final de Minecraft entre dois computadores em redes diferentes permanece necessário.

## Dependências e análise estática

DOMPurify, devalue, source-map-js e postcss-selector-parser foram atualizados para versões com correções publicadas. A auditoria posterior registra um aviso alto em `braces`, trazido pela cadeia de compilação do Tailwind, sem versão corrigida publicada no momento: [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm). Não foi ocultado da auditoria. Essa ferramenta trabalha com os padrões do projeto durante a compilação.

Clippy concluiu sem erros e apontou 116 avisos, principalmente sugestões de estilo e simplificação em código existente. `cargo check` apresenta cinco avisos de código não utilizado. Este relatório não declara ausência de avisos ou vulnerabilidades.

## Entrega

Pacote do launcher: 3.6.0, revisão 3. O instalador conserva o nome **Lux MC Launcher.exe**. A atualização mantém contas, mundos, instâncias e preferências. A pasta Documentos que contém o projeto não participa da exclusão de instâncias.

O instalador final foi compilado e conferido. Entrega local: [Lux MC Launcher.exe](../../../../release-windows/Lux%20MC%20Launcher.exe), 21.238.582 bytes. O arquivo de entrega tem o mesmo SHA-256 do pacote NSIS gerado pelo Tauri.

SHA-256: `62ce2c5cb0806e7f41db4b9885e855f06eb8cd380c2117775d79500e0b3cae85`.

`release-windows/SHA256SUMS` e `local-release.json` foram assinados com a chave Ed25519 oficial e verificados com `packaging/security/release-public-key.pem`. Essas assinaturas protegem os manifestos de integridade; não representam um certificado Authenticode do Windows.

A API do painel do dono está implantada no servidor. Esta revisão do instalador não foi publicada novamente no GitHub ou como download do site; a versão pública anterior permanece disponível. O teste de partida entre dois computadores em redes diferentes precisa usar este pacote local nos dois PCs.

Os tempos registrados são uma medição neste computador, não uma promessa para outros discos, computadores ou antivírus. Os testes dos scripts de instalação Linux exigem um ambiente Linux e não foram concluídos neste Windows.
