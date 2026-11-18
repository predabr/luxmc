# Instaladores e recursos — revisão local .908

## Falha reproduzida

O instalador oficial NeoForge 21.1.248 usado no Pixelmon estava íntegro e continha a classe principal. Com Java 21, um caminho Windows verbatim para o JAR reproduziu ClassNotFoundException; o mesmo JAR passado por nome relativo abriu a ajuda com sucesso. A pasta de trabalho verbatim também permanece em user.dir do Java.

A execução agora usa o nome relativo luxmc-loader-installer.jar dentro de uma pasta de trabalho Java normalizada, mantendo a validação canônica das pastas antes do processo. A normalização cobre unidades locais, espaços e caminhos UNC. O mesmo tratamento é usado para o processo do servidor e compartilhado entre as famílias de loaders.

Os instaladores oficiais são testados em diretórios temporários, sem iniciar Minecraft, sem aceitar EULA e sem criar mundos. Os arquivos gerados e o reaproveitamento do registro do loader são conferidos.

Quilt: o catálogo de instaladores devolveu digests diferentes dos publicados no Maven oficial para os mesmos arquivos. A conferência do instalador Quilt usa agora o checksum SHA-256 servido pelo repositório oficial. Nenhum download é executado sem conferir o digest esperado; os outros providers mantêm suas verificações. Testes com vetores SHA-1 e SHA-256 rejeitam bytes modificados e hashes inválidos.

## Consumo e interface

Removidas as cópias completas automáticas em alterações de conteúdos, ativação de mods, configuração de loader e importações. As transações locais que permitem desfazer uma aplicação incompleta continuam existentes. Os backups antigos não foram apagados. A aba Backups e sua opção na exclusão do servidor foram retiradas da interface.

A porta continua em Configurações; o atalho Alterar porta saiu da visão geral, que mantém apenas a entrada por endereço quando o servidor estiver pronto.

Novos servidores usam visão 6 e simulação 4. Distâncias já salvas continuam preservadas, com um botão para aplicar o alcance menor. O modo de economia pausa servidores vazios após 60 segundos nas versões compatíveis e permite coleta periódica G1 para liberar memória ociosa em Java 17 ou mais recente. O heap inicial continua em 256 MiB e o teto continua configurável até 8192 MiB. Os flags foram aceitos pelos runtimes Java 17, 21 e 25 presentes na máquina. Modpacks pesados continuam exigindo memória e processamento; não há promessa de consumo nulo ou de percentual não medido.

Animações de entrada das seções e respostas de hover/pressão usam transformação e opacidade, respeitando movimento reduzido. A interface continua consultando o serviço com frequência menor quando está oculta. Também foi corrigida a leitura dos metadados de capas para pacotes maiores que 64 KiB, até o limite validado de 1 MiB.

## Estado de verificação

540 arquivos de fonte congelados. Svelte sem erros e avisos; 232 testes frontend aprovados. Interface de produção aprovada em 1440, 1366 e 960 pixels escuros e em 1440 claro, incluindo ausência de Backups/Alterar porta e persistência do perfil leve.

Instalações reais aprovadas: NeoForge 21.1.248 / Minecraft 1.21.1; Fabric 0.16.9 / 1.21.1; Forge 47.4.10 / 1.20.1. Quilt 0.27.1 / 1.21.1 também passou pelo handler nativo com SHA-256 do Maven em 7,63 s. A repetição final de NeoForge com a pasta Java normalizada passou em 163,14 s. Testes Rust finais: 351 aprovados, 17 ignorados, executados com dois threads; a verificação isolada de parada/salvamento também passou. A rodada paralela anterior encontrou um prazo curto de 100 ms no fixture de parada sob carga, sem falha na repetição isolada ou na rodada completa com paralelismo menor. Nenhum timeout de produção foi afrouxado. Revisão .908 instalada e aberta em 28/10/2026 às 20:40 (Brasília). Backup da atualização backup-20261028-203902; payload NSIS conferido, 2 contas, 4 perfis, 17 tabelas e 639 arquivos protegidos preservados. Serviço separado .908 reconheceu 1 servidor existente e respondeu normalmente; catálogo nativo retornou 100 versões. Nenhum mundo do usuário ou servidor Minecraft foi iniciado. Os instaladores Java oficiais foram executados somente nos testes temporários. Recibos light-ready.json, light-installation.json e light-delivery.json na pasta privada de build. A limpeza da pasta temporária de instalação e do binário de teste foi bloqueada pela revisão automática de política; esses arquivos foram preservados.

Referências técnicas: https://docs.neoforged.net/user/docs/server/ ; https://docs.oracle.com/en/java/javase/21/gctuning/ergonomics1.html ; https://meta.quiltmc.org/index.html . Evidências privadas em %LOCALAPPDATA%/Luxmc-build/requests/local-server-20261026/light-*.log e light-path-reproduction.json.
