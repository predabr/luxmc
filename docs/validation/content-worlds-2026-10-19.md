# Retomada: capas, mundos e fluidez — 19/10/2026

O usuário precisou desligar o PC. Compilação e testes em andamento foram interrompidos; nenhuma instalação ou publicação desta revisão foi realizada.

## Implementado

- Recuperação de imagens remotas que falham: leitura nativa dos CDNs autorizados, limites de rede/decodificação, miniatura PNG local persistida e reutilizada no banco.
- Sincronização da pré-carga de capas com a lista já aberta, inclusive quando a linha tinha um resultado negativo antigo. Respostas negativas atrasadas não removem uma imagem já encontrada.
- Atualização das capas por chave, alterando apenas os itens correspondentes; evita recriar todas as linhas e recalcular metadados em cada lote.
- Carregamento da seção selecionada sem depender de IntersectionObserver. Mundos atualizam ao entrar na aba, recuperar foco e voltar do Minecraft; solicitações concorrentes são compartilhadas. Indicador de carregamento antes de mostrar estado vazio.
- Abertura da aba Mods não faz mais varredura de mundos/datapacks em segundo plano.
- Animações leves na chegada das capas e nas interações com campos, controles e cartões; respeitam redução de movimento e desempenho.

## Verificado

- pnpm check: zero erros/avisos.
- cargo check: passou; seis avisos anteriores de código não usado.
- Vitest: 36 arquivos, 219 testes passaram.
- Rust: dois testes de content_icons passaram, incluindo limites de URL/decodificação e redimensionamento.
- Teste novo content-world-refresh.browser.cjs passou em 1440x900 e 1440x600: primeira capa com falha, imagem que chega pela pré-carga, lista de mil mods, retorno à linha virtualizada, mundos após estado vazio/foco/saída do jogo.
- Testes focados instance-content, scroll-profile-polish, wallpaper-restart, efficient-motion e friends-visible-profiles passaram.
- Leitura direta do executável instalado encontrou oito perfis e quatro mundos em saves: Vanilla 26.3/New World, Fabulously Optimized/Novo mundo, Pixelmon/New World e Vanilla Perfected/New World. Não mover nem recriar esses mundos.
- AccuracyCrosshair.zip pertence a Immersed With Shaders (3d4da23d-4932-4d9f-ad9c-7d6f0e2ccc7a). O cache tinha URL válida do CurseForge; a imagem real respondeu HTTP 200 e 256x256. Não é necessário fabricar capa nem alterar o mod.
- Um teste extra antigo social-download-polish falhou no seletor “Salvar”; o editor atual usa “Salvar perfil”. Ele não integra a suíte padrão. O teste atual friends-visible-profiles passou. Não apresentar isso como falha nova do produto.

## Ainda necessário

1. Retomar a compilação release e aguardar conclusão; não instalar binário antigo/resultado parcial.
2. Validar o frontend compilado com o backend nativo e dados reais usando o script privado native-ui.cjs. Ele verifica a miniatura real de AccuracyCrosshair e os quatro mundos, sem autenticar nem modificar contas.
3. Terminar a suíte padrão de navegador a partir do teste interrompido ou repetir somente o necessário. Ela inclui agora o novo teste.
4. Empacotar com pnpm tauri bundle --bundles nsis, sem repetir a compilação do frontend.
5. Conferir ausência de Java ativo, salvar backup fresco, instalar com /S /UPDATE, verificar executável e preservação dos dados e abrir para o usuário testar.
6. Registrar resultado real e hashes. Não publicar no GitHub/site sem nova orientação nesta retomada.

## Ambiente e arquivos de apoio

- Node: C:/Users/preda/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node.exe
- pnpm: C:/Users/preda/AppData/Local/Luxmc-build/tools/pnpm-10.30.3/package/bin/pnpm.cjs
- Cargo: C:/Users/preda/.cargo/bin/cargo.exe
- CARGO_TARGET_DIR=C:/Users/preda/AppData/Local/Luxmc-build/target
- LUXMC_BUILD_ID=38781690526.4 (revisão local planejada; a instalada continua 38781690526.3).
- Compilar: pnpm tauri build --no-bundle, com Node/Cargo no PATH. A compilação completa pode levar cerca de 15 minutos; não interromper por mera demora.
- Scripts privados e logs: C:/Users/preda/AppData/Local/Luxmc-build/requests/accuracy-crosshair/
- native-ui.cjs: iniciar preview do frontend compilado na porta 1430; definir LUXMC_CHROMIUM_EXECUTABLE=C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe; executar com Node depois que o executável novo estiver pronto.
- install-local.ps1 e capture-data.py: preparados, ainda não executados. Fazem backup de banco, JSONs de configuração, mundos e executável anterior; verificam perfis, arquivos dos mundos, mods e hashes de configuração antes/depois. Backup planejado em C:/Users/preda/AppData/Local/Luxmc-build/backups/content-worlds-20261019.
- O verificador NSS/UNK do instalador está em requests/scroll-profile-20261019/verify-installed.py e aceita somente os três bytes contíguos esperados do marcador Tauri.
- Preservar todas as alterações antigas não relacionadas e tests/archive-io-reference.py. Não fazer git add indiscriminado.

Último teste iniciado na suíte interrompida: tests/v3.6-hotfix.browser.cjs. Testes iniciados: 21. Ver logs antes de escolher o índice de retomada.
