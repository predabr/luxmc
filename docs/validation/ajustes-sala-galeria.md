# Luxmc — ajustes locais de sala, recursos e galeria

Data: 10 de outubro de 2026. Versão preservada: 3.7.0. Base Git: e39331c9268fb8a539739543084633fbef7607c2.

## Alterações

- Luxmc Essentials no Java: ordem corrigida depois de vanilla e antes dos pacotes pessoais; migração da ativação anterior; preservação das opções padrão de modpacks que usam YOSBR. Desativar manualmente depois da migração continua sendo respeitado.
- Skins e capas: troca de PNGs pelo túnel privado, com limites de tamanho e dimensões, associação à identidade admitida na sala e expiração local. Funciona com UUID da conta e com o UUID de LAN offline correspondente ao nome. Texturas de pessoas fora da sala conservam o tratamento anterior.
- Sala: a aba selecionada nos Amigos é conservada durante a navegação, inclusive ao chegar pela tela de sala independente.
- Instalação: uma conferência principal, correção de duplicatas apenas quando necessárias, leitura limitada em memória para arquivos de até 32 MiB e contador de arquivos durante a etapa final. Gravação de downloads com buffer de 1 MiB e reutilização do pacote por URL, mantendo verificação de integridade e descarte de downloads inválidos.
- Inicialização: reutilização da aparência instalada quando conteúdo, modelo, capa e versão não mudaram. A verificação de segurança dos mods permanece na sequência de lançamento.
- Galeria: cabeçalho redesenhado, acesso à biblioteca, imagens de fallback visíveis e sem cortar ícones ou cartazes, animação de entrada nos cards e interação de hover. Preferências de movimento reduzido e economia são respeitadas.
- Discord: o anfitrião conta entradas e saídas registradas pelo servidor integrado; o cliente consulta o status do servidor conectado. O número de participantes da sala não é usado como número de jogadores da partida.
- Site: Skin Studio, navegação, demonstração, scripts, código 3D e referências de sitemap removidos. Links antigos preparados para redirecionar à página principal. O personalizador do launcher continua disponível; o botão antigo de Web Studio foi removido.
- Compilação do site: arquivos gerados idênticos são preservados e alterações usam arquivo temporário e substituição atômica. Isso resolveu a falha de escrita em arquivos mapeados pelo Windows durante esta verificação. Os caminhos de saída são limitados à pasta do site.

## Validação concluída após a retomada dos comandos

- pnpm check: zero erros e zero avisos Svelte/TypeScript.
- pnpm build: compilação de produção do frontend concluída.
- pnpm build:website: compilação local do site concluída.
- Frontend: 153 testes passaram.
- Site: 54 testes passaram, incluindo três regressões da gravação dos arquivos gerados.
- Interface: 18 suítes verificaram catálogo, conteúdo, criação, navegação, personalizador, salas, perfis, cancelamento tardio, fila, retomada, temas, wallpapers, movimento reduzido e opções Java. O teste novo confirmou retorno à sala, imagem de fallback e contador do Discord.
- Agente Java: 15 execuções headless passaram com Authlib 1.5.21, 4.0.43, 6.0.54, 7.0.63 e 10.0.77 nos Java 17, 21 e 25 disponíveis. Cobertura de skin, capa, modelo, identidade da sala, expiração e identidade não relacionada.
- cargo check --tests e cargo check: a revisão final compila o código Rust e os testes, com avisos de código não utilizado já existentes.
- git diff --check: sem problemas de espaços.

## Limites desta rodada

A execução de cargo test --lib foi bloqueada pela política de Controle de Aplicativo do Windows (erro 4551). Após a retomada, cargo test --lib network:: -- --test-threads=1 compilou a revisão final, mas recebeu o mesmo bloqueio antes de iniciar qualquer teste. Os testes Rust não foram executados. Nenhuma proteção do Windows foi desativada ou contornada. O benchmark Rust novo de arquivos instalados também não pôde ser executado; não há uma medição comparativa de velocidade desta alteração.

O usuário confirmou o P2P no vanilla 26.3 e informou uma falha de descoberta no One Terror Block 1.20.1: o mundo não ficou disponível na sala do Luxmc nem na lista LAN do amigo. O log local registra a abertura LAN desse modpack, que usa Forge 47.4.10. Foram encontradas fragilidades na descoberta do launcher; a causa exata da tentativa não foi comprovada por uma reprodução em dois PCs.

Nesta rodada, Minecraft e o launcher nativo não foram iniciados. A nova troca de aparência e a presença foram verificadas nos adaptadores Java e na interface; a confirmação visual dentro de uma partida em dois PCs fica para a próxima rodada autorizada.

Não foi criado instalador, alterada a versão, feito commit/push ou publicado o site. O aplicativo instalado e o site público continuam com a publicação anterior. Contas, mundos e instâncias reais não foram alterados por estes testes.

## Revisão da descoberta LAN em modpacks

- O agente Java informa a porta depois de o servidor integrado concluir a abertura TCP. A identificação é limitada a classes derivadas de MinecraftServer, com o método de publicação LAN reconhecido. Outros servidores de mods e tentativas de abertura que falham não alteram esse registro.
- O registro local contém o PID do jogo e uma porta válida. O launcher verifica a conexão local antes de anunciar o mundo. Registros de outra execução do jogo são rejeitados.
- A leitura incremental do log conserva a porta quando muitos registros de mods a empurram para além dos antigos 64 KB. Reconhece mensagens de Forge, versões antigas e vanilla recente; acompanha fechamento e reabertura do mundo.
- A leitura acompanha a troca do arquivo e processa até 4 MiB por consulta, avançando na consulta seguinte. O limite é aplicado ao tamanho observado na abertura; um mod produzindo logs continuamente não mantém a consulta em um ciclo sem fim.
- Uma falha ao escutar os anúncios multicast não impede consultar os sinais do jogo. A busca multicast continua disponível como alternativa. O transporte QUIC, os convites e a autenticação da sala foram preservados.
- A conferência de compatibilidade dos mods acontece em segundo plano, sem bloquear o anúncio do mundo.
- Nas últimas edições, portas informadas pelo agente e pelo log são consideradas juntas. Uma porta anterior fechada não descarta a nova porta. Quando a porta do jogo responde, a descoberta dispensa a espera de 2,2 segundos do scanner multicast e evita sondagens TCP duplicadas.
- A descoberta e o encaminhamento do túnel usam IPv4 local primeiro e tentam o loopback IPv6 quando a conexão IPv4 é recusada. Essa alternativa atende servidores integrados que escutam somente em IPv6, conservando o caminho IPv4 existente. Foram escritas regressões Rust de transferência TCP nos dois tipos de listener; a execução dessas regressões segue bloqueada pela política do Windows.
- O teste do canal real encontrou uma recusa ao mudar imediatamente de uma sessão expirada para outra porta válida: o agente Java conservava o estado anterior por 500 ms. Esse cache foi removido. Cada tentativa de login consulta o registro atual, limitado a 512 bytes, e continua exigindo porta válida, sessão não expirada e conexão local.

Antes da pausa, o teste headless do agente passou nos Java 17 e 25: abertura TCP, PID, falha de bind e rejeição de listener não relacionado. A transformação foi verificada no bytecode instalado do vanilla 1.8.9, 1.20.1, 1.21.1 e 26.3, além do Forge 1.20.1-47.4.10 usado pelo One Terror Block. Isso verifica os pontos de integração; não equivale a uma partida real com esses modpacks.

Após a correção do cache, passaram os testes do login privado e do canal TCP Netty real do Minecraft 26.3 em IPv4 e IPv6, preservando autenticação em outras portas e após a expiração da sessão. O teste alterna imediatamente entre sessões, sem esperar o antigo prazo de cache. O agente de descoberta também foi revalidado nos Java 17 e 25, e os 15 testes de aparência passaram novamente com o JAR final.

A pausa solicitada durante Fortnite foi respeitada. Depois de o usuário liberar os comandos, foram retomadas compilação e validação. A galeria, o retorno à sala, o contador do Discord, as orientações LAN e o cancelamento tardio também passaram novamente nas duas suítes de interface direcionadas.

O Luxmc Shield continua sendo chamado antes de iniciar Java e novamente depois de um hook pré-lançamento. A implementação de hash por conteúdo e o bloqueio de achados críticos foram preservados. Essa sequência foi revisada no código; os testes nativos do scanner não puderam ser executados por causa do bloqueio do Windows.

Permanecem pendentes a execução das regressões Rust quando permitida pela política do Windows e a repetição do teste One Terror Block nos dois PCs com uma futura instalação das alterações. A publicação e o aplicativo instalado continuam com a versão anterior do código 3.7.0.
