# Luxmc 3.9.1: início do Minecraft, atualização e planos

## Alterações

- O launcher só se oculta depois que o processo ativo do Minecraft cria uma janela visível no Windows.
- A opção de fechar ao jogar preserva o processo do launcher em segundo plano, incluindo os logs, o acompanhamento do jogo e a sala LAN.
- Encerramento imediato do Java, resposta de início recebida depois de um evento de saída e falha do monitor deixam de ser apresentados como início concluído.
- Erros do jogo restauram a janela do launcher. O acompanhamento continua ao mudar de aba.
- Aberturas simultâneas são recusadas enquanto a instância ativa está em execução.
- A preparação do jogo deixou de repetir no frontend a verificação e o download já executados no backend. As verificações de integridade continuam no backend.
- O atualizador verifica o download e distingue bloqueios do antivírus e da política do Windows. Ele acompanha a saída imediata do instalador e mantém a possibilidade de tentar novamente.
- No Windows, o atualizador não encerra o launcher depois de apenas criar o processo do instalador. O instalador NSIS controla o encerramento quando chega à etapa que substitui os arquivos.
- A página de planos usa as cores, fontes, navegação e composição da página principal. Preços, autorização de pedidos, vínculo de conta e processamento de comprovantes foram preservados.

## Verificações

- `pnpm check`: nenhum erro ou aviso.
- `pnpm test:run`: 185 testes aprovados em 32 arquivos.
- `pnpm test:website`: 85 testes aprovados.
- `cargo check --release --features tauri/custom-protocol --locked`: aprovado; cinco avisos anteriores de código não utilizado.
- Suite Rust: 236 testes aprovados, sete testes de integração externa ignorados. O último ajuste de manter a janela durante a abertura do instalador foi compilado novamente; não altera o helper de acompanhamento exercitado por essa suite.
- Fluxo de início no navegador com processo simulado: espera pela janela, falha antes e depois da resposta, troca de aba e preservação do monitor aprovados.
- Fluxo do atualizador no navegador: configuração de busca automática, erro de consulta, seleção do instalador e nova tentativa depois de jogo aberto aprovados.
- Página de planos em 1440 e 390 pixels: sem transbordamento horizontal ou erros de script.
- Página publicada: https://luxmc-r92.pages.dev/planos. Publicação 295c4440.luxmc-r92.pages.dev.

## Limites

O amigo informou a versão 3.9.1, mas não forneceu o modpack, a versão do Minecraft ou os logs. As correções cobrem o fluxo que escondia ou encerrava o launcher antes de confirmar o jogo; o erro específico do Minecraft desse computador ainda precisa ser confirmado em uma nova tentativa.

Não houve teste real de dois computadores, execução do Minecraft ou abertura do launcher instalado nesta revisão. Os testes de navegador usam um processo simulado e não comprovam compatibilidade com todo modpack ou conectividade de uma rede externa.

Os executáveis continuam sem assinatura Authenticode de um editor confiável. SHA-256, assinatura do manifesto de atualização e uma análise do Defender não substituem essa assinatura. Esta revisão não confirma liberação pelo Controle Inteligente de Aplicativos nem aprovação dos fornecedores que receberam os instaladores anteriores.

Nenhuma configuração de proteção do Windows foi alterada. Não foi publicada uma nova versão no GitHub, e o instalador público original 3.9.1 foi preservado.

## Evidências

Os relatórios, capturas, logs de compilação, auditoria e recibo da instalação local ficam em `docs/validation/launch-update-plans/`. O recibo da instalação registra os hashes antes e depois, a cópia de segurança, a preservação dos dados e a ausência de execução automática.

## Instalação local concluída

- Executável instalado: `C:/Users/preda/AppData/Local/Luxmc/luxmc.exe`.
- SHA-256 instalado: `1205adbe8506dbefefbbf12bcb7813c1cee0d4a011b8258a26af234be502ce05`.
- Instalador para esta revisão: `C:/Users/preda/Downloads/Lux MC Launcher - Inicio Seguro.exe`.
- SHA-256 do instalador: `9e4e0a332494b81480f67db3c1eb2f831767fab8a79498614e641a042fd77cb1`.
- Cópia de segurança: `C:/Users/preda/AppData/Local/Luxmc-build/backups/inicio-seguro-20261012-190353`.
- Três arquivos de dados existentes foram preservados e conferidos por hash. Dois atalhos continuam apontando diretamente para o launcher.
- O programa extraído do instalador confere com o executável compilado; a única diferença são os três bytes do identificador de pacote NSIS inserido pelo Tauri.
- Defender analisou instalador, programa e desinstalador sem detectar ameaça. Os três continuam com Authenticode `NotSigned`.
- O instalador público original permaneceu com SHA-256 `516ef1b8a340f4ee4e1971564de9d9370075b360d94ae0f6cc7867ff7b270c39`.
