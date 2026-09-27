# Correções após teste real no Linux

**Revisão concluída em 26/09:** o relato de erro levou à reprodução de falhas reais no Prominence e no wallpaper. As seções finais registram as correções e a confirmação visual dos dois menus. Os testes anteriores que encerravam a JVM após carregar texturas foram substituídos; aqueles sinais isolados não comprovavam chegada ao menu.

Esta revisão substitui o diagnóstico de orientação das capas e a estratégia de vídeo do relatório de 23/09. Os testes anteriores no Chromium não reproduziam todos os problemas do WebKitGTK.

## Wallpaper

O vídeo funciona como fonte de decodificação. A superfície visível é um canvas persistente, atualizado no máximo cerca de 30 vezes por segundo e limitado a 1920 pixels de largura. Um canvas intermediário verifica se o frame pode ser lido antes de substituir o último frame válido. Durante seek, espera de dados ou frame inválido no loop, a imagem anterior permanece. A pausa do jogo/visibilidade conserva a imagem. Todos os callbacks e fontes de mídia são descartados ao desmontar o componente.

A miniatura usa proporção 16:9 e um cache limitado a seis frames, com resolução máxima de 480 pixels. O componente compartilha a resolução do endereço de mídia com o wallpaper principal. Selecionar um fundo padrão agora desativa a exibição do vídeo importado, preservando o arquivo para uma futura seleção.

Arquivos: `src/lib/components/visuals/VideoWallpaper.svelte`, `src/lib/utils/wallpaperSource.ts`, `src/lib/components/settings/ThemeSection.svelte`, `src/routes/+layout.svelte`.

## Skins e capas

A prévia usa preserveDrawingBuffer para conservar a renderização WebGL no compositor. O fallback 2D tenta a textura escolhida antes da textura local. As seleções são um rascunho; apenas **Aplicar skin e capa** envia a aparência para o backend e atualiza o avatar aplicado. Falhar ao salvar não muda a aparência da conta em memória. Sincronização Microsoft é uma ação separada.

O modelo de capa é girado 180 graus: apesar dos nomes front/back da geometria, a face externa após essa rotação usa a região x=1..10, y=1..16. O gerador colocava o desenho em x=12..21, voltado para o corpo. As faces das capas geradas foram corrigidas. A migração reconhece por igualdade de pixels as duas variantes antigas (incluindo a espelhada). PNGs importados que não correspondem exatamente às capas antigas permanecem intactos.

Referência auditada: `node_modules/skinview3d/libs/model.js`, `setUVs`, `CapeObject` e a rotação em `PlayerObject`.

## Interface e instâncias

O painel social recebeu margens, cantos arredondados e contorno contínuo. O modal P2P da instância agora reutiliza os dois cards do túnel QUIC; entrar inicia a instância dessa página. A introdução usa uma composição simples com o logotipo, transições curtas e suporte a movimento reduzido, sem porcentagens artificiais ou chamadas duplicadas de inicialização.

## Modpacks grandes

O download de cada arquivo tinha um limite total de 15 segundos. Agora há limites separados de conexão (15 s), inatividade de leitura (45 s) e transferência total (10 min). O monitor de cancelamento continua ativo. A concorrência de arquivos foi reduzida para seis para limitar a pressão de memória de arquivos grandes. Hashes, arquivos desativados, manifestos e overrides continuam preservados.

Dois testes Rust adicionais cobrem uma transferência que entrega o restante após 16 segundos e o cancelamento de uma resposta parada.

## Verificação

Os resultados finais e limitações dos testes são registrados abaixo ao término da execução. O teste WebKitGTK usa a skin escolhida pelo usuário e seu vídeo local de 15,85 segundos; o IPC da interface é simulado. A importação de packs usa o backend Rust real e diretórios isolados; a revisão de 25/09 conserva as evidências em `~/.cache/luxmc-validation/2026-09-25/`.


Resultados de interface:

- `pnpm check`: zero erros e avisos; 52 testes de frontend aprovados.
- Rust: 63 testes aprovados, um teste ignorado pelo projeto. `cargo check --locked` aprovado.
- WebKitGTK: 75 segundos com o vídeo real, quatro loops, 13 amostras da skin visível e dez frames diferentes. Nenhuma aplicação automática da skin.
- Chromium: três loops; canvas preservado durante seek/espera; pausa/retomada; escolha de fundo padrão e retorno ao customizado; miniatura preenchida com o vídeo pausado.
- Auditoria da interface: salvamento somente após Aplicar; mesma instância do canvas; recuperação WebGL; migração das duas faces da capa; painel P2P com dois cards.
- All the Mods 10 8.2: importação real CurseForge e reparo aprovados, 491 JARs. A inicialização gráfica também foi confirmada no teste separado descrito abaixo.

Prominence II 4.1.0: importação real Modrinth e reparo aprovados, 435 JARs. All the Mods 10 iniciou a JVM, permaneceu em execução e inicializou o renderizador e os atlas de texturas. A JVM de teste foi encerrada pelo próprio teste após a confirmação. Não foi testada criação de mundo nem uma sessão longa.

Prominence II também iniciou com Fabric 0.19.3 / Minecraft 1.20.1 e inicializou renderizador/recursos, permanecendo em execução durante a observação. O teste encerrou sua JVM após confirmar a inicialização. Os dois packs foram testados em diretórios temporários, sem alterar as instâncias do usuário.


## Revisão de 25/09: confirmação visual

O teste anterior encerrava a JVM com SIGTERM após mensagens de renderização. Esse encerramento pode acionar o CrashAssistant incluído pelos packs; a mensagem original do usuário não foi preservada, portanto a causa daquele aviso permanece uma hipótese.

O teste de inicialização agora exige uma captura do menu confirmada visualmente, criada durante a execução atual. Depois solicita fechamento normal da janela. Encerramento forçado não conta como aprovação. A saída do teste registra separadamente a confirmação do menu e o fechamento normal.

- All the Mods 10 8.2 / NeoForge 21.1.251 / Minecraft 1.21.1: importação e reparo aprovados, 491 JARs preservados. Menu principal confirmado em captura, exibindo 539 mods carregados. Log do ModernFix: 105,9 segundos para iniciar. Fechamento normal confirmado por `Stopping!` e término da JVM; teste saiu com código zero. O comando de fechamento foi adaptado à API Lua do Hyprland desta máquina.
- `pnpm check`: zero erros e avisos. `git diff --check`: aprovado.

Evidências locais: `~/.cache/luxmc-validation/2026-09-25/atm-current.png`, `atm-launch.log` e `heavy-packs-8dyzey/launch-9eae3b0f-590c-4b32-8b94-e82507985dcb.json`.


### Falhas reproduzidas na nova execução

A primeira execução completa do Prominence II 4.1.0 falhou de verdade: `Default font failed to load`, seguido de `Rendering overlay`. O mod Prominent referencia `prominent:textures/gui/realms.png`, fornecido por `resourcepacks/ProminenceFancyServerListing.zip`. Esse ZIP estava íntegro, mas não estava ativado. A injeção de aparência criava `options.txt` com a lista padrão antes do YOSBR aplicar `config/yosbr/options.txt`, impedindo a configuração de recursos fornecida pelo pack. A inicialização de aparência agora usa esse arquivo de defaults quando ainda não existem opções pessoais. Dois testes aprovados cobrem os defaults na primeira execução, a preservação das opções pessoais nas seguintes e a limpeza de capas antigas.

Na versão Tauri de produção, o caminho DMA-BUF do WebKit apresentou vídeo com ruído colorido. Um teste isolado reproduziu isso tanto em canvas comum quanto com `willReadFrequently`. Desabilitar DMA-BUF corrigiu ambos sem `LIBGL_ALWAYS_SOFTWARE`. Esse teste isolou o problema, mas não foi a configuração final: desabilitar apenas o sink DMA-BUF do GStreamer permite manter a aceleração da interface. O launcher também autoriza a reprodução do elemento de vídeo invisível usado como fonte do canvas, evitando a suspensão automática pelo WebKit. Ambos os ajustes respeitam valores explícitos do ambiente.

O wallpaper ganhou uma verificação periódica de progresso: retoma pausas inesperadas e recarrega o decodificador se o tempo do vídeo ficar parado por dez segundos. A superfície visível mantém o último quadro. Pausas intencionais por jogo, modo de desempenho, prévia e janela oculta continuam respeitadas; o intervalo é removido ao desmontar.


### Continuação em 26/09

O Prominence do Modrinth também exibe uma tela própria informando que FTB Quests 2001.4.13, FTB Teams 2001.3.1, FTB Library 2001.2.9 e FTB XMod Compat 2.1.3 não estão incluídos nessa distribuição. Os links oficiais estão no `config/fancymenu/customization/noftbquests.txt` publicado no pack. O reparo agora reconhece esses quatro projetos FTB no Prominence e usa o fluxo CurseForge existente para baixar/verificar exatamente os arquivos declarados. Versões diferentes já instaladas pelo usuário permanecem preservadas. A instância de teste passou de 435 para 439 JARs, sem remoção dos originais.

O loop nativo do vídeo substitui a repetição manual como caminho normal; a superfície persistente ainda oculta quadros transitórios. A experiência de converter o arquivo inteiro para Blob foi descartada: a solução final mantém streaming e não retém uma cópia integral do vídeo em JavaScript.

Os testes agora verificam a assinatura do quadro completo, ruído visual e atividade nos últimos trinta segundos, além da retomada após pausa inesperada. Isso evita aprovar um vídeo cujo relógio continua avançando com a imagem congelada.

Validações de código: 65 testes Rust aprovados, um ignorado pelo projeto; `cargo check --locked` aprovado; `pnpm check` sem erros ou avisos. Resultados visuais finais registrados a seguir após conclusão.


Wallpaper validado em 26/09 com o arquivo real do usuário: teste WebKitGTK de 180 segundos aprovado, 12 loops, 34 amostras válidas e 34 assinaturas distintas de quadros completos. Nenhum ruído detectado; retomada após pausa inesperada aprovada; skin visível e nenhuma aplicação automática de aparência. Log: `~/.cache/luxmc-validation/2026-09-25/webkit-delivery.log`.


### Resultado final em 26/09

- `pnpm tauri build --no-bundle`: aprovado, executável final aberto no desktop do usuário.
- Wallpaper atualmente selecionado pelo usuário (`vodyanitsa-eye-genshin-impact-moewalls-com.mp4`): 180 segundos de WebKitGTK usando o servidor de mídia do launcher final, 25 loops, 34 amostras válidas e 30 quadros distintos; retomada após pausa inesperada, sem ruído e sem congelamento detectado. Log: `webkit-current-wallpaper.log`.
- Prominence II 4.1.0: menu completo confirmado visualmente após os quatro complementos FTB, sem bloqueio por dependências; encerramento normal confirmado, teste com código zero. Evidências: `prominence-final-menu.png`, `prominence-delivery.log` e `heavy-packs-8dyzey/launch-bf00d59f-8a28-4e16-920c-475091c95305.json`.
- All the Mods 10 8.2: menu e encerramento normal confirmados anteriormente nesta revisão.

As evidências estão em `~/.cache/luxmc-validation/2026-09-25/`. Os testes dos packs utilizaram instâncias isoladas e conta offline. Criação de mundos, sessões longas e multiplayer não foram validados nesta revisão. As instâncias e preferências pessoais do usuário foram preservadas.
