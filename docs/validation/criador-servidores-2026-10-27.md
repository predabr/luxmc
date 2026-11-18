# Criador independente de servidores — 27/10/2026

## Alterações

A criação deixou de depender de perfis instalados. O assistente oferece Vanilla, Fabric, Quilt, Forge, NeoForge e catálogo de modpacks, com etapas para software, mundo, recursos e revisão. O servidor recebe diretório próprio; criar não abre o Minecraft nem aceita a EULA pelo usuário. Java e instalador do loader são concluídos no início do servidor.

Modpacks Modrinth usam o manifesto do pacote para fixar Minecraft e loader. CurseForge requer integração disponível, download autorizado e pacote de servidor associado pelo autor; o manifesto original identifica a versão exata do loader. Downloads têm verificação de integridade e importação validada. Pacotes declarados exclusivos do cliente são recusados. Não há garantia de suporte a todos os packs ou execução de scripts arbitrários. O catálogo exibe um conjunto limitado de versões por consulta.

Falha de preparação mantém o servidor reservado, um bloqueio de início incompleto e a referência do catálogo para retomar na visão geral. O botão de tentar novamente no assistente reutiliza o mesmo servidor. Pendências existentes que precisam de revisão não são apagadas silenciosamente. Uma preparação concluída é reconhecida por recibo para evitar instalação duplicada.

O formulário permite modo de jogo, dificuldade, seed, hardcore, estruturas, whitelist, PvP, memória, jogadores e distâncias de visão/simulação. Seed ausente em dados antigos preserva a propriedade anterior. Seed e geração não recriam mundos existentes. Os perfis Econômico e Equilibrado alteram distâncias de forma explícita; não instalam mods de desempenho nem prometem desempenho universal.

## Planos

Novos pedidos: Plus R$ 20,00 e Pro R$ 30,00 por 30 dias. Pedidos anteriores preservam o preço registrado. Plus admite cinco participantes na LAN e Pro dez, incluindo o anfitrião. Dedicado exige Pro e mantém até vinte jogadores e heap configurável de até 8192 MiB. Convidados não precisam de Pro para jogar no dedicado. O comportamento anterior do Free não foi alterado nesta revisão.

Limites LAN são aplicados no registro nativo da sala, com admissão serializada. Falha ao consultar o plano não concede uma capacidade de fallback ao usuário autenticado. Permissões do dedicado continuam verificadas no backend. O número de participantes da rede não é apresentado como contagem de jogadores Minecraft.

Site publicado em `https://366bf00d.luxmc-r92.pages.dev`, com produção em `https://luxmc-r92.pages.dev/planos`. HTTP 200, os dois preços e os limites LAN foram conferidos na página publicada. Nenhum pedido real, cobrança ou alteração de assinatura foi feito na validação.

## Interface e exclusão

O assistente tem progresso por etapas, seleção visual de software, catálogo, revisão e indicador da operação atual. Botões, foco e estados receberam tratamento compartilhado no launcher. O painel de servidores destaca entrada local normal do dono, separada da conexão P2P. Animações respeitam movimento reduzido e as preferências de efeitos existentes.

Excluir exige servidor parado, ausência de importação/download ativo e confirmação pelo nome. Remove definitivamente a pasta validada do servidor, com opção explícita de apagar seus backups independentes. O launcher não cria uma nova cópia em recuperação. A seção de restaurar servidores foi retirada da interface. Dados que já estavam arquivados em revisões anteriores foram preservados; esta atualização não os apaga automaticamente. Backups de mundo continuam sendo um recurso separado.

Foi corrigido o envio de proxies reativos no IPC de gerenciamento: os pedidos são convertidos em snapshots JSON antes de atravessar a comunicação. Isso evita falha de clonagem ao salvar configurações e depois criar outro servidor.

## Evidências e limites

A validação final passou com 341 testes nativos (14 auxiliares ignorados), 232 de frontend e 143 do site. O teste de migração preserva seed, hardcore e estruturas de configurações antigas e verifica que escolhas novas explícitas prevalecem. pnpm check terminou sem erros ou avisos; cargo check passou com os seis avisos preexistentes. O navegador passou em 1440×900, 1366×768 e 960×600 para criação Vanilla e por catálogo, confirmação EULA, RAM até 8 GiB, regras do mundo, ausência de cópia de perfil, console, jogadores, configurações, backups, navegação e exclusão definitiva. A interface de produção também passou sem perfis instalados e com falha de download seguida de nova tentativa sem duplicação, além de uma rodada em tema claro. As configurações de instância e armazenamento passaram em verificações adicionais de navegação, persistência de rascunho e operação assíncrona.

A revisão `40255548682.905` foi compilada, conferida e instalada em 27/10/2026 às 23:00, após confirmação do mantenedor de que encerrou o launcher e os servidores. Os hashes de 533 fontes de runtime corresponderam à liberação. O instalador terminou com código zero e a conferência integral do payload NSIS passou. O backup `backup-20261027-230001` preservou 17 tabelas SQLite e 60 arquivos protegidos, com duas contas e quatro perfis. A janela Luxmc foi reaberta e respondeu à consulta do sistema. Logs e recibos privados usam prefixo `creator-` em `%LOCALAPPDATA%/Luxmc-build/requests/local-server-20261026/`. Fontes de runtime congelados em `creator-validation.json`.

Testes de interface usam fixtures e não comprovam partida real. Nenhum Java/Minecraft do usuário foi iniciado e nenhuma nova EULA foi aceita. O dono poderá usar o endereço loopback apresentado no Multijogador, sem sala LAN; amigos em outros PCs precisam do caminho de rede correspondente. Entrada pública sem Luxmc continua dependente de gateway TCP/domínio, que não foram provisionados. Não há promessa de disponibilidade 24 horas, consumo mínimo universal ou ausência absoluta de falhas. Nenhum commit ou push foi feito nesta rodada.

A checagem de um pacote real Modrinth Adrenaline confirmou Minecraft 26.3, Fabric 0.19.5, manifesto e SHA-1 do arquivo. O catálogo do Fabulously Optimized declarou suporte de servidor como unsupported, coerente com a recusa do novo fluxo. Isso valida metadados e formato; não é teste de partida ou de instalação completa de todos os arquivos do pack.


## Conferência instalada

O serviço separado da revisão 40255548682905 respondeu à listagem e ao fluxo externo retirado. A listagem ativa estava vazia; nenhum servidor foi recriado. A consulta nativa ao catálogo real Modrinth retornou 100 versões de Adrenaline, incluindo Minecraft 26.3. Uma tentativa sem conta Pro foi recusada antes de criar ou instalar servidor. Nenhum Java/Minecraft foi iniciado pelo agente.

Instalador local: `%USERPROFILE%/Downloads/Luxmc 4.0 - criador de servidores.exe`. SHA-256 do instalador: `b48df9480dd3094d70f16f6ebd64e53d7bba50e67697a9d087adc710039cdfe2`. Recibos: creator-ready.json, creator-installation.json, creator-worker-smoke.json e creator-native-catalog-smoke.json. As prévias de desenvolvimento e produção usadas na validação foram encerradas.
