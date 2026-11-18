# Plano de servidores próprios Luxmc — 27/10/2026

Status: planejamento solicitado pelo mantenedor. Não altera o aplicativo instalado, não provisiona infraestrutura e não apaga dados. Substitui a proposta de recuperação automática de servidores excluídos. Não promete ausência absoluta de falhas.

## Resultado esperado

O jogador cria um servidor do zero no Luxmc, escolhe Vanilla, loader ou modpack em um catálogo próprio do fluxo de servidores, configura o mundo e inicia. Nenhuma instância de jogo precisa estar instalada. O mundo é novo, independente e salvo no PC anfitrião. A interface organiza o trabalho; um serviço separado mantém o processo e a conexão vivos quando a janela fecha. Não há cadastro obrigatório em provedor de túnel. A referência de sequência é um painel de hospedagem como o Aternos, sem copiar sua identidade e sem depender de sua infraestrutura.

Há duas modalidades distintas: LAN/P2P com Luxmc nos participantes e endereço público compatível com Minecraft padrão. Não apresentar uma delas como equivalente à outra. Pro permite servidor dedicado com até 20 jogadores e configuração até 8192 MiB. Os limites LAN são separados: Plus até 5 participantes e Pro até 10, incluindo o anfitrião. Esses limites são regras do produto, não garantia de desempenho.

## Planos e aplicação das regras

Esta revisão do planejamento fixa Plus em R$ 20,00 e Pro em R$ 30,00 por 30 dias, substituindo os preços anteriores nesta proposta. Valores em reais, sem conversão cambial no checkout. Ainda não altera preços publicados ou assinaturas reais.

| Regra | Plus | Pro |
| --- | --- | --- |
| Preço por 30 dias | R$ 20,00 | R$ 30,00 |
| Rede LAN pelo Luxmc, incluindo anfitrião | Até 5 pessoas | Até 10 pessoas |
| Criar e hospedar servidor dedicado | Não | Sim |
| Jogadores no servidor dedicado, incluindo dono conectado | Não se aplica | Até 20 |
| Heap configurável do dedicado | Não se aplica | Até 8192 MiB, sujeito à memória disponível |
| Demais benefícios existentes | Preservados conforme implementação | Preservados conforme implementação |

O limite de 20 no dedicado permanece da definição anterior; o novo limite de 10 solicitado se refere à LAN do Pro. Não reduzir silenciosamente a capacidade do dedicado nem anunciar recursos futuros como benefícios entregues. Convidados podem jogar no dedicado sem assinar Pro; a permissão de hospedagem pertence ao anfitrião. Não bloquear conexão direta normal do Minecraft por causa do plano do convidado.

Aplicar permissões em handlers nativos e no serviço que coordena a rede, além da interface. Plus não pode contornar a regra chamando a API de criação, início ou compartilhamento do dedicado. Validar capacidade ao entrar, inclusive entradas simultâneas, reconexão e saída; não usar apenas o número mostrado no painel. Diferenciar participantes da rede e jogadores autenticados no servidor.

A alteração precisa ser coordenada no checkout, catálogo de benefícios, site PT/EN/ES, documentação, launcher e regras de autorização. Criar uma versão identificável da tabela de planos para evitar valores divergentes. Pedidos existentes mantêm o preço registrado; novos pedidos usam R$ 20/R$ 30 após publicação. Não cobrar diferença nem modificar pagamento existente retroativamente. Expiração ou redução de plano impede novas ações de hospedagem conforme política apresentada, mas preserva parar/salvar/exportar e nunca encerra Java abruptamente nem apaga mundos.

Validar limites Plus 5/6 e Pro 10/11 participantes, duas entradas disputando a última vaga, recusa do dedicado para Plus e liberação para Pro. Incluir preços em novos pedidos, preservação de pedidos antigos e consistência da informação pública. Regras de LAN do Free permanecem fora desta alteração.

## Base existente e lacunas

O projeto usa Tauri/Rust, processo separado server_worker e Iroh para P2P. Minecraft Java continua sendo executado pelo servidor oficial ou pelo loader compatível; não é reimplementado em Unreal Engine. Engenharia reversa não é necessária para esta arquitetura.

A revisão .904 tem gerenciamento, conteúdo, configurações e testes automatizados, mas ainda exige partidas reais entre redes para validar o conjunto. O fluxo Playit foi retirado. Não existe infraestrutura própria provisionada para ingresso TCP público. O site atual não fornece esse ingresso.

## Arquitetura proposta

1. Interface Svelte: configura e acompanha estado; nenhuma operação longa bloqueia a interação. Criação, importação, início e exclusão mostram progresso, cancelamento quando seguro, prazo e erro acionável.
2. Serviço local Rust: supervisiona Java, arquivos, salvamento, conexão e métricas. IPC autenticado apenas local. Fechar a janela preserva hospedagem; parar e salvar encerra o servidor. Atualização do serviço não interrompe uma partida silenciosamente.
3. Servidor Java: diretório próprio, identidade estável, porta estável quando disponível, autenticação oficial, mundo independente e configuração aplicada no início. Mudança pendente de reinício fica visível.
4. Rede Luxmc: primeiro tenta conexão direta entre participantes; usa relay criptografado quando necessário. A interface distingue servidor pronto, rede conectada e caminho realmente testado.
5. Entrada pública opcional: gateway TCP com IP público, domínio controlado e túnel autenticado iniciado pelo anfitrião. Jogador externo usa somente Minecraft; servidor e mundo continuam no PC do dono.

## Endereço público sem Playit

Operar gateway e serviço de controle sob responsabilidade Luxmc. Para o primeiro piloto, uma máquina com IP público pode concentrar ambos, com isolamento, limites e monitoramento. Produção exige avaliar disponibilidade, tráfego, abuso e proteção contra ataques.

O serviço de controle autoriza o anfitrião, reserva identificador e porta/rota, renova sessões e fornece diagnóstico. Não confundir login de controle com autenticação dos jogadores no Minecraft. O gateway publica a entrada TCP e encaminha ao serviço local por conexão autenticada, com reconexão e limites de filas e de conexões.

Nomes como owner.jogar.dominio-controlado.example são exemplos, não endereços ativos. Usar DNS A/AAAA e SRV conforme a estratégia. Começar com porta atribuída por servidor; compartilhar uma porta entre nomes exige roteador que interprete corretamente o handshake Minecraft, inclusive ping, login e loaders. DNS sozinho não atravessa CGNAT e não cria hospedagem.

Custos e capacidade precisam de orçamento medido antes de lançar. Não contratar serviços nesta etapa. Não prometer proteção DDoS completa ou disponibilidade contínua sem infraestrutura que sustente isso. Bibliotecas de rede podem ser reutilizadas; o usuário vê apenas Luxmc, sem conta de provedor de túnel.

## Assistente de criação independente

Este fluxo substitui a criação baseada em instância. A tela inicial tem Criar servidor e lista dos servidores existentes. Nenhuma consulta ao catálogo depende de encontrar um perfil local. Cada etapa permite voltar sem perder o preenchimento e mantém um rascunho sem iniciar downloads ou Java.

1. Identidade: nome, ícone opcional e descrição no Multijogador. O endereço é tratado separadamente e só aparece como público depois de provisionado e verificado.
2. Software: Vanilla, Com mods (Fabric, Quilt, Forge ou NeoForge), ou Modpack. Servidores com plugins, como Paper, ficam como extensão futura explícita até haver instalador e testes próprios; não publicar opção que não funciona. Plugins e mods não são intercambiáveis.
3. Versão: Vanilla/loader oferecem somente combinações obtidas de metadados oficiais. Modpack abre busca Modrinth/CurseForge, filtros, capa, autor, versões publicadas e situação de suporte a servidor. Escolher uma versão do pack fixa Minecraft, loader e dependências; não permitir combinação arbitrária incompatível.
4. Mundo e regras: mundo novo por padrão, seed opcional, tipo de geração suportado, estruturas quando aplicável, Sobrevivência/Criativo/Aventura/Espectador conforme versão, dificuldade, hardcore quando suportado, PvP e limite de jogadores. Regras como manter inventário ficam em opções adicionais e são aplicadas pelos mecanismos reais da versão. Valores não suportados ficam ausentes ou explicados. Não confundir modo de jogo de novos jogadores com alteração de jogadores existentes.
5. Recursos e acesso: RAM sugerida com ajuste de 512 a 8192 MiB dentro do disponível, distâncias de visão/simulação, acesso aberto ou whitelist, opções de conexão com estado real e salvamento. Pro permite até 20 jogadores; a sugestão inicial considera capacidade, sem prometer que o limite inteiro rodará bem. Configurações avançadas ficam recolhidas.
6. Revisão: mostrar software e versões exatas, mundo novo, RAM, estimativa de download/disco quando conhecida, compatibilidade e requisitos dos convidados. A aceitação da EULA pertence ao usuário. A ação Criar e preparar reserva a pasta e começa a instalação; iniciar é uma ação explícita após a preparação, podendo ser escolhida nessa revisão.
7. Preparação: resolver pacote, baixar/verificar arquivos, instalar Java/loader adequado, aplicar configurações e marcar preparado. A geração do mundo e sua confirmação ocorrem no primeiro início do Java, não no simples cadastro. Exibir fase, bytes/itens quando conhecidos, cancelar com limpeza apenas dos temporários próprios e tentar novamente sem duplicar servidor. Não inventar porcentagem ou prazo.

Cada servidor recebe UUID e diretório próprios, com mundo, mods, configurações e backups independentes. Dois servidores criados do mesmo pack podem ter regras e mundos diferentes. Não copiar saves, opções ou autenticação do cliente. Importar um mundo existente é uma operação avançada posterior e explícita. Servidor e cliente não abrem o mesmo diretório de mundo.

## Instalação de modpacks pelo catálogo

Preferir o pacote de servidor disponibilizado pelo autor e associado à versão escolhida. Modrinth permite preparar o ambiente servidor pelo manifesto e server-overrides. CurseForge pode informar um serverPackFileId correspondente. Respeitar URLs, autorizações, termos e arquivos indisponíveis; não contornar restrições de download nem prometer todos os packs do catálogo.

Classificar cada versão como pacote de servidor disponível, instalação por manifesto validável, somente cliente, ou suporte ainda não determinado. Somente cliente não vira servidor por remover alguns mods às cegas; oferecer criar um Vanilla/loader separado e explicar o requisito do cliente. Suporte desconhecido não recebe selo compatível e pode exigir pacote manual em área avançada. Um pacote cliente de otimização não implica que haja um servidor de mesmo nome a instalar.

Conferir versão Minecraft, Java, loader, dependências e integridade antes de ativar a instalação. Excluir arquivos declarados cliente e apresentar relatório. Ausência de declaração não prova compatibilidade. Pacotes que exigem procedimentos especiais devem ter adaptador validado ou bloqueio explicado; não executar scripts arbitrários baixados.

Persistir origem, ID do projeto, versão, arquivos instalados e hashes. Preparar em diretório temporário e só ativar após validação; falha mantém estado diagnosticável, sem servidor apresentado como pronto. Troca de pack/software em servidor existente exige parada, revisão de compatibilidade e decisão sobre o mundo; não reutilizar silenciosamente um mundo incompatível. Atualizações são explícitas, sem alterar mods durante uma partida.

O botão Jogar prepara ou seleciona separadamente um perfil cliente compatível após autorização do usuário. Não exige cliente previamente instalado e não inicia Minecraft ao criar o servidor. Convidados recebem informação da versão exata e dos mods exigidos. Mods usados apenas pelo servidor não são impostos ao cliente sem necessidade.

## Conteúdos e controle posterior

Após criação, o dono pode adicionar/remover mods compatíveis, datapacks, recursos e importar mundo. Conteúdos mostra inventário real e pendências, independentemente de ter sido instalado por catálogo ou individualmente. Preparar downloads com o servidor ligado é permitido; aplicar mudanças que exigem parada fica explícito. Customizar pack marca a instalação como modificada; atualizações mostram conflitos e não apagam personalizações silenciosamente.

Shaders pertencem ao cliente. Resource pack automático exige arquivo distribuível por HTTPS e integridade; não tratar ZIP local como download público. Mundo exige validação de arquivos/versão e substituição explícita com servidor parado. Arquivos de outros servidores e instâncias permanecem isolados.

## Entrada do dono como em um servidor normal

O dono inicia o servidor dedicado e aguarda a confirmação real de que está aceitando conexões. Depois abre um cliente compatível e entra pelo menu Multijogador. Não precisa criar sala Luxmc, conectar a LAN, abrir mundo solo para LAN ou ativar P2P para acessar um servidor no mesmo PC.

No próprio PC, mostrar endereço 127.0.0.1:porta-estável como entrada local, com copiar e opção explícita Adicionar ao Multijogador da instância escolhida. Essa opção altera somente a lista de servidores do cliente selecionado, preserva favoritos existentes e exige acesso seguro ao arquivo; se o jogo estiver aberto, orientar adição pelo próprio Minecraft em vez de sobrescrever servers.dat em uso. O endereço local pode ser salvo como servidor normal e continua válido enquanto a porta se mantiver.

Exemplo: servidor Vanilla 26.3 pronto, dono abre Vanilla 26.3 e conecta ao endereço local. O mundo aberto é o diretório independente do servidor, não um save da instância cliente. O dono conta como um jogador apenas ao se conectar; iniciar o serviço não ocupa uma vaga de jogador.

Para servidor com mods, conferir versão e loader e informar o conjunto necessário no cliente. Um Vanilla não passa a ser compatível com qualquer servidor modificado por ser do mesmo dono. Se houver whitelist, incluir a identidade Minecraft verificada do dono conforme a opção escolhida; não desligar autenticação nem conceder operador automaticamente como solução de conexão.

Amigos em outros PCs não usam 127.0.0.1, pois esse endereço aponta para o próprio computador de cada um. Eles usam o endereço público quando disponível, ou a conexão P2P pelo Luxmc enquanto a infraestrutura pública estiver pendente. A entrada local do dono deve funcionar independentemente dessa publicação, da sala LAN e da disponibilidade do relay.

Critérios: dono entra sem sala criada e sem sessão P2P, reconecta após reinício preservando o mundo, vê o servidor salvo no Multijogador e recebe erro claro se Java estiver parado ou o cliente incompatível. Validar separadamente que fechar só a janela Luxmc mantém a partida e que Parar e salvar desconecta após o salvamento normal.

## Eficiência com medição

Executar Java em modo servidor sem janela de jogo, sem iniciar cliente em segundo plano. Selecionar Java compatível e compartilhar apenas downloads imutáveis verificados em cache; mundos e configurações mutáveis permanecem separados. Não usar hardlinks de mods editáveis como se fossem cópias isoladas. Limpeza de cache não remove arquivos em uso.

Medir RAM do processo além do limite de heap, CPU, espaço, duração de início, MSPT/TPS quando obtidos com confiabilidade e tráfego. Informar quando uma métrica não está disponível. O limite de 8 GiB é de heap configurado, não teto total de RAM do PC. Somar sistema, cliente e servidor ao recomendar recursos.

Perfis Econômico, Equilibrado e Personalizado ajustam parâmetros documentados para a versão, sobretudo distâncias e capacidade sugerida. Mostrar o que muda e preservar escolhas do dono. Não instalar mods de desempenho ou alterar geração/regras sem consentimento. Não prometer igual desempenho ao reduzir recursos: pouca RAM pode aumentar pausas e causar falha.

Sem jogadores, reduzir trabalho quando o software oferecer suporte confiável; pausa ou parada por inatividade são opcionais, com salvamento e explicação do efeito na reconexão. Não parar por padrão um servidor que o dono pretende manter disponível. Evitar pré-geração extensa automática e logs/renderizações ilimitados.

Comparar o mesmo mundo, versão, máquina e carga antes/depois das otimizações. Medir primeira preparação separada de reinícios com cache. Um pack pesado pode precisar de mais que 8 GiB: informar que fica fora do limite atual, sem prometer resolver com flags. O launcher mantém um servidor gerenciado ativo por vez nesta fase; vários podem ser criados e armazenados.

## Painel e experiência

Lista de servidores compacta, ação Hospedar em destaque e um servidor selecionado. Cabeçalho com nome, estado e iniciar/parar; navegação estável para Visão geral, Conteúdos, Jogadores, Console, Configurações e Backups. Usar componentes e tokens do tema, hierarquia clara e estados de carregamento por operação.

Visão geral mostra endereço utilizável, copiar, conexão, jogadores, RAM e salvamento confirmado. Jogadores mostra avatar, nome, online/offline e ações de expulsar/banir/desbanir. Configurações distinguem gravado, aplicado e reinício necessário. Backups são cópias explícitas do usuário e não uma lixeira de servidores.

Sem bloqueio global invisível ou carregamento infinito: operações identificadas, respostas atrasadas descartadas ao trocar servidor, atualização incremental e reconexão do painel. Console limitado e renderização eficiente para logs intensos. Navegação por teclado, foco correto e movimento reduzido respeitados.

## Reforma visual de todo o launcher

Pedido adicional do mantenedor: reformar o design do aplicativo inteiro, com prioridade para a área nova de servidores. Esta etapa é parte do escopo de implementação, não somente uma sugestão estética. Não se limita a trocar cores dos botões ou acrescentar animações sobre a organização existente.

### Linguagem visual e componentes

Antes de redesenhar telas, revisar os tokens existentes de superfície, contraste, tipografia, espaçamento, raios, elevação e cores de estado. Preservar identidade Luxmc e personalização de temas; usar tokens, sem cores hexadecimais avulsas. Definir componentes coerentes para botões primários/secundários/destrutivos, campos, seletores, abas, cartões, menus, diálogos, avisos, tooltips e indicadores de progresso.

Cada componente deve ter estados normal, hover, foco de teclado, pressionado, carregando, desabilitado e erro quando aplicável. Prioridade visual corresponde à ação: iniciar/criar em destaque, ações secundárias discretas porém legíveis, apagar distinguível sem competir com a ação principal. Ícone e texto claros; controles essenciais não dependem exclusivamente de hover ou de ícone sem identificação.

Hierarquia de títulos, largura de leitura, alinhamento, densidade e espaçamento devem ser consistentes. Evitar painel dentro de painel sem função, excesso de bordas e textos técnicos em áreas de uso comum. Explicações avançadas ficam acessíveis por expansão. Wallpaper e efeitos de fundo não atravessam a legibilidade dos controles; limitar opacidade e aplicar superfície adequada aos temas claros e escuros.

### Revisão por área

| Área | Resultado exigido |
| --- | --- |
| Servidores | Assistente independente organizado por etapas, catálogo visual, progresso legível e painel com navegação estável; principal prioridade da reforma |
| Início e biblioteca | Cartões consistentes, capas sem deformação, hover restaurado, jogar e ações identificáveis, busca e organização claras |
| Descoberta de conteúdos | Busca/filtros com hierarquia, capas adequadas ao tipo de conteúdo, compatibilidade e estados de instalação compreensíveis |
| Detalhes e configurações de instância | Seções claras e barra de ações no lugar correto, sem cobrir campos durante rolagem; alterações pendentes visíveis |
| Amigos e LAN | Separar pessoas, salas e estados de conexão; criar/entrar com instruções curtas e ações claras |
| Conta e benefícios | Identidade, plano e recursos realmente disponíveis apresentados com clareza; sem promessas genéricas ou informação contraditória |
| Personalização | Prévia fiel de tema, wallpaper e efeitos, com controles de intensidade e opção de desativar animações |
| Configurações gerais | Categorias consistentes, busca quando útil, valores atuais, explicações curtas e feedback real de aplicação |
| Downloads, notificações e erros | Progresso persistente, mensagens acionáveis e acesso ao diagnóstico sem bloquear toda a interface |

### Animações e efeitos

Adicionar transições de navegação e abas, entrada/saída de diálogos, expansão de detalhes, hover/pressionamento dos cartões, feedback de botões e confirmação de ações. Usar movimento para indicar relação entre estados e continuidade de tarefas. Ações rápidas não devem esperar uma animação terminar para aceitar o próximo comando.

Usar, como ponto de partida ajustável, microinterações de 100–180 ms e mudanças de painel de 180–260 ms, com curvas consistentes. Preferir transform e opacity; limitar blur, sombras animadas e animações contínuas em listas grandes. Efeitos de brilho, gradiente e profundidade ficam subordinados à leitura. Não repetir animação de entrada a cada atualização de métricas, log ou consulta de servidor.

Respeitar prefers-reduced-motion e a preferência interna do usuário. Oferecer intensidade de efeitos sem alterar funções. Evitar flashes, parallax obrigatório, áudio automático e movimento contínuo sobre textos. Cancelar animações obsoletas ao trocar rapidamente de tela; garantir estado final e foco corretos mesmo quando animações estiverem desativadas.

### Configurações e comportamento

Revisar os controles de configuração junto do visual: faixa válida, unidade, padrão, dependências, persistência e efeito real. Distinguir ajuste imediato, aplicado ao salvar e pendente de reinício. Não mudar silenciosamente preferências existentes para combinar com o novo design. Mudanças com impacto no jogo, memória, disco ou conexão precisam de descrição concreta.

Definir política consistente de salvar: alterações explícitas com ação visível nas telas que exigem confirmação, ou salvamento automático com indicação de resultado nas telas apropriadas. Ao sair com alterações pendentes, permitir salvar, descartar ou permanecer. Nunca exibir sucesso antes da confirmação da operação. Erros devem preservar o preenchimento.

### Sequência e aceite visual

1. Inventariar telas e componentes atuais e registrar problemas concretos de layout, interação e contraste.
2. Consolidar tokens/componentes e aprovar uma tela representativa de servidores e uma da biblioteca antes de propagar a linguagem visual.
3. Reformar criação e painel de servidores; depois biblioteca, descoberta, configurações, amigos e demais áreas, mantendo funções existentes.
4. Verificar estados com dados reais e fictícios: vazio, carregando, muitos itens, texto longo, falha, confirmação, servidor ativo e operação cancelada.
5. Conferir pelo menos 960×600, 1366×768 e 1440×900, além de escala do Windows e janela redimensionada. Sem overflow horizontal involuntário, ações cortadas, barra sobre conteúdo ou diálogos inacessíveis.
6. Validar teclado, foco visível, Escape, retorno de foco, contraste e movimento reduzido. Capturar antes/depois e inspecionar visualmente as telas prioritárias.
7. Medir fluidez com logs e downloads ativos, muitos cartões e wallpaper habilitado/desabilitado. Comparar CPU/RAM da interface antes/depois; corrigir regressões relevantes em vez de ocultá-las com animações.

O critério de conclusão é uma interface coerente, legível e responsiva com operações funcionando. Quantidade de efeitos não substitui organização ou validação funcional. Nenhuma reformulação visual deve ser apresentada como já instalada enquanto permanecer somente neste plano.

## Exclusão definitiva

Remover a seção Servidores excluídos e o comportamento de arquivar como resultado normal de Excluir. Mostrar confirmação com nome exato e tamanho conhecido, informando que mundo e conteúdos serão removidos sem restauração pelo aplicativo.

Exigir servidor parado e impedir corrida com início, importação, backup ou conexão. Validar UUID, diretório absoluto pertencente ao Luxmc e ausência de links/junctions que escapem da pasta. Excluir apenas os diretórios verificados desse servidor. Desconectar sessões e liberar eventual rota pública autenticada.

Oferecer exportação antes da confirmação final, sem criar cópia oculta obrigatória. Backups independentes terão escolha explícita de manter ou apagar, com padrão e consequência descritos na confirmação. Cache compartilhado, instância de origem e arquivos de outros servidores nunca entram na exclusão.

Os servidores já arquivados pela .904 não serão purgados automaticamente na atualização. A migração permitirá exportar ou confirmar a remoção definitiva desses dados, sem manter o painel normal de restauração. Se houver falha de filesystem, informar quais dados permaneceram e permitir nova tentativa; não anunciar sucesso falso. Exclusão comum não promete apagamento forense de SSD ou de cópias externas.

## Salvamento e continuidade

Manter salvamento nativo e solicitar flush periódico; registrar confirmação real. Encerramento normal salva e aguarda saída; timeout leva a opção explícita de forçar, com risco informado. Tratar falta de disco e falha de backup como erros reais. Não reiniciar indefinidamente um servidor que falha na preparação.

Fechar janela não encerra o serviço. Reiniciar Windows, suspender o PC, perder energia ou internet interrompe a sessão; iniciar com o sistema será configuração explícita futura, não pressuposto. Não confundir serviço separado com serviço já instalado no Windows ou disponibilidade 24 horas.

## Etapas e critérios de aceite

1. Consolidar serviço e interface: ciclo criar/iniciar/parar/reabrir; cancelamento; exclusão definitiva e migração sem purge automático. Aprovar navegação, foco, contraste e ausência de travamento em resoluções menores.
2. Implementar assistente e catálogo independentes: criar Vanilla sem nenhum perfil instalado, loader sem mods, pack Modrinth com ambiente servidor e pack CurseForge com pacote oficial disponível. Cobrir pack somente cliente, download indisponível, cancelamento, falta de disco e reinício da preparação. Criar dois servidores do mesmo pack e provar isolamento de mundos/configurações. Conferir regras realmente aplicadas e estado persistente após reinício. Testar combinações escolhidas em jogo sem declarar suporte universal.
3. Validar LAN/P2P: dois PCs e duas redes, incluindo CGNAT, tentativa direta, relay, queda de internet e retorno. Separar limite de sala e jogadores realmente autenticados no Minecraft. Testar GUI fechada durante a partida.
4. Construir piloto de gateway próprio apenas após infraestrutura e orçamento autorizados: domínio real, túnel autenticado, rota TCP, revogação, reconexão e limites. Testar de rede externa com Minecraft sem Luxmc e sem convite obrigatório para servidor público.
5. Liberar gradualmente: logs sem credenciais, métricas de falha/latência/tráfego, versão de protocolo e atualização compatível. Só anunciar cada modalidade depois da evidência correspondente.

Verificações de código: pnpm check após TS/Svelte, cargo check após Rust, testes de contrato e regressão dos caminhos alterados, build e conferência do instalador. Testes de rede e jogo precisam de participantes e contas autorizados; mocks não substituem essa evidência.

## Limites assumidos

P2P direto não é garantido em todo NAT/firewall; relay possui custos e disponibilidade. Minecraft padrão não fala o protocolo Iroh sozinho. Gateway público adiciona latência e dependência externa. Upload, CPU, disco e mods do anfitrião limitam desempenho. Portas UDP de mods de voz precisam de suporte específico. Nenhuma biblioteca ou engenharia reversa elimina essas restrições.

## Fontes técnicas

- Iroh: https://docs.rs/iroh/latest/iroh/ e https://github.com/n0-computer/docs.iroh.computer/blob/main/add-a-relay.mdx
- Formato de modpack: https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack
- Associação de pacote de servidor CurseForge: https://docs.curseforge.com/rest-api/
- Servidores com loader: https://fabricmc.net/use/?page=server e https://docs.neoforged.net/user/docs/server/
- Distinção de proxy TCP/UDP: https://developers.cloudflare.com/spectrum/reference/configuration-options/
