# Revisão do design e abertura do Minecraft

Os botões compartilhados receberam variantes próprias para ações principais, secundárias, Microsoft, Jogar e exclusão. As áreas de clique aumentaram para 40–56 px, com foco visível e estados de carregamento. O wallpaper continua governando o fundo dos controles pela opacidade configurada; texto e ícones mantêm opacidade integral. Animações curtas de entrada e interação respeitam redução de movimento e o desligamento de animações.

A barra lateral expandida tem 240 px e exibe nomes e versões das instâncias, portal, GitHub e conta em linhas completas. A biblioteca usa colunas calculadas pela largura disponível. Em janelas menores, o painel secundário deixa espaço para a biblioteca; Amigos permanece na navegação principal. O atalho Jogar da biblioteca fica sempre visível.

Amigos agora apresenta uma única ação principal de adicionar jogador no cabeçalho, orientações concretas para nome/código e seleção identificada nas abas. O personalizador removeu o catálogo de capas prontas e preserva a capa atual, da conta ou importada. O NameMC abre uma janela isolada e captura a seleção para importar a textura à coleção. Verificações do próprio NameMC podem exigir interação nessa janela; há alternativa por link ou arquivo PNG.

Os editores de controles/configuração e o Jukebox passaram para Ferramentas na instância. Configurações recebeu navegação por teclado, resolução personalizada persistente e efeitos de som com volume e prévia. Opções sem consumidor real foram removidas. Os presets Java oferecidos são Automático e G1GC; argumentos personalizados continuam disponíveis.

O início do jogo reaproveita a extração nativa válida e os perfis Quilt instalados, respeita arquitetura das DLLs e mantém Xms dentro do limite Xmx. As otimizações preservam mundos e arquivos de mods. Detalhes e limites da medição em [startup-performance-2026-10-04.md](startup-performance-2026-10-04.md).

O instalador mantém a cena 3D do portal e mostra passos claros de conta, versão/modpack e início, com explicação da preparação inicial e preservação de dados nas atualizações. O nome de distribuição continua `Lux MC Launcher.exe`.

## Verificação

- `pnpm check`: zero erros e avisos.
- Vitest: 121 testes passaram.
- `cargo check`: passou, com quatro avisos preexistentes.
- Rust: 113 passaram, um ignorado, nenhuma falha.
- NameMC, aplicação Microsoft e importação/animação WebGL de skin: passaram.
- Importação real de PNG do NameMC pelo cliente Rust: passou após identificar as requisições com `Luxmc/<versão>`; o endpoint rejeitava o cliente sem identificação. O teste online foi executado separadamente da suíte local.
- Configurações em 1600/960, conteúdo de instâncias com 1000 mods, sete cenários de idioma, atualizador e diagnóstico Windows: passaram.
- P2P convidado e pedidos de amizade: passaram nas simulações de interface. Conexão real entre dois computadores precisa de teste entre os jogadores.
- Wallpaper estático/animado, blur ligado/desligado e modo de desempenho: cinco cenários passaram, oito painéis em cada.
- Layout em 1600/960, sidebar sem overflow, uma ação de adicionar amigo, transparência e bloqueio de login Microsoft concorrente: passaram.
- Troca de contas: Microsoft fecha após sucesso e preserva textura/modelo/capa com expiração normalizada; Lux mantém sua própria skin slim e limpa a capa anterior. Concorrência entre provedores, fechamento durante login e envio duplicado foram verificados.

Entrega, instalação e verificações nativas ficam registradas em `redesign-delivery.json` e `redesign-native-verify.json` após geração da release.

## Entrega final — 5 de outubro de 2026

A compilação Windows terminou com sucesso. O instalador atualizado foi copiado para `C:\Users\preda\Downloads\Lux MC Launcher.exe` e instalado, com saída zero. Tem 20.081.242 bytes; o hash coincide com o pacote gerado e está registrado em `redesign-delivery.json`. Os hashes de banco, contas e configurações permaneceram iguais durante a instalação.

O executável instalado inicializou normalmente, identificou `pt-BR`, carregou a conta e as duas instâncias existentes, importou a textura real do NameMC em PNG 64 × 64 e passou em `PRAGMA integrity_check`. O subsistema Windows do executável é GUI. A janela `Luxmc` foi reaberta e confirmou `Responding: true`; o resultado está em `redesign-open-verify.json`.

O instalador e o aplicativo continuam `NotSigned`. Nenhuma assinatura confiável foi gerada ou solicitada nesta entrega. As verificações de interface usaram automação de navegador e simulações dos comandos nativos; a abertura e os comandos acima foram conferidos no executável instalado. O teste de P2P entre dois computadores continua dependendo dos jogadores.

As opções gratuitas de assinatura e as restrições da licença atual estão em [assinatura-windows.md](../assinatura-windows.md).
