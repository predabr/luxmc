# Revisão de skins, desempenho e interface — 26/09/2026

## Alterações

- Importação de skins e capas lê PNG pelo backend, com limite de 2 MB e dimensões de até 2048 pixels. O canvas recebe uma URL de dados, evitando a origem externa do protocolo de arquivos. A busca de skin por nickname usa o resolvedor de textura antes da prévia. Aplicação explícita preservada.
- Wallpaper usa, por padrão, superfície de até 1280 pixels e desenho a 24 FPS. Configurações oferecem 960/1280/1920 pixels, 15/24/30 FPS, pausa fora de foco e desfoque opcional sobre vídeo. Os valores são salvos pela persistência existente.
- Quando FFmpeg está disponível, o backend prepara uma cópia H.264 otimizada, sem áudio, preservando o original. A chave do cache inclui caminho, tamanho, modificação e qualidade. Conversões são serializadas, limitadas a 120 segundos e arquivos de entrada de até 512 MB. Sem conversão disponível, permanece o vídeo original.
- Prévia compartilha a imagem estática em cache; não mantém um segundo vídeo decodificando quando o poster existe. Painéis sobre wallpaper animado deixam de recalcular desfoque por padrão.
- Notícias laterais exibem o feed carregado inteiro, limitado pelo store a 40 artigos, com rolagem e imagens lazy.
- Configurações da instância têm cabeçalho com nome, arte, versão, fechamento acessível e área maior. Ícone do perfil substitui a imagem fixa. Botões de ordenação, modo de exibição e paginação do explorador usam os controles compartilhados.
- Compartilhar mundo no cabeçalho abre o painel de convite criptografado. O fluxo de endereço direto permanece acessível. A detecção automática de porta do túnel filtra anúncios LAN de outras máquinas, pois o túnel conecta ao Minecraft local.
- Verificação de arquivos do modpack passa por workers bloqueantes. No Linux, hashes já confirmados são reaproveitados em memória enquanto dispositivo, inode, tamanho, mtime, ctime e hash esperado permanecem iguais. Outros sistemas continuam verificando o conteúdo. Não se confia apenas no tamanho, nem se removem JARs por nome. O log informa o tempo da fase de integridade.
- Site recebeu revisão de hierarquia, espaçamento, botões e responsividade. Métricas de RAM/abertura sem evidência foram retiradas. Dois scripts de anúncios sobrepostos foram removidos; os espaços de publicidade na página continuam.

## Verificações

- `pnpm check`: zero erros e avisos.
- `pnpm build`: compilação de produção do frontend concluída.
- `pnpm tauri build --no-bundle`: executável final compilado em 8 min 06 s e aberto para teste local.
- `pnpm test:run`: 59 testes aprovados.
- `pnpm test:website`: 15 testes aprovados.
- `cargo check --locked`: aprovado.
- `cargo test --locked --lib`: 68 aprovados; teste de relay público executado separadamente.
- Relay público: transferência forçada sem transporte IP direto aprovada. O teste local também verifica convites inválidos, transferência concorrente e encerramento.
- WebKitGTK: teste de 150 segundos aprovado, cerca de 22,7 FPS médios, 20 ciclos de vídeo, recuperação de pausa, pausa/retomada durante jogo e prévia sem segundo vídeo. O teste utiliza IPC simulado; não substitui medição do aplicativo instalado.
- Comando nativo de leitura PNG testado no executável de produção: bytes idênticos ao arquivo original.
- Comando nativo de conversão validado: 1280×720 a 24 FPS; primeira preparação em 10,64 s durante a compilação, chamada seguinte em 0,00016 s sem reescrever o cache. PNG inválido rejeitado pelo comando nativo.
- WebKitGTK: 18 cenários de interface aprovados, incluindo conservação da qualidade ao reabrir ajustes e banner editável no modal da instância.
- Reparação nativa em perfis isolados: Prominence, 439 JARs preservados, 4,42 s na primeira verificação e 0,014 s na seguinte; All the Mods, 491 JARs preservados, 3,63 s e 0,012 s. São tempos da etapa de reparação, não do carregamento completo do jogo.

Evidências desta revisão ficam em `/home/pedro/.cache/luxmc-validation/2026-09-26-revision/`. Logs de compilação, testes e publicação foram copiados de `/tmp/luxmc-current-*` para esse diretório.

## Medição no aplicativo de produção

Tela inicial com wallpaper animado, sem compilação ativa: 42,78% de um núcleo em 30 segundos; 616,93 MiB de RSS somados em três processos (memória compartilhada pode ser contada mais de uma vez). Não é uma promessa de consumo em outros PCs. No teste WebKit com skin 3D e vídeo otimizado, foram observados 43,96% de um núcleo e intervalo entre quadros p95 de 18 ms. O teste prolongado da cópia otimizada passou com recuperação, pausa/retomada e prévia em cache.

## Inicialização real do Minecraft

Prominence II chegou ao menu, confirmado visualmente em perfil isolado de validação. A preparação pelo launcher levou 7,83 s; o log do jogo registrou `Game took 84.552 seconds to start`. São etapas distintas. A janela de teste foi encerrada sem abrir mundos, e o launcher do usuário permaneceu aberto. All the Mods teve sua reparação validada nesta revisão, mas não recebeu uma nova confirmação visual de menu.

## Site

Publicado no Cloudflare Pages sem push para o GitHub. URL oficial: https://luxmc-r92.pages.dev. Deployment final com demonstração: https://7406f4f1.luxmc-r92.pages.dev.

## Limites e trabalho dependente de validação adicional

Foi gravada uma prévia real de 18 segundos do personalizador com skin 3D e wallpaper animado, publicada em `website/assets/launcher-personalizer.mp4`. O vídeo completo iniciando Minecraft não foi produzido. Não foi feita uma partida entre dois computadores/redes físicas; os testes validam o transporte criptografado local e via relay público. Windows não foi testado nesta revisão.

Os logs existentes mostram aproximadamente 38 segundos do início do DawnCraft até iniciar o áudio e 65 segundos no Homestead. Esses marcos não comprovam menu pronto. DawnCraft também apresenta grande volume de erros OpenGL; nenhum mod ou ajuste gráfico do usuário foi removido para ocultar esse problema. Não há medição que permita prometer redução do tempo total de carregamento de todos os packs.
