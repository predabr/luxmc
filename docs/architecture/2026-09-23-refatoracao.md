# Refatoração Luxmc — 23/09/2026

## Entrega e limites

As alterações abrangem os oito pilares do pedido, mas **não equivalem à certificação de todos eles em produção**. O launcher usa Tailscale instalado e autenticado externamente; não instala uma VPN própria nem provisiona participantes pelo código da sala. O serviço WebSocket está implementado e compilado, mas ainda precisa de implantação. Não foi comprovada a eliminação definitiva do travamento nativo WebKitGTK/GStreamer relatado após dois minutos.

## Memória, vídeo e IPC

- `VideoWallpaper.svelte`: um elemento de vídeo, loop nativo, `object-fit: cover`, fallback estático, pausa com jogo/performance/aba oculta e liberação da fonte no descarte.
- O wallpaper anterior carregava/reiniciava vídeo por reatividade. O CSP também bloqueava o servidor local de mídia; foi ajustado.
- `SkinViewer3D.svelte`: importação assíncrona, placeholder 2D, renderizador limitado a 30 FPS, IntersectionObserver, pausa ao ocultar/jogar e cancelamento de RAF. Dispose de recursos gráficos e perda explícita do contexto no desmontar.
- `LiveWallpaper.svelte`: cancelamento do RAF do mouse e limite de quadros.
- Corrigido acúmulo de timers de toast: a chave removida não correspondia à chave registrada. Remoção também ao descartar notificações antigas.
- Logs do processo: linhas limitadas a 2 KiB sem quebrar UTF-8, buffer limitado e lotes IPC. Downloads limitam eventos intermediários a um a cada 250 ms.
- Servidor de mídia: limite de oito conexões, timeout, `Cache-Control: no-store`, streaming em blocos e interpretação de Range com respostas 416 válidas. Chat LAN limita conexões e leitura antes de alocar a mensagem inteira.

Essas correções removem mecanismos concretos de retenção e excesso de trabalho. O teste Chromium mede heap JavaScript, não RSS do launcher, memória GPU ou pipelines GStreamer. Pausar um vídeo reduz trabalho; não representa garantia de 100% da GPU livre.

## Skins e capas

`auth_save_appearance` grava skin, modelo e capa em um único UPDATE SQLite. O frontend atualiza a seleção/localStorage e serializa as gravações. Falhas aparecem na interface; SQLite e localStorage não formam uma transação distribuída. A seleção local não chama a API oficial da Microsoft. A sincronização remota tem ação explícita.

Carregamento de imagens tem cancelamento, timeout e limite de dimensões. Capas OptiFine proporcionais, quadradas e 2:1 são normalizadas para a prévia. Isso corrige formatos/aspectos suportados; não reconstrói automaticamente pixels ausentes de uma textura arbitrária. A persistência foi exercitada com IPC simulado; a aparência dentro de um jogo real permanece um teste necessário.

## Java, Forge, NeoForge e downloads

O pipeline existente de seleção de Java/Fabric/Quilt foi preservado. Flags geracionais explícitas são usadas somente no Java 21/22; versões posteriores não recebem a opção removida. Downloads usam arquivo temporário único, tamanho/hash, retry, remoção de parciais e validação do cache.

Foi reproduzida falha real de SHA do processador Forge 1.20.1: o Java Microsoft instalado vinculava libzip à zlib-ng do sistema. Remover os checksums dos processadores mascarava o problema. A nova implementação mantém a verificação, exige sucesso do instalador e utiliza um JRE Temurin gerenciado no Linux para os processadores. Download vem do repositório oficial Adoptium no GitHub, com SHA-256, limite de tamanho e extração em staging. Esse runtime fica no diretório de dados do aplicativo; não altera Java ou bibliotecas do sistema.

Evidência upstream: [Forge Installer #80](https://github.com/MinecraftForge/Installer/issues/80) e [Forge #10777](https://github.com/MinecraftForge/MinecraftForge/issues/10777).

Instaladores têm timeout, log persistente e marcador de conclusão. Bibliotecas ausentes/corrompidas não são silenciosamente ignoradas. Integrações reais passaram para Forge 1.20.1, NeoForge 1.20.4 e NeoForge 1.21.1, incluindo processadores e JARs gerados. Isso não significa que todos os modpacks existentes foram executados.

Crash Doctor oferece aplicação de RAM e instalação/seleção de Java compatível quando diagnosticado. Conflitos arbitrários de mods e drivers continuam exigindo a orientação exibida; não há conserto universal seguro em um clique.

## Rede mesh e social

- Adaptador Rust para Tailscale: status, IPs virtuais, peers e ping real; sem valores simulados no produto.
- Tailscale fornece transporte direto ou relay DERP conforme conectividade. DERP não é TURN. [Documentação de conexões](https://tailscale.com/docs/reference/connection-types).
- Sala social com seis dígitos, expiração de uma hora, controle de amizade/bloqueio e limitação de requisições.
- Código identifica o endereço do mundo compartilhado. **Não concede acesso ao tailnet**. Os dois computadores precisam ter Tailscale instalado, conectado à mesma rede privada e ACL permitindo o tráfego.
- Entrada direta no mundo usa IP/porta publicados. Não foi implementado encaminhamento multicast para a lista LAN nativa do Minecraft. Também não foi validado NAT restrito entre duas máquinas físicas.
- Bloquear/remove amizade é acessível pela interface; desbloqueio existe na API, ainda sem tela de gerenciamento de bloqueados. Favoritos e solicitações existentes foram preservados.
- Toast de mundo aberto permite entrar. Adicionar nome inexistente não produz mais sucesso local fictício.

### WebSocket

`services/social-stream` contém Worker e Durable Object com sockets hibernáveis. O Pages emite ticket de uso único, validade de 30 segundos, armazenado somente como SHA-256 no D1. O worker consome esse ticket e envia apenas a lista autorizada do usuário, com limite de conexões. O cliente valida mensagens, fecha ao desconectar e reconecta com backoff.

O Durable Object compara snapshots a cada cinco segundos enquanto há sockets; o heartbeat REST mantém presença e fallback. Assim, não é um broadcast instantâneo de cada escrita. Sem `SOCIAL_STREAM_URL`, a presença continua pelo REST existente.

## Interface e site

Sidebar recolhível com preferência persistida, navegação Amigos & Rede, tokens semânticos nas novas superfícies, glass/contraste e progresso no botão Jogar. Funcionalidades e rotas existentes foram mantidas.

Landing page com hero dividido, mockup com interação, showcase, downloads por SO/distro e assets do release GitHub. Benchmark mostra campos a medir e metodologia, sem números de desempenho inventados. Depoimentos não verificáveis foram substituídos por chamada para feedback real. Imagens em `docs/visual/2026-09-23` são evidências de teste; contas, skins remotas e releases foram simulados nos testes de navegador.

## Ativação externa

Não foram publicados site, Worker, migrations ou releases. Para ativar o social novo na conta Cloudflare configurada:

1. Aplicar as migrations pendentes, inclusive `0003_mesh_social.sql` e `0004_social_stream.sql`, no D1 usado por Pages e Worker:
   `pnpm dlx wrangler@4.135.0 --cwd website d1 migrations apply luxmc-social --remote`
2. Publicar o worker:
   `pnpm dlx wrangler@4.135.0 deploy --config services/social-stream/wrangler.toml`
3. Configurar `SOCIAL_STREAM_URL` no Pages com o endereço WSS `/connect` retornado para esse Worker. Confirmar que ambos usam o mesmo D1.
4. Publicar Pages: `pnpm deploy:website`.
5. Instalar/autenticar Tailscale em ambos os computadores na mesma rede privada, permitir a porta LAN pelas ACL/firewalls, abrir o mundo para LAN e compartilhá-lo no painel.

Ainda necessários: sessão prolongada do binário nativo com wallpaper real, RSS/GPU/GStreamer, autenticação Microsoft real, gameplay dos modpacks alvo, conexão entre redes físicas e builds/testes Windows/macOS. Benchmarks e reviews precisam de coleta real antes de publicação como evidência.

## Validação local

- `pnpm check`: zero erros e avisos.
- `pnpm build`: aprovado.
- `pnpm test:run`: 50 testes aprovados.
- `pnpm test:website`: 15 testes aprovados, incluindo bloqueios, salas e tickets.
- `cargo check`: aprovado.
- `cargo test --lib`: 55 testes aprovados, incluindo logs Unicode extensos, transferências incompletas, Range e ping relay.
- Integrações reais Forge/NeoForge: três cenários aprovados.
- `pnpm electron:compile`: aprovado.
- Worker: `wrangler deploy --dry-run` aprovado, sem publicar.
- Fluxos `luxmc://` em Chromium com IPC simulado: aprovados, sem erros JavaScript.
- Site em Chromium: botão macOS correspondeu ao `.dmg` do release simulado, sem erros JavaScript.

O teste `tests/stability.browser.cjs` exercita oito ciclos Skins/Amigos, aguarda 125 segundos e compara heap após GC, com tolerância de 20 MiB. Esse limite é uma regressão automatizada, não uma prova matemática de ausência de leaks.

Resultado final do teste prolongado: heap após GC de 23.381.324 para 22.929.292 bytes; nenhum erro JavaScript. O modelo 3D foi aguardado visível a cada retorno à página. Não houve crescimento de heap nessa execução. Site móvel verificado em 390 px, com largura do documento também de 390 px.

Compilação nativa: `pnpm tauri build --no-bundle` aprovada em perfil release otimizado. Executável gerado em `src-tauri/target/release/luxmc`. `git diff --check` aprovado. Nenhum deploy, commit ou alteração de configuração do sistema foi realizado; o arquivo preexistente `snake.html` foi preservado.
