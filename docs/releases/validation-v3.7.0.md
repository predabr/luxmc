# Verificação do Luxmc 3.7 — Jogar Juntos

Preparação da publicação em 10 de outubro de 2026. A versão 3.7 reúne o código validado nas revisões locais 3 e 4 da 3.6 e atualiza os metadados do launcher, instalador, reparo, site e publicação.

## Base funcional validada

- 150 testes frontend, 205 testes nativos e 50 testes do site aprovados na revisão local, além de 12 verificações de manifestos e assinatura. Seis integrações optativas ficam fora da execução nativa padrão.
- 17 suítes de interface aprovadas, incluindo catálogo, conteúdo, criação e exclusão de instâncias, gerenciamento de Java, salas, dono, cancelamento, wallpapers, animações e oficina.
- Interface compilada conferida para salas, tags públicas, confirmação de exclusão, controles de membros e painéis com wallpaper.
- Transferência pelo relay público com transporte direto desativado, convite curto resolvido em produção e descoberta multicast pelo listener padrão do Java 8 com transferência TCP pelo endereço anunciado.
- O Shield do executável Windows da revisão local analisou 730 mods de quatro modpacks instalados sem indicadores conhecidos. Nenhum mod foi executado ou removido para essa conferência.
- A atualização real de um modpack preservou a pasta anterior e os mundos. O pacote Essentials foi preparado nas seis instâncias Java testadas, sem alterar outras opções.
- Exclusão de perfil exige confirmação do nome e a identidade autenticada do dono no servidor. Tags atribuídas ao próprio dono, a outros usuários e recebidas por sincronização foram verificadas. Pedidos anônimos e de membros são recusados. Nenhum perfil real foi excluído nos testes de produção.
- A atualização local conservou seis instâncias, uma conta e os hashes dos arquivos dos cinco mundos presentes. O reparo confirmou a integridade e a janela do launcher abriu e respondeu.

## Publicação 3.7

A numeração é 3.7.0, revisão inicial 0. O instalador continua com o nome **Lux MC Launcher.exe**. Os metadados dos pacotes nativos, dos idiomas, do reparo e do atualizador precisam corresponder à mesma versão antes da publicação. O título da release é obtido das notas da versão para conservar o nome **Luxmc 3.7 — Jogar Juntos**.

- `pnpm check`: zero erros e zero avisos; `cargo check --locked`: aprovado, com os avisos de código não utilizado descritos abaixo.
- 153 testes frontend, 50 testes do site e 12 testes de manifestos e assinaturas aprovados na fonte 3.7. Os testes de versão cobrem a atualização da 3.6 revisão 4 para a 3.7 e impedem oferecer uma revisão da 3.6 como atualização da 3.7.
- Quatro cenários de identidade do site conferidos no navegador, incluindo cache da versão anterior, versão atual, revisão futura e versão futura. O site foi compilado e conserva a identificação histórica das capturas e da medição de desempenho.
- A [verificação de código no CI](https://github.com/predabr/luxmc/actions/runs/38089437127) e a [geração da release](https://github.com/predabr/luxmc/actions/runs/38089440411) correspondem ao commit `ffaf5dd`, marcado como `v3.7.0`.
- O CI terminou com sucesso no Windows e no Linux. Foram aprovados 205 testes da biblioteca nativa no Windows e 203 no Linux; seis testes optativos ficam fora da execução padrão em cada plataforma. As integrações Linux e a regressão de exclusão de instâncias no Windows também passaram.
- O pacote Arch recebeu `license = custom:Luxmc` e o arquivo `usr/share/licenses/luxmc/LICENSE` antes da publicação dos manifestos. A conferência preservou todos os demais arquivos e o hash do executável; o gerador e o PKGBUILD também foram corrigidos, com sintaxe verificada.

A [release Luxmc 3.7 — Jogar Juntos](https://github.com/predabr/luxmc/releases/tag/v3.7.0) foi publicada em 10 de outubro de 2026, às 22:49 UTC. Todos os jobs terminaram com sucesso: sete instaladores Windows, Linux e macOS conferidos e quatro arquivos de manifesto e assinatura publicados. As assinaturas de `latest.json` e `SHA256SUMS` foram verificadas com a chave Ed25519 oficial, e os hashes assinados correspondem aos sete arquivos públicos. A assinatura do manifesto não representa um certificado Authenticode ou uma notarização do macOS.

O instalador Windows tem 21.289.301 bytes e SHA-256 `275991c96f1eae9087251bf16bef71847076fb1506eb3ca69f7c7b0fb903da6e`. Instalação, reinstalação da mesma versão e remoção do aplicativo e do registro passaram no runner da release. A [verificação adicional em uma máquina Windows nova](https://github.com/predabr/luxmc/actions/runs/38092911326) confirmou também a remoção completa dos dados descartáveis, preservando os arquivos do projeto e o documento de teste. Nenhum dado do computador do usuário foi apagado para essa verificação.

O [site oficial](https://luxmc-r92.pages.dev) foi publicado com a versão 3.7.0. Foram conferidos a versão exibida, a API de release, os seis caminhos de download e os onze arquivos públicos do GitHub. O download Windows do site entrega exatamente os bytes e o hash do instalador assinado no manifesto, conservando o nome **Lux MC Launcher.exe**. O atualizador detectou a release pública 3.7.0 em um navegador com cliente 3.0.2 simulado, sem instalar nada; a passagem da 3.6 revisão 4 para a 3.7 também foi coberta pelos testes de versão.

As cinco capturas históricas da 3.6 foram conferidas no site como recortes da janela, incluindo o personalizador. Os arquivos de estilos e animações publicados correspondem aos arquivos locais verificados; as capturas e a medição histórica continuam identificadas como material da 3.6.

## Limites

A partida real entre dois computadores em redes diferentes ainda depende do teste dos participantes. Os testes automatizados de rede verificam transporte, convite e descoberta, sem modificar mundos. Simulações de interface não comprovam login Microsoft, execução de qualquer modpack ou funcionamento de toda versão Bedrock.

O Shield verifica hashes e indicadores estáticos documentados. Os testes usam dados inertes em arquivos temporários; não executam malware, desativam o antivírus ou restauram arquivos em quarentena. A proteção complementa o antivírus. Contagem de usuários online depende da presença recente enviada ao serviço; clientes sem conexão não podem ser observados como online.

O `cargo check` local no Windows conserva cinco avisos de código Rust não utilizado; a compilação Linux registra seis. A auditoria Rust passou com sete avisos permitidos de dependências sem manutenção, e permanece o aviso documentado em `braces`, da cadeia de compilação. A medição de desempenho apresentada no site foi feita na 3.6 e permanece identificada como histórica. Nenhuma medição de consumo fixo ou garantia universal de desempenho foi atribuída à 3.7.
