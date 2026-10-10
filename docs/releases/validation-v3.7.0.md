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

A publicação gera pacotes Windows, Linux e macOS pelos jobs da release, confere os arquivos e publica os manifestos assinados com a chave Ed25519 oficial. A assinatura do manifesto não representa um certificado Authenticode ou uma notarização do macOS. As verificações e os resultados finais da publicação serão registrados nesta seção quando os jobs terminarem.

## Limites

A partida real entre dois computadores em redes diferentes ainda depende do teste dos participantes. Os testes automatizados de rede verificam transporte, convite e descoberta, sem modificar mundos. Simulações de interface não comprovam login Microsoft, execução de qualquer modpack ou funcionamento de toda versão Bedrock.

O Shield verifica hashes e indicadores estáticos documentados. Os testes usam dados inertes em arquivos temporários; não executam malware, desativam o antivírus ou restauram arquivos em quarentena. A proteção complementa o antivírus. Contagem de usuários online depende da presença recente enviada ao serviço; clientes sem conexão não podem ser observados como online.

Permanecem os cinco avisos anteriores de código Rust não utilizado e o aviso documentado em `braces`, da cadeia de compilação. A medição de desempenho apresentada no site foi feita na 3.6 e permanece identificada como histórica. Nenhuma medição de consumo fixo ou garantia universal de desempenho foi atribuída à 3.7.
