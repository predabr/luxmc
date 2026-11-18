# LUXMC-P2P-REFORMA-COMPLETA-01

Plano de continuação preparado em 26/10/2026 e ampliado a pedido do usuário, mantendo o mesmo código. **Execução local iniciada após o usuário enviar `1. LUXMC-P2P-REFORMA-COMPLETA-01`. As regras de publicação, infraestrutura, EULA e preservação abaixo continuam válidas. As verificações e pendências reais estão no registro de execução.**

## Registro da execução local

O painel, permissões, limite de RAM, whitelist, jogadores, configurações, conteúdos, preparação de loaders e backups foram implementados localmente para validação. Consulte [o registro de execução e os limites reais](../validation/p2p-reform-2026-10-26.md). As etapas abaixo continuam sendo os critérios completos de aceite; a partida real entre PCs e o domínio público não são considerados resolvidos por testes de fixture.

## Como retomar

Enviar no chat: `LUXMC-P2P-REFORMA-COMPLETA-01`.

Ao receber esse código como pedido explícito de continuação, ler este documento e o início de `CONTINUAR-LUXMC.md`, conferir o estado real do repositório e executar as etapas abaixo. Mencionar o código para editar o plano não autoriza começar a implementação. Preservar todas as alterações locais anteriores. O código retoma o trabalho local; publicação, contratação de infraestrutura, compra de domínio e aceitação de EULA não ficam autorizadas por este plano.

## Resultado desejado

Hospedar e entrar em um mundo pelo Luxmc deve ser compreensível sem conhecer portas, Java ou comandos de console. O usuário escolhe a instância ou o servidor, inicia e compartilha o endereço. Para servidor aberto, o participante informa esse endereço no Multijogador do Minecraft, sem convite obrigatório. A mesma área reúne estado, acesso, participantes, configurações e console. O mundo permanece salvo no PC, inclusive entre reinícios do launcher.

**Ampliação posterior do mesmo código:** entrada por endereço, sem convite obrigatório; painel com jogadores online, histórico de jogadores conhecidos e banimentos; LAN do Plus e P2P como critérios de entrega que exigem teste real. A publicação documental no site/GitHub foi autorizada separadamente; isso não autoriza começar esta implementação nem contratar domínio ou infraestrutura.

O trabalho abrange os erros encontrados na instalação local, a reorganização da hospedagem, modpacks instalados, gerenciamento de conteúdos do servidor, identidade visual do servidor, endereços fáceis de usar, salvamento automático e os ícones dos mods instalados. O proprietário deve conseguir administrar seu servidor por ações visuais, sem depender do terminal. Não adicionar lista pública de servidores, jogos novos ou Android nesta revisão.

## 0. Regras de produto atualizadas — prevalecem sobre as regras antigas

Estas são as regras solicitadas para a implementação futura. A instalação atual ainda tem o comportamento anterior; não anunciar que as novas restrições já estão ativas.

| Recurso | Free | Plus | Pro |
| --- | --- | --- | --- |
| Hospedar mundo aberto por LAN/P2P | Preservar a regra atual, sem ampliação neste pedido | Sim, até 10 participantes | Sim, até 20 participantes |
| Hospedar/rotear servidor dedicado pelo Luxmc | Não | Não | Sim, até 20 participantes |
| Compartilhar por P2P servidor dedicado externo já aberto | Não | Não | Sim, até 20 participantes |
| Painel completo de servidor dedicado, conteúdos e backups | Sem acesso à gestão Pro | Sem acesso à gestão Pro | Sim, para o proprietário |
| Entrar como convidado autorizado em servidor Pro | Sim | Sim | Sim |

- O pedido restringe **hospedagem de servidor dedicado** ao Pro. LAN de mundo aberto continua um caminho distinto. Não vender o mesmo recurso com nomes diferentes para contornar a regra do Plus.
- Preservar as regras existentes do Free fora dessa exclusividade. O limite Plus de 10 mantém o valor anterior, pois o usuário não pediu outro número. Convidados não precisam comprar Pro para entrar; essa é a premissa de implementação, sem criar cobrança não solicitada.
- Os 20 participantes incluem o anfitrião da sala. Configuração de `max-players`, capacidade da sala e convites precisam concordar; não apresentar 20 convidados além do dono.
- Validar direito de hospedagem no backend nativo e no serviço responsável pela autorização, além da apresentação na UI. Cobrir criação/início/compartilhamento/importação/gestão pelo produto, inclusive servidor externo e caminhos alternativos Tauri/Electron/daemon. Não confiar no plano enviado pelo frontend, no nome de usuário ou em campos editáveis de configuração.
- A restrição é sobre o produto Luxmc. Não tentar impedir que alguém execute Minecraft/Java ou um servidor independente fora do launcher.
- A política atual que cai para capacidade 10 quando não confirma Pro é insuficiente para servidor dedicado: uma nova hospedagem exige comprovação válida de Pro ou apresenta erro recuperável de verificação. Sessões temporariamente sem rede não devem perder o mundo nem sofrer encerramento forçado; definir política limitada de validade/grace period e avisos antes de aplicá-la.
- Em expiração, downgrade ou troca de conta, impedir novas operações exclusivas sem apagar dados. Permitir parar e salvar, exportar os próprios dados e recuperar backups locais. Não bloquear o salvamento por ausência de assinatura. Documentar o tratamento de uma sessão já ativa.
- Atualizar os textos de benefícios no launcher, site, comparação de planos, idiomas PT/EN/ES e README junto da implementação validada: Plus hospeda LAN; Pro acrescenta servidor dedicado e sua administração, até 20 participantes. Não atribuir 8 GB à nuvem: memória e processamento são do PC de quem hospeda.

### Memória: teto de 8 GB para o servidor gerenciado

- O novo teto solicitado é **8 GiB = 8192 MiB de heap máximo do Java do servidor**, disponível no Pro. Não significa RAM entregue por um host externo nem consumo total garantido do processo.
- O código atual admite até 65536 MiB; a revisão deve alinhar slider, campo numérico, validação nativa, configuração persistida e argumentos JVM ao novo teto de 8192 MiB. Não mudar a RAM do cliente Minecraft como efeito colateral.
- Faixa planejada: 512–8192 MiB, com passos compreensíveis e ajuste fino opcional. Começar com valor conservador, como 2048 MiB, e sugerir valores conforme pacote e recursos disponíveis; não selecionar 8 GB automaticamente.
- Mostrar RAM física/disponível, RAM configurada do servidor e consumo adicional do cliente/OS. Não assumir que ter 8 GB físicos permite dedicar 8 GB ao servidor. Alertar para pressão de memória sem travar a interface ou prometer que aumentar RAM resolve TPS.
- Não forçar `-Xms` igual a `-Xmx`. Verificar argumentos customizados para não duplicar heap ou ultrapassar a política do servidor.
- Configuração antiga acima do teto precisa de backup e aviso de ajuste antes do próximo início; não truncar silenciosamente nem reiniciar um servidor em uso. O teto não autoriza interferir na memória de um processo externo que o usuário iniciou por conta própria.

Aceite: Plus consegue hospedar LAN e não consegue hospedar servidor dedicado por nenhuma API do Luxmc; um Pro validado consegue os dois modos; convidados autorizados entram sem paywall de hospedagem; nenhuma configuração gerenciada inicia com heap acima de 8192 MiB.

## Estado real e evidências

- Instalada e aberta para o usuário a revisão local 4.0.0, build `40255548682.900`, em `C:/Users/preda/AppData/Local/Luxmc/luxmc.exe`.
- O servidor Vanilla foi criado e iniciado pelo usuário. As capturas mostram Minecraft 26.3, processo pronto e conexão do cliente chegando ao servidor.
- Recusa explícita: `You are not white-listed on this server!`. Consulta somente de leitura confirmou `white-list=true` e `enforce-whitelist=false` no servidor local. Ainda investigar quem definiu essa opção; não atribuir sua origem a uma causa não demonstrada.
- Erro separado no painel: `Nenhum servidor Minecraft respondeu nesta porta local`, apesar do estado pronto. A causa ainda não está comprovada. Investigar status/ping, pausa quando vazio, protocolo 26.3, corrida de estado e compartilhamento.
- O console registra `Server empty for 60 seconds, pausing` e salvamento de chunks. Esses registros isolados não demonstram corrupção. O aviso de desconexão duplicada não deve substituir o diagnóstico principal da whitelist.
- A interface empilha cabeçalho, lista/console, servidor externo, avisos e um `RoomPanel` inteiro. Isso explica parte do excesso vertical observado, sem comprovar todos os problemas de layout.
- A criação gerenciada atual é Vanilla. Encaminhar a porta de um servidor externo já existente não equivale a criar um servidor de um modpack instalado.
- O defeito de imagem reportado nesta rodada está nos **mods instalados da instância**. Pacotes de recursos foram apontados como referência correta.

Capturas preservadas em `docs/validation/p2p-followup-2026-10-26/`: `01-painel.png`, `02-botao.png`, `03-whitelist.png`, `04-console.png`, `05-icones-mods.png`.

## 1. Entrada no servidor e diagnóstico confiável — prioridade máxima

1. Reproduzir a recusa em uma configuração de teste e investigar o caminho que deixou a whitelist ligada. Preservar a configuração e o mundo reais do usuário.
2. Definir uma política explícita: servidor aberto por endereço, com autenticação oficial, sem convite obrigatório; whitelist é opção de acesso restrito escolhida pelo dono. Em servidores existentes, importar e mostrar a política atual; nunca apagar silenciosamente whitelist, banimentos ou permissões. Corrigir a exclusão acidental do dono sem transformar todo servidor em sala privada.
3. Usar o UUID/nome oficial do perfil Minecraft validado. Apelido Luxmc e identidade declarada pelo convidado não comprovam autorização. A conta usada no jogo precisa corresponder à identidade permitida.
4. Quando o acesso restrito estiver ativo, adicionar o dono validado e permitir gerenciar a lista pela interface. No modo aberto, conhecer o endereço basta para iniciar a conexão, sujeito à autenticação Minecraft, banimentos e capacidade, sem aprovação individual por convite. Não conceder operador automaticamente aos participantes; administração do dono deve ser uma escolha visível.
5. Aplicar alterações pelo canal apropriado, com confirmação de sucesso. Se uma propriedade exigir reinício, explicar e agendar; não prometer aplicação imediata. Manter autenticação oficial do Minecraft.
6. Separar três verificações: processo vivo, servidor aceitando Minecraft e túnel acessível. Uma falha de leitura de status não deve virar diagnóstico definitivo de servidor inexistente; uma porta TCP aberta também não prova que existe Minecraft nela.
7. Reproduzir compartilhamento logo após iniciar, durante pausa por inatividade e após retomada. Conferir versão do pacote de status, limites de leitura, timeouts, IPv4/IPv6 e revalidação de geração/porta. Usar tentativas limitadas e canceláveis; não mascarar falhas definitivas.
8. Mensagens claras com ação: acesso não permitido, versão incompatível, mod ausente, servidor parado, porta ocupada ou túnel indisponível. Remover o prefixo técnico `invalid state` da apresentação comum, mantendo detalhes copiáveis para diagnóstico.

Aceite: dono entra no próprio servidor; convidado autorizado entra; convidado não autorizado recebe a razão correta; o painel não conserva erro antigo após sucesso; compartilhar não quebra ao pausar/retomar o servidor.

## 2. Uma área de hospedagem simples e organizada

Composição proposta para desktop:

```text
Hospedar com amigos                 [Criar servidor]
┌ Seus servidores ─┬ Nome, capa, estado e versão ─────────────────┐
│ Seleção          │ [Iniciar / Parar e salvar] [Jogar] [Endereço] │
│ Vanilla          │ Endereço, jogadores e último salvamento     │
│ Modpack          ├ Visão geral | Conteúdo | Jogadores | Mais ▾  ┤
│                  │ Conteúdo da aba selecionada                 │
└──────────────────┴─────────────────────────────────────────────┘
```

- Conservar a lista de servidores à esquerda, como o usuário gostou. Ao selecionar, mostrar o conteúdo daquele servidor; não confundir seleção com outro servidor ativo.
- Remover o empilhamento de duas interfaces completas de sala/servidor. Integrar endereço, participantes e estado P2P ao mesmo painel, reaproveitando a lógica existente. Convite privado fica como opção de acesso restrito, não como requisito do fluxo padrão.
- Limitar o console à área disponível, com rolagem própria, pausa de acompanhamento ao subir, copiar diagnóstico e ações de comando opcionais. O console não é a primeira exigência para conseguir jogar.
- Configurações extensas ficam em abas ou painel de edição. Em janelas menores, lista recolhível/seletor e conteúdo adaptado; não comprimir controles para forçar tudo em uma altura impossível.
- Dentro do mesmo painel, organizar Visão geral, Conteúdo, Jogadores, Console, Ajustes e Backups. Usar overflow/menu de navegação quando necessário. Em Conteúdo, oferecer Mods, Modpack, Datapacks, Texturas/Resource packs, Mundos e Arquivos; evitar uma fileira de abas que escape da janela.
- Estado visível: preparando, iniciando, pronto, compartilhando, salvando, parando e falhou. Evitar várias ações concorrentes, reenvio duplicado e respostas antigas substituindo o estado atual.
- Ações principais acessíveis sem percorrer blocos inferiores. Foco de teclado, contraste, textos PT/EN/ES e redução de movimento. Efeitos curtos, sem reanimar todo o console a cada atualização.

Aceite: em 1440×900 e 1366×768, iniciar/convidar/parar ficam visíveis na área principal; em 960×600, nenhuma ação some ou cobre conteúdo; não há rolagem horizontal; abas preservam foco e estado.

## 3. Hospedar a partir das instâncias e dos modpacks instalados

- Colocar uma ação evidente **Hospedar com amigos** na biblioteca/detalhe da instância e no menu correspondente. Levar nome, versão, loader e origem ao assistente, evitando pedir que o usuário redigite tudo.
- Oferecer dois caminhos explicados: **Compartilhar meu mundo aberto**, reaproveitando a integração LAN, e **Criar servidor separado desta instância**, com execução independente do cliente.
- O primeiro caminho precisa funcionar com mundos abertos nos modpacks instalados compatíveis com LAN. Detectar o mundo/porta e orientar quando ainda for necessário abrir para LAN; não confundir essa sessão com servidor dedicado.
- No segundo, resolver compatibilidade pela versão exata instalada. Confirmar os perfis citados pelo usuário: Vanilla, Fabulously Optimized, Transform Modpack e Vanilla Perfect; nomes semelhantes não provam identidade do projeto.
- Para CurseForge, priorizar o pacote oficial de servidor associado àquela versão. Para Modrinth, respeitar o ambiente de servidor do manifesto, dependências e arquivos específicos de servidor. Metadados ausentes ou errados precisam de diagnóstico; não tratar todos os mods como universais.
- Instalar o loader servidor correto, Java correto e dependências, priorizando Fabric, Forge, NeoForge e Quilt conforme suporte demonstrado. Não reutilizar o comando de cliente para iniciar um servidor.
- Não copiar cegamente todos os JARs do cliente. Não executar automaticamente scripts arbitrários de pacotes importados. Se não houver caminho automático confiável, manter o modo LAN e oferecer importação guiada de pacote de servidor, com explicação concreta.
- Criar diretório independente de servidor. Reutilizar downloads imutáveis/cache por hash quando seguro, sem vincular arquivos mutáveis que façam uma edição alterar a instância original.
- Criar mundo novo ou importar cópia consistente de um mundo escolhido. Para copiar mundo em uso, exigir salvamento e parada apropriados; manter o original intacto. Não trocar versão/loader de mundo existente silenciosamente.
- Atualizações de pacote servidor exigem correspondência de versão, backup, conferência de arquivos e possibilidade de retorno. Instância cliente não pode ser modificada como efeito colateral.

### 3.1. Gerenciar conteúdo diretamente no servidor

Fluxo comum: abrir **Conteúdo**, escolher categoria, buscar ou importar arquivo, ver compatibilidade e dependências, confirmar e acompanhar a instalação. O destino já é o servidor selecionado; não perguntar pela instância cliente por engano.

| Categoria | Operações pelo painel | Destino e comportamento |
| --- | --- | --- |
| Mods | Buscar em Modrinth/CurseForge, instalar, importar JAR, atualizar, desativar/ativar e remover | `mods/` do servidor com loader e versão compatíveis; mudanças de runtime exigem parada/reinício controlado |
| Modpack | Escolher pacote/versão, importar pacote servidor, atualizar e consultar modificações locais | Pacote servidor correspondente; substituir versão exige backup e plano de alterações, sem misturar dois modpacks arbitrariamente |
| Datapacks | Buscar/importar, listar, habilitar/desabilitar, atualizar e remover | Pasta de datapacks do mundo escolhido, com versão/formato compatível e aplicação/reload controlados |
| Texturas / Resource packs | Buscar/importar, pré-visualizar, definir pacote distribuído pelo servidor, atualizar e remover | Recurso recebido pelo cliente; exige publicação acessível e configuração apropriada do servidor, não instalação como JAR |
| Mundos | Criar, selecionar, importar cópia, exportar, fazer backup e restaurar | Mundo independente do servidor; importação/troca/restauração somente em estado seguro, sem sobrescrever o original |
| Arquivos e configurações | Abrir pasta, importar/exportar e editar arquivos de configuração suportados | Editor limitado à pasta do servidor; alterações relevantes identificam necessidade de reinício |

- **Mod novo com um fluxo:** selecionar um mod, resolver dependências obrigatórias, apresentar resumo, baixar versões corretas, conferir hash, instalar no destino e atualizar o índice. Dependências opcionais são apresentadas como escolha; não inferir obrigatoriedade pelo nome.
- Resolver lado cliente/servidor e loader pela versão real do arquivo e seus metadados. Mod exclusivo de cliente não deve entrar no dedicado por engano. Quando os metadados forem inconclusivos, explicar a incerteza e oferecer diagnóstico/importação guiada; não usar uma lista frágil de nomes como prova de compatibilidade.
- Mostrar nome, ícone, versão, autor/origem, dependências, compatibilidade, estado de instalação e se os jogadores também precisam instalar o conteúdo. Cache de imagens e metadados evita consultas completas em cada visita.
- Registrar mod/arquivo adicionado manualmente sobre o modpack. Na atualização, mostrar o que será preservado, substituído ou removido. Não apagar modificações locais por uma reparação automática que desconheça sua origem.
- Manter downloads canceláveis/retomáveis, concorrência limitada e fase de preparação separada da aplicação. Evitar arquivos parciais na pasta ativa, duplicatas do mesmo mod e alterações simultâneas no mesmo servidor. Validar hashes, caminhos de arquivos e extração de ZIP contra saída da pasta e excesso de tamanho.
- Remover/desativar conteúdo deve avaliar dependências e avisar sobre conteúdo de mundo que pode ser perdido. Fazer backup e permitir desfazer alterações da instalação; desfazer arquivos não recupera automaticamente um mundo já migrado sem seu backup correspondente.
- Quando mudar um mod exigido pelos clientes, atualizar o resumo de compatibilidade e avisar os participantes. Preparar uma lista comparável de versões/hashes e links oficiais para sincronização no launcher, respeitando permissões de distribuição; não redistribuir JARs de terceiros sem autorização.
- Para pacote Vanilla, instalar mod exige um caminho explícito para servidor com loader compatível, com backup/validação do mundo. Não trocar silenciosamente o loader ou converter o mundo em uso.

### 3.2. Conteúdo com o servidor em execução

O usuário pode navegar, pesquisar e preparar downloads enquanto joga. O painel distingue **baixado**, **aguardando aplicação** e **ativo no servidor**.

- Mods e mudança de loader: preparar em staging, mostrar alterações pendentes e oferecer **Salvar, reiniciar e aplicar**. Não tentar hot reload genérico de JARs.
- Datapacks: validar o mundo e a versão; usar o mecanismo de aplicação compatível com aquela versão/loader, com backup e confirmação. Quando exigir reload/reinício, oferecer ação clara e informar impacto aos jogadores.
- Resource packs: podem ser obtidos sem fechar o cliente, mas disponibilização aos jogadores depende do recurso de distribuição/configuração suportado. Não exigir encerrar o Minecraft só para pesquisar ou baixar; não prometer que todos os clientes recarregam automaticamente.
- Mundos: não substituir um mundo carregado. Oferecer preparação e aplicação após parada limpa.
- Edições de arquivos: detectar modificação externa e conflito de versão; pedir escolha entre versões antes de sobrescrever. Serializar alterações para não competir com autosave ou com o próprio servidor gravando configurações.
- Se a aplicação falhar, manter o estado anterior utilizável ou explicar recuperação; nunca marcar como instalado só porque o download terminou.

### 3.3. Texturas distribuídas pelo servidor

- Texturas e resource packs compartilham a mesma categoria funcional para evitar duplicação confusa.
- Um servidor dedicado não renderiza texturas. O painel deve configurar a distribuição do pacote aos clientes, com URL HTTPS acessível, hash e opções de aceite compatíveis com a versão do Minecraft.
- Definir pacote obrigatório deve ser uma escolha explícita do dono, com explicação do efeito para quem recusar. Não escrever na pasta pessoal de texturas dos convidados sem o fluxo autorizado no launcher/jogo.
- Armazenar um ZIP no PC do dono não produz uma URL pública acessível. Planejar distribuição pela infraestrutura existente somente se suportada e validada, ou um transporte pelo launcher para participantes autenticados. Se exigir novo serviço, registrar a dependência e não anunciar distribuição automática antes de implementá-la.
- Preservar licença/origem dos arquivos; limitar tamanho e consumo de disco. Quando a versão do Minecraft limitar a quantidade de pacotes distribuídos, respeitar essa capacidade; não combinar ZIPs arbitrariamente e prometer que equivalem a uma pilha de pacotes.

Aceite adicional: instalar um mod compatível resolve suas dependências; um conteúdo incompatível apresenta motivo útil; preparar downloads com o jogo aberto funciona; mudanças que precisam de reinício entram na fila; datapack afeta o mundo correto; pacote de textura chega ao cliente por um fluxo efetivamente testado; atualizar o modpack não apaga os acréscimos locais sem decisão do dono.

Aceite: cada perfil testado informa versão/loader e caminho de hospedagem real; um pacote só de cliente não é rotulado como servidor compatível sem validação; instalação interrompida pode retomar sem duplicar mods; originais permanecem preservados.

## 4. Servidor com nome, imagem e configurações próprias

- Nome de exibição, descrição/MOTD, imagem do servidor, número de jogadores dentro do plano, dificuldade, modo de jogo, PvP, distância de visão/simulação e gerenciamento de acesso.
- Separar capa do cartão no launcher e ícone que o Minecraft mostra no Multijogador. Preparar o `server-icon.png` no formato exigido pela versão, com prévia e conversão validada; manter a imagem original. Verificar o comportamento em 26.3 e versões antigas suportadas antes de fixar regras de formato.
- Mostrar se a mudança é imediata ou requer reinício. Evitar alterações conflitantes enquanto o processo inicia ou salva. Configurações avançadas ficam acessíveis sem ocupar toda a visão principal.
- Seed e opções de geração devem explicar que não regeneram um mundo existente. Importar/exportar configuração com cuidado para não substituir mundos, operadores ou banimentos.
- Aplicar a regra futura da seção 0: Plus hospeda LAN; servidor dedicado pelo Luxmc é exclusivo do Pro, com até 20 participantes incluindo anfitrião e heap gerenciado de até 8192 MiB. Esses limites são política planejada, não uma descrição da instalação atual.

### Administração pelo proprietário

- O dono pode iniciar, parar salvando, reiniciar de forma controlada, convidar/remover participantes, fechar novas entradas, gerenciar whitelist/banimentos/operadores e ver jogadores conectados.
- Disponibilizar ações visuais para dificuldade, modo de jogo, PvP, acesso, nome/imagem, memória, conteúdo, mundos, backups e propriedades suportadas. Manter console e editor avançado para quem quiser usá-los.
- Confirmar ações destrutivas com identificação do servidor/mundo. Distinguir remover conteúdo, restaurar backup e excluir servidor. Exportação e backup devem ser acessíveis antes de exclusão.
- Controle do servidor fica no processo do anfitrião. Ser convidado na sala não concede acesso a arquivos, comandos administrativos ou credenciais. Um eventual administrador delegado exige concessão explícita e escopo definido; não criar controle remoto irrestrito como atalho.
- Servidores externos: apresentar somente controles que o Luxmc realmente possui. O processo iniciado fora do gerenciador não pode mostrar um botão de parada/edição que finja controlar seu ciclo de vida; oferecer vinculação/importação gerenciada com consentimento e preservação de dados.

Aceite: nome/MOTD/ícone aparecem corretamente no launcher e na lista do Minecraft; reiniciar preserva todas as escolhas e o mesmo mundo.

## 5. Endereço fácil e integração com Adicionar servidor

Objetivo: o usuário dá um nome ao servidor, compartilha seu endereço e o participante entra em **Multijogador → Adicionar servidor / Conexão direta**, sem convite obrigatório, código de sala ou aprovação individual no servidor aberto. Convites podem existir para quem escolher acesso privado.

**Dois caminhos precisam ser distinguidos e validados:**

1. **Endereço público utilizável pelo Minecraft diretamente:** domínio/subdomínio sob controle do projeto apontando para uma entrada TCP Minecraft realmente alcançável, com transporte até o PC do dono. O backend precisa de relay/proxy público quando a rede doméstica não aceitar entrada direta. O uso de NAT traversal P2P pode reduzir tráfego intermediário quando o cliente tiver integração apropriada, mas Minecraft padrão não fala o protocolo QUIC/iroh do Luxmc por receber um nome DNS. Não anunciar funcionamento sem launcher no convidado antes de implementar a entrada pública.
2. **Endereço com integração Luxmc no participante:** nome estável identifica o servidor; o launcher resolve esse nome, estabelece o túnel e mapeia a conexão local automaticamente, sem exigir colar um convite. Validar como o Minecraft resolve esse nome e chega ao proxy, inclusive sem parâmetros manuais. Informar a necessidade do Luxmc aberto. Não tratar esse modo como equivalente a um servidor público acessível por qualquer cliente.

Para atender ao exemplo do usuário de só informar o endereço, o caminho 1 é o objetivo completo. Preparar especificação de infraestrutura e custos; não contratar, registrar domínio ou abrir portas automaticamente. Se essa dependência não estiver resolvida, declarar a parte pública pendente e não substituir o pedido por um convite obrigatório.

- Ação **Adicionar ao Minecraft** para uma instância escolhida, com nome e ícone quando suportado, preservando entradas existentes e fazendo backup de `servers.dat`.
- Evitar alterações concorrentes com o Minecraft gravando a lista. Detectar cliente aberto e usar integração validada ou orientar atualização/reabertura da tela; não afirmar que editar o arquivo em disco atualiza imediatamente a interface do jogo.
- Aceitar nome amigável/slug, oferecer sugestão aleatória e tratar colisão, caracteres inválidos e alteração de nome. Um nome público ou nickname não é segredo de acesso. Verificar banimento e capacidade em cada entrada; permitir modo restrito opcional e revogação dos seus convites.
- Manter endereço numérico copiável como alternativa e explicar claramente quando o Luxmc precisa estar aberto no PC do convidado.

**Domínio como o usuário pediu:** os exemplos `mush.com` e `luxmc.net.OWNER` são referências de experiência, não autorização para usar domínios de terceiros nem prova de domínio registrado. A forma proposta é `owner.<domínio-controlado-pelo-Luxmc>` ou `<nome-escolhido>.<domínio-controlado-pelo-Luxmc>`, após comprovar controle do domínio e configurar o transporte. Oferecer nome automático ou escolhido pelo dono, respeitando unicidade. Não apresentar `luxmc.gamer`, `luxmc.net` ou outro domínio como disponível sem verificação.

Antes dessa parte pública, documentar alternativas: domínio sob controle do projeto com resolução para o ponto de entrada adequado, ou alias resolvido pela integração do launcher/cliente. Validar resolução real no Java/Windows/Linux; não editar o arquivo hosts nem instalar DNS de sistema automaticamente. Custos, domínio e infraestrutura ficam como decisão explícita do usuário. Entregar o endereço local/convite funcional mesmo que a parte pública permaneça pendente, informando a pendência.

Aceite: dono e convidado recebem o endereço correto para seus próprios PCs; a entrada salva funciona após reconexão; um alias não é exibido como funcional antes de resolver e completar a conexão em teste real.

### 5.1. Jogadores e moderação no painel do dono

- **Online agora:** jogadores efetivamente conectados ao Minecraft, com contagem reconciliada com o servidor. Participantes no túnel ou launcher aberto não contam automaticamente como jogadores ativos.
- **Já jogaram:** total de perfis Minecraft únicos que entraram com sucesso, por UUID oficial validado; mostrar primeira/última entrada e duração quando disponíveis. Tentativa recusada deve constar no diagnóstico, não como jogador que já entrou.
- **Offline conhecidos:** jogadores do histórico que não estão conectados. Não alegar contar pessoas desconhecidas que nunca entraram. Separar totais únicos de número de sessões/visitas.
- Lista com nome/UUID, ícone/skin quando disponível, estado, busca, ordenação e paginação. Atualização sem recarregar página; eventos de entrada/saída e reconciliação limitada evitam polling pesado. Após perda de eventos/reinício, mostrar estado desconhecido até reconciliar, sem inventar números.
- Ações do dono: expulsar, banir, desbanir, gerenciar whitelist opcional e operadores. Banir identifica UUID, pode registrar motivo e remove a sessão quando aplicável. Confirmar resultado e preservar o ban após reinício.
- Não usar o IP loopback/proxy compartilhado como identidade de todos os jogadores: banir esse IP pode atingir a turma inteira. Aplicar moderação por identidade Minecraft no servidor; decidir política de IP público somente se a arquitetura obtiver origem confiável e justificar isso.
- Persistir histórico mínimo na pasta/banco do servidor, com retenção e exclusão de dados definida. Informar o registro ao dono e usuários pelos termos apropriados; não publicar históricos pessoais no site nem expor comandos/controle a quem só conhece o endereço.

### 5.2. LAN do Plus e P2P como requisitos de entrega

- A LAN foi reportada pelo usuário como ainda problemática; não marcar como corrigida só porque fixtures passam. Testar Vanilla e os modpacks citados em dois PCs, tanto descoberta LAN quanto endereço direto.
- Verificar descoberta da porta, anúncio multicast, firewall, autenticação, compatibilidade de versão/mods, rede direta e fallback relay. Garantir que Plus tenha o caminho LAN acessível sem acionar bloqueio de servidor dedicado Pro.
- Servidor dedicado Pro: testar entrada por endereço sem convite no modo aberto, jogador visto no painel, ban/desban, reconexão e preservação do mundo. Testar caminho direto e redes que exigem relay.
- Critério bloqueante para declarar pronto: registro de partida real bem-sucedida nos cenários combinados, além dos testes automatizados. Até o usuário participar dos testes, registrar validação real pendente e manter o site explícito sobre o estágio do recurso.
- O compromisso é corrigir e validar esses cenários. Não prometer funcionamento em qualquer NAT/firewall, disponibilidade absoluta de terceiros ou ausência total de erros sem evidência.

## 6. Salvamento automático e recuperação

- Preservar e verificar o autosave nativo do servidor. Exibir último salvamento confirmado; detectar logs por adaptadores de versão/loader, sem supor que todo texto `Saving` prova conclusão.
- Oferecer política configurável de salvamento periódico com `save-all flush` quando suportado e necessário, sem sobrepor operações ou disparar gravação pesada continuamente. Usar confirmação, timeout e adaptação a servidor pausado.
- **Parar e salvar** aguarda encerramento limpo. Não anunciar que salvou quando o processo foi encerrado à força. Não matar processos Java não pertencentes ao gerenciador.
- Fazer snapshots/backups consistentes com retenção e limite de espaço; backup não é o mesmo que autosave. Não copiar um mundo enquanto ainda está sendo gravado sem um protocolo de consistência válido.
- Tratar falta de disco, falha de gravação, crash e desligamento. Guardar configuração/índice de forma atômica e oferecer recuperação da última cópia válida, sem prometer recuperar dados que não chegaram ao disco.
- Sair da sala mantém o servidor/mundo conforme a ação explicada; sair do launcher salva e encerra os processos gerenciados. Nenhuma implementação deve sugerir que o servidor continua online com o PC desligado.

Aceite: salvar, parar, reabrir e reiniciar conserva mudanças no mundo; backups têm retenção previsível; falha de salvamento é visível e não vira mensagem de sucesso.

## 7. Corrigir as imagens dos mods instalados

- Inspecionar o bloco de ícones da lista de mods em `src/routes/instances/[id]/+page.svelte` e o carregamento associado. O usuário não pediu redesenhar os cartões de download nesta rodada.
- Um contêiner define tamanho, raio e recorte; a imagem respeita esse recorte. Ajustar padding/fundo para ícones transparentes, quadrados, retangulares e com borda já desenhada, sem moldura escapando da imagem.
- Preservar a arte inteira quando apropriado, sem esticar ou cortar letras. Aplicar fallback somente após falha real e permitir recuperação do carregamento.
- Manter cache/chave por arquivo/versão corretos, reservar dimensões e não reextrair imagem do mesmo JAR em cada renderização. Priorizar itens visíveis, cancelar trabalho obsoleto e limitar concorrência.
- Usar pacotes de recursos como referência visual, preservando seu funcionamento atual. Não alterar JARs para corrigir apresentação.

Aceite: imagens das capturas ficam alinhadas, sem vazamento de borda ou deformação, em temas claro/escuro e diferentes escalas; lista grande continua fluida e não mostra ícone de outro mod por erro de cache.

## 8. Ordem de execução e arquivos envolvidos

| Etapa | Entrega | Pontos principais |
| --- | --- | --- |
| A | Diagnóstico, acesso, plano e memória corretos | `src-tauri/src/core/local_server.rs`, `commands/local_servers.rs`, membership, protocolo de status e `network/p2p_tunnel.rs` |
| B | Painel único e CTA visível | `LocalServerPanel.svelte`, `RoomPanel.svelte`, `/hosting`, páginas de instâncias e wrappers tipados |
| C | Hospedagem e gestão de conteúdo | resolvers de pacotes, adapters de loaders, APIs de conteúdo servidor, staging, índices, imports e preservação de mundos |
| D | Identidade do servidor e endereços | configuração persistente, ícones, convites e integração com lista do Minecraft |
| E | Autosave, backups e recuperação | gerenciador de processo, armazenamento, estados e retenção |
| F | Fluidez, ícones e benefícios coerentes | lista de conteúdo da instância, pipeline de imagens, textos do launcher/site/README e idiomas |
| G | Validação e entrega local | testes pertinentes, build otimizada, backup e instalação para o usuário |

Manter compatibilidade Tauri/Electron/daemon e snapshots antigos do túnel. Novos campos persistidos devem ter defaults/migração segura. Toda operação nativa passa por comandos e wrappers tipados. Seguir `AGENTS.md`, runes Svelte 5, tokens de tema e ausência de comentários de código não solicitados.

### Fluidez e uso de recursos

- Medir navegação, tempo até os primeiros itens/ícones, atualização do console e uso de CPU/RAM/disco antes/depois, na mesma máquina e com os mesmos dados. Não inventar ganho percentual.
- Carregar a aba visível e reutilizar resultados válidos; cancelar consultas antigas, deduplicar downloads e limitar trabalho concorrente. Não varrer todos os JARs/mundos a cada mudança de aba.
- Atualizar status e logs por cursor/eventos ou polling controlado, reduzindo atividade com a janela oculta. Reservar espaço de imagens para evitar saltos; virtualizar listas grandes quando a medição justificar.
- Animações curtas de hover, seleção e troca de aba devem continuar visíveis e respeitar redução de movimento. Priorizar transform/opacity e evitar blur/sombras animadas ou fundos que tornem console e botões ilegíveis.
- Armazenamento discrimina cache, downloads temporários, arquivos do servidor, mundos e backups. Limpeza de cache não apaga conteúdo ativo nem backups necessários para uma atualização em andamento. Retenção limita crescimento sem remover o mundo atual.

## 9. Matriz de validação e critérios para entregar

1. Regressão de whitelist: dono, convidado permitido, conta diferente, lista preexistente e falha de verificação de identidade.
2. Rede: status lento, porta errada/ocupada, processo encerrado, pausa/retomada, compartilhamento antes de pronto, IPv4/IPv6, reconexão e saída durante ação pendente.
3. Jogo: Vanilla 26.3 e uma versão anterior; Fabric/Forge/NeoForge/Quilt conforme cada caminho implementado; cliente com versão ou mods divergentes recebe diagnóstico correto.
4. Modpacks citados: registrar IDs e versões reais e resultado de LAN e de servidor dedicado separadamente. Não usar só a presença do loader como prova de compatibilidade.
5. Dados: importação de mundo sem alterar origem, cancelamento de download, reinício, parada limpa, crash, falta de espaço e restauração de backup em pasta de teste.
6. Interface: diferentes janelas e escalas, teclado, PT/EN/ES, animações reduzidas, console grande, ícones variados e atualização sem travamentos perceptíveis.
7. Verificações obrigatórias: `pnpm check` após TS/Svelte e `cargo check` após Rust; testes direcionados de contratos e regressões, build frontend e NSIS otimizada. Repetir suítes amplas apenas quando as mudanças justificarem.
8. Preservar a preferência do usuário de testar o jogo pessoalmente: fixtures/processos de teste são permitidos; não abrir uma partida nem aceitar EULA por ele. A conexão real entre duas redes depende de teste combinado com o usuário e deve ser registrada como pendente até acontecer.
9. Criar backup atualizado antes de instalar, verificar processos sem encerramento forçado, validar payload NSIS e preservação de dados, instalar e abrir para o usuário. Só pedir que feche o launcher se estiver realmente em uso.
10. Entregar relatório do que passou, o que não passou e limites reais. Não prometer ausência absoluta de erros. Não publicar no GitHub/site como efeito automático do código de continuação.

### Casos adicionais obrigatórios da ampliação

- Plus: hospedar LAN permitido; criar/iniciar/compartilhar dedicado pelo produto negado, inclusive chamando APIs diretamente. Pro: dedicado gerenciado e encaminhamento de servidor externo permitidos. Free/Plus convidados: entrada autorizada funciona.
- Pro expirado, verificação indisponível, troca de conta, limite 20 e tentativa do 21º participante: mensagens corretas, nenhuma perda de mundo, nenhum bypass por metadados de cliente. Parar/salvar/exportar continuam acessíveis.
- Memória em 512, 2048 e 8192 MiB; valores negativos, não numéricos e acima do teto; configuração antiga de 16/64 GB; pressão de memória física; argumentos JVM tentando duplicar/superar o limite. Não fazer teste pesado de 8 GB sem necessidade nem iniciar o Minecraft do usuário.
- Conteúdo: dependência obrigatória/opcional, mod só de cliente, versão/loader incompatível, hash inválido, arquivo parcial, ZIP malformado, caminho fora da pasta, duplicata, cancelamento, atualização interrompida e conflito com edição externa.
- Aplicação com servidor ativo: staging funciona, JAR não é trocado em uso, fila persiste/é reconciliada após reinício, falha preserva a versão anterior, o dono vê o que exige reinício. Instância cliente continua intacta.
- Resource pack: URL/transporte realmente alcançável pelo convidado, aceite/recusa, hash, atualização e comportamento por versão. Datapack: mundo certo, compatibilidade, habilitação/desabilitação e necessidade de reload. Mundo: restauração sem sobrescrever uma sessão em execução.
- Administração: convidado não consegue editar arquivos nem executar comandos; dono consegue gerir acesso/conteúdo; servidor externo informa suas capacidades reais. Todos os novos benefícios têm textos equivalentes no site, launcher e idiomas.
- Entrada pública sem convite, modo privado opcional, DNS sem transporte (erro honesto), domínio não controlado, alias em conflito e resolução Java real. Métricas online/únicos/offline reconciliadas, histórico após reinício, ban/desban e tentativas recusadas sem inflar contagens.

## Referências consultadas para o planejamento

- [Modrinth: formato mrpack](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack): diferencia os ambientes cliente e servidor; informa hashes e arquivos próprios de cada lado. Orienta a resolução do pacote, sem garantir a qualidade dos metadados de cada autor.
- [CurseForge: API](https://docs.curseforge.com/rest-api/): expõe a associação de arquivos de servidor, incluindo `isServerPack` e `serverPackFileId`. A associação precisa corresponder à versão instalada.
- [Iroh: relays](https://docs.iroh.computer/concepts/relays): a conexão tenta caminho direto e pode usar relay criptografado; os relays públicos têm limites e não oferecem garantia de desempenho. DNS não substitui esse transporte.
- [RFC 6761, seção 6.3](https://www.rfc-editor.org/rfc/rfc6761.html#section-6.3): `.localhost` é reservado para loopback; não é um domínio público registrável. Qualquer alias local ainda precisa ser validado no resolvedor efetivamente usado pelo Minecraft.

## Prompt de retomada completo

> Continue LUXMC-P2P-REFORMA-COMPLETA-01. Leia integralmente o plano ampliado e o estado recente em CONTINUAR-LUXMC.md antes de editar. Trabalhe por etapas até entregar uma revisão local coerente e revisável. Primeiro corrija a recusa por whitelist e o erro de status/compartilhamento, com diagnósticos distintos. Aplique a regra de produto: Plus hospeda somente LAN; Pro também cria, administra e compartilha servidor dedicado, gerenciado ou externo, com até 20 participantes incluindo anfitrião. Convidados autorizados não precisam de Pro. Valide o direito de hospedagem nas APIs, não apenas na interface. Configure o teto do heap do servidor gerenciado em 8192 MiB, com orientação sobre memória disponível e sem alterar a RAM do cliente. Reorganize a hospedagem em uma área única com lista à esquerda e abas responsivas; destaque Hospedar com amigos nas instâncias. Implemente LAN e criação dedicada dos modpacks instalados conforme compatibilidade real de versão, loader, dependências e lado de execução. Crie gerenciamento visual de Mods, Modpack, Datapacks, Texturas/Resource packs, Mundos, Arquivos, jogadores, configurações e backups. Permita buscar/importar um conteúdo e preparar download com o jogo aberto; aplique com integridade, índice, dependências, fila de reinício e recuperação, sem trocar JAR em execução ou modificar a instância original. Distribua resource packs por um transporte realmente acessível e validado, sem fingir que são mods do servidor. Entregue controle do dono sobre nome, imagem, MOTD, acesso, memória, conteúdo e ciclo de vida, preservando limites dos processos externos e permissões dos convidados. Integre entrada pelo Minecraft por endereço sem convite obrigatório no modo aberto, mantendo acesso privado opcional, métricas reais de jogadores online/únicos/offline e ban/desban persistentes; valide LAN do Plus e P2P entre dois PCs antes de declarar pronto; implemente o endereço com transporte real, sem inventar domínio funcional, salvamento automático confirmado, backups consistentes e retenção. Corrija os ícones dos mods instalados e preserve as animações e a fluidez, medindo resultados. Alinhe benefícios reais do launcher, site e README em PT/EN/ES. Preserve contas, instâncias, mundos e alterações locais; execute as verificações pertinentes e prepare a instalação local para eu testar. Informe testes aprovados, pendências e limites sem garantir ausência absoluta de falhas. Não publique, não compre domínio/infraestrutura, não inicie Minecraft nem aceite EULA por mim.
