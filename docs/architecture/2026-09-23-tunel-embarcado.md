# Revisão: túnel embarcado, skins e isolamento

Esta revisão substitui a dependência de Tailscale da entrega anterior no fluxo principal de multiplayer. O código foi implementado nos arquivos do projeto; não é uma proposta de arquitetura.

## Como usar

Em **Amigos & Rede → P2P**, o anfitrião abre seu mundo para LAN no Minecraft, detecta/informa a porta e escolhe **Compartilhar mundo**. O launcher gera um convite privado `luxmc-world:…`, válido por uma hora. O amigo cola o convite, conecta e usa **Entrar Agora**. O Minecraft conecta a uma porta aleatória em `127.0.0.1`; o Rust encaminha os bytes à porta LAN do anfitrião.

Não exige Tailscale, adaptador TUN, root, instalação no SO ou conta adicional. O convite longo contém uma capacidade secreta; somente quem o recebe deve ter acesso. Foi escolhida a alternativa de convite privado prevista no pedido, não um código curto global que dependeria de um serviço de rendezvous próprio. A sessão não é publicada automaticamente na lista de amigos. O anfitrião deve permanecer com launcher e mundo abertos.

## Rede implementada

- `src-tauri/src/network/p2p_tunnel.rs`: biblioteca iroh 1.2.0 embarcada, QUIC/TLS, identidade criptográfica do endpoint, autenticação adicional com segredo aleatório de aproximadamente 244 bits.
- Comandos Tauri e daemon: `host_world`, `join_world`, `stop_session`, `tunnel_status`.
- Transporte direto com hole punching; fallback pelos relays públicos padrão de iroh/n0. Nenhuma instalação ou autenticação do jogador nesses relays.
- O relay transporta tráfego cifrado entre endpoints. Dependência de infraestrutura pública continua existindo: não há promessa de disponibilidade universal de um serviço remoto.
- `tokio::io::copy_bidirectional` liga TCP local ao stream QUIC com backpressure, suporte a EOF em uma direção e limites de conexões/streams. Timeouts de handshake, expiração e cancelamento de tarefas encerram sockets.
- Destino do host restrito a `127.0.0.1:porta`; listener do cliente restrito a loopback. Nenhum proxy TCP aberto na rede local.
- Status do cliente mostra RTT e caminho selecionado real, direto ou relay. Não há interface VPN/IP virtual no kernel: o recurso é um túnel de aplicação para Minecraft Java/TCP. Mods que exigem UDP separado, como alguns de voz, não são transportados automaticamente.

## Referências pesquisadas

- [e4mc — QuiclimeSession](https://github.com/vgskye/e4mc-minecraft-architectury/blob/rererewrite/common/src/main/java/link/e4mc/QuiclimeSession.java): separação entre sessão de controle e tráfego; o projeto também integra Dialtone/iroh. O endereço e4mc/e4mc do pedido não respondeu; foi localizada a árvore efetiva do projeto.
- [Essential — documentação de hosting](https://essential.gg/wiki): uso de ICE/TURN para conectar jogadores. Não foi assumido que seu protocolo proprietário pode ser reutilizado diretamente.
- [Playit — encaminhamento TCP](https://github.com/playit-cloud/playit-agent/blob/master/packages/agent_core/src/network/tcp/tcp_clients.rs): ponte entre conexão remota e serviço local.
- [BoringProxy](https://github.com/boringproxy/boringproxy): túnel reverso com infraestrutura de servidor.
- [BoringTun](https://github.com/cloudflare/boringtun): implementação userspace de WireGuard; o executável não elimina sozinho a necessidade de TUN/permissões para uma VPN de sistema. Por isso não foi usado como se resolvesse o requisito sem privilégios.
- [iroh](https://docs.rs/iroh/1.2.0/iroh/): conexões QUIC entre pares, autenticação e fallback por relay. Implementação própria do protocolo Luxmc sobre essa biblioteca, sem copiar código dos projetos pesquisados.

## Skins, capas e vídeo

O renderer acompanha `requestAnimationFrame`, sem o limite anterior de 30 FPS; em uma tela de 60 Hz acompanha 60 quadros por segundo quando o hardware permite. IntersectionObserver, visibilidade e estado do jogo suspendem o loop. A primeira imagem é renderizada explicitamente; perda de contexto WebGL troca para a prévia 2D. O cleanup libera listeners, RAF, geometria, materiais, renderer e contexto.

A cadeia de fallback tenta a textura solicitada, base64 resolvido pelo backend/cache e Steve/Alex do bundle. Falha total de WebGL mantém a prévia 2D local, sem canvas vazio. O fallback visual não substitui silenciosamente a seleção persistida.

Texturas dos provedores conhecidos podem ser resolvidas pelo Rust para evitar CORS. Downloads têm timeout, limite de 3 MB, redirecionamentos restritos e validação PNG. A seleção é normalizada em base64 e persistida junto com capa/modelo no SQLite. O refresh de login deixou de sobrescrever a seleção local. A injeção respeita classic/slim e propaga falhas de arquivo. O pacote gerado é reconstruído para evitar capa antiga e o cache de capa é limpo ao regenerar.

O agente Java foi recompilado e o JAR embarcado foi atualizado. O Shift direito foi removido dos hooks LWJGL2/LWJGL3, do fechamento da janela e do editor de mapeamento dedicado. Shift+Tab continua disponível para acessibilidade de navegação; não abre overlays.

`VideoWallpaper.svelte` mantém o primeiro frame em canvas sob o vídeo. Espera/seek esconde o vídeo temporariamente e revela esse frame, com o mesmo `object-fit: cover`. Isso trata lacunas de decodificação; não modifica frames pretos que estejam codificados dentro do próprio arquivo e não constitui prova de ausência de flashes em todos os codecs/WebKitGTK.

## Interface e instâncias

`GlassSelect.svelte` substitui os selects nativos dos filtros para remover as caixas pintadas pelo sistema. Sidebar direita ocupa a altura da viewport, com rolagem interna e superfície translúcida. Cards de skins/instâncias receberam fundos translúcidos e tokens do tema.

Novas instâncias, importações e duplicações usam ID único com `.minecraft`. Antes de lançar uma instância antiga, a migração copia arquivos para `instances/<id>/.minecraft`, preserva a origem e só atualiza o banco depois da cópia. Colisão de destino e diretórios simbólicos interrompem a migração com erro, sem mesclar pastas. A cópia pode aumentar o uso de disco; a origem antiga não é apagada automaticamente.

## Validações

- `pnpm check`: zero erros/avisos.
- `cargo check`: aprovado.
- Frontend: 50 testes aprovados.
- Rust: 61 testes aprovados; o teste de relay público é marcado como integração opt-in e foi executado separadamente, com sucesso.
- Túnel: três fluxos simultâneos de 1 MiB, comparação byte a byte, EOF, recusa de segredo incorreto e fechamento do listener.
- Relay: endpoints sem transporte IP direto, troca de bytes por relay público e confirmação do caminho selecionado como relay.
- Instaladores reais: Forge 1.12.2/1.20.1 e NeoForge 1.20.4/1.21.1 aprovados.
- Navegador: sidebar em `y=0`, altura 1000 px na viewport de 1000 px; controles transparentes; skin visível com URL remota falhando; sem erros JavaScript. Contas/IPC de interface simulados; testes de rede Rust usaram sockets reais.
- Agente: compilado com JDK 17 `--release 8`; `scripts/build-client-agent.mjs` permite reconstrução com `LUXMC_JAVA_HOME`.

Não foi feita uma partida completa de cada modpack nem uma validação em Windows/macOS. Também não foi medida a memória GPU/RSS de WebKitGTK com vídeo real. O teste de relay comprova transporte real, mas não substitui uma partida entre duas máquinas físicas. A aparência em servidores remotos depende das regras e mods do servidor; esta persistência local não publica skins/capas arbitrárias na conta Microsoft.

Validações adicionais: perda real do contexto WebGL via `WEBGL_lose_context`, fallback 2D e recuperação 3D aprovados; captura do primeiro frame de um vídeo de teste retornou RGBA `[0, 0, 253, 255]` e o vídeo pausou ao iniciar o estado de jogo. Teste prolongado final: heap após GC de 23.435.248 para 26.062.880 bytes, oito ciclos de navegação, 125 segundos e zero erros JavaScript. O crescimento observado ficou abaixo do limite de regressão de 20 MiB; isso não mede memória nativa/GPU.

A revisão dos loaders também passou nos testes reais de Fabric 1.21.4 e Quilt 1.20.1. Fabric/Quilt agora exigem o checksum SHA-1 Maven, validam cache e usam transferência atômica com retry. O teste de Quilt sobrescreveu um JAR com bytes inválidos e confirmou restauração byte a byte do original.

## Aparência por UUID no agente Java

`AppearanceAgent.java` intercepta a resolução de texturas da Authlib apenas para o UUID da conta local. Os métodos `getTextures`, `getPackedTextures` e `unpackTextures` recebem a skin/capa normalizada pelo launcher; outros perfis seguem a implementação original. Autenticação, login e validação de sessão não são interceptados. Para a textura local, o adaptador fornece o estado de confiança esperado pelo cliente; isso não gera assinatura Mojang nem publica a textura na conta Microsoft.

Um servidor HTTP em loopback entrega somente os PNGs carregados, com nomes derivados de SHA-256, limite de tamanho, fila limitada e timeout. O agente usa ASM 9.10.1 vendorizado, com SHA-256 verificado no build, namespace privado e licença BSD inclusa. Um JAR de suporte separado é carregado pelo bootstrap para permitir uso por classloaders dos loaders sem misturar as classes do HUD. Nos modpacks, `appearance-only` desativa a inicialização do HUD e do monitor de teclado.

O antigo resource pack gerado continua como pasta interna dos arquivos normalizados, mas deixa de ser ativado globalmente em `options.txt`: assim não substitui as skins padrão de outros jogadores. Os demais packs são preservados. Um teste Rust confirma isso e a remoção de uma capa antiga. Quando não há skin escolhida para uma conta local, o lançamento usa Steve do bundle, sem depender de um provedor remoto. Falhas ao decodificar a skin escolhida são apresentadas, em vez de produzir um PNG transparente.

Teste reproduzível: `python3 scripts/test-client-agent.py --data-dir /caminho/dos/dados/luxmc`, após `LUXMC_JAVA_HOME=/caminho/do/jdk17 pnpm build:client-agent`. Requer os runtimes e bibliotecas indicados no script já instalados pelo launcher. Foram executados 14 cenários com as implementações reais da Authlib: 1.5.25/Java 8; 3.11.49 e 4.0.43/Java 17; 6.0.54 e 7.0.63/Java 21; 9.0.75 e 10.0.77/Java 25. Cada versão passou com slim/capa e clássica/sem capa; o teste verifica o modelo, a textura entregue por HTTP byte a byte e que o adaptador não altera outro UUID.

Esses testes carregam e instrumentam as classes reais da Authlib, mas não executam uma partida completa nem todos os mods que podem substituir o renderer. A aparência é local ao cliente; sua publicação para outros jogadores continua dependente dos mecanismos oficiais ou de mods compatíveis.

Referências do adaptador: [Java Instrumentation](https://docs.oracle.com/javase/8/docs/api/java/lang/instrument/ClassFileTransformer.html), [ASM](https://asm.ow2.io/) e [licença ASM](https://asm.ow2.io/license.html).

O smoke nativo encontrou um atalho preexistente que ignorava a integridade quando havia cinco JARs na pasta. Esse atalho foi removido: o reparo volta a verificar cada entrada do manifesto, e falhas de recuperação CurseForge deixam de ser ignoradas. Um teste de regressão usa seis JARs alheios e confirma que eles não satisfazem um arquivo obrigatório ausente.

A mesma auditoria removeu a desativação/reativação automática de JARs por prefixo de nome e o uso de um override com nome semelhante para satisfazer outro arquivo do manifesto. Apenas um override no caminho exato substitui a entrada correspondente. O lançamento propaga erros de integridade do modpack e de validação/download dos arquivos Minecraft, sem considerar a simples existência do client JAR como prova de validade.

Smoke nativo após as correções: aprovado usando o daemon compilado e diretórios temporários. Cobriu nomes com tentativa de traversal, IDs/pastas distintos, importação Modrinth com 51 arquivos verificados por hash, recuperação de JAR de zero bytes, preservação de JAR desativado, importação CurseForge com 46 entradas e 48 JARs, caminhos Windows/raiz aninhada, precedência de client-overrides, cancelamento e bloqueio de importação concorrente. Esse comando foi executado sem `--launch`, portanto não é uma partida completa.

Teste adicional de inicialização: `pnpm test:native --launch` passou, sem variáveis adicionais de pilha, com Fabulously Optimized 1.20.1 importado de Modrinth e CurseForge. As duas JVMs inicializaram renderer/recursos, foram observadas por 45 segundos cada e encerradas pelo teste. A abertura dos recursos gráficos não comprova uma partida completa nem a aparência vista por outro jogador.

A investigação do estouro de pilha levou a uma correção em `main.rs`: os workers recebem uma reserva de pilha de 16 MiB, e os comandos Tauri compartilham o runtime configurado. A reserva por thread não corresponde a 16 MiB de páginas residentes imediatamente; evita o limite padrão insuficiente observado durante o lançamento no build debug.

Build de produção aprovado: `pnpm tauri build --no-bundle`, seguido de `cargo build --release --locked --bin luxmc --features tauri/custom-protocol` para incluir a correção final do runtime sem regenerar o frontend. Executável Linux: `src-tauri/target/release/luxmc`. O build foi feito com Rust 1.98.1; a dependência iroh 1.2.0 exige Rust 1.91 ou superior.

Checagem final do executável release: `LUXMC_NATIVE_BINARY=src-tauri/target/release/luxmc pnpm test:native` aprovada, repetindo os seis cenários de integridade/importação/cancelamento diretamente no binário otimizado final.
