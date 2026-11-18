# Entrega de 3 de outubro de 2026

## Alterações

- A sala de hospedagem permanece acessível ao retornar à aba Amigos. A busca para adicionar amigos aceita a seleção explícita ou o nome exato, e o painel lateral tem largura configurável de 280, 320 ou 360 px.
- A prévia da skin continua animada enquanto o Minecraft está rodando. Há uma preferência para pausar essa animação durante o jogo.
- Avatares personalizados são publicados na rede social e usados no portal, na lista de amigos e nas salas. A migração adiciona apenas a coluna `avatar_url`; as imagens PNG recebidas têm tamanho e assinatura validados. As preferências permitem ocultar a aparência e a atividade de jogo.
- As oito abas existentes de configurações receberam controles e ações funcionais: atualizações, logs, identidade social, idioma do sistema, largura do painel, Java, comandos da instância, privacidade e resumo de armazenamento.
- O relatório automático pós-partida foi removido. Os dados internos de tempo de jogo continuam disponíveis.
- A verificação de inicialização considera os arquivos dos mods, sem invalidar o cache apenas porque o jogo salvou suas opções ou configurações. Isso evita repetir a reparação completa por esse motivo; não representa uma medição do tempo de inicialização de cada modpack.
- As instâncias vinculadas a modpacks oferecem seleção de versões, confirmação e aplicação pelo fluxo existente de atualização com backup. Nenhuma versão de um mundo do usuário foi substituída durante os testes.
- O contador mostra “Arquivos de mods”. Arquivos JAR e entradas visíveis no Mod Menu podem ter contagens diferentes por bibliotecas e agrupamento; os seis itens de diferença não foram individualmente classificados.
- O Discord recebeu leitura de IPC com tempo limite no Windows, reconexão periódica, tratamento de rejeição e preferências de privacidade e identificação do aplicativo.
- O site usa modelos de skins Minecraft em três seções, com carregamento progressivo, suspensão fora da tela e suporte à redução de movimento.

## Publicação

O portal existente foi atualizado em https://luxmc-r92.pages.dev. Publicação identificada por https://d1c3aa37.luxmc-r92.pages.dev.

As migrações `0003_mesh_social.sql`, `0004_social_stream.sql` e `0005_social_avatars.sql` foram aplicadas ao banco existente `luxmc-social`. A verificação nativa identificou que as tabelas de bloqueios, salas e tickets ainda não existiam em produção. Os dados das contas existentes foram preservados. O serviço de eventos foi atualizado em https://luxmc-social-stream.waydroid917.workers.dev.

## Verificações

- `social-delivery-check.log`: Svelte e TypeScript sem erros ou avisos.
- `social-modpack-version-tests-v2.log`: cinco testes de seleção de versões e cache aprovados.
- `social-portal-regression-tests-v2.log`: vinte testes de portal, identidade, avatares, protocolo e catálogo aprovados.
- `social-presence-rust-check.log`: compilação Rust aprovada; três avisos preexistentes.
- `social-fast-launch-tests.log`: mudanças em opções não invalidam a verificação; mudanças em mods invalidam.
- `social-room-avatar-tests.log`: validação de avatares aprovada.
- `social-host-navigation-browser-v3.log`: navegação de retorno à hospedagem, participantes, moderação e armazenamento aprovados em ambiente de teste.
- `social-skin-animation-browser-v2.log`: importação, fidelidade de textura, capas, persistência e animação com o jogo ativo aprovadas.
- `social-live-site-browser.log`: site em produção verificado para três modelos, mudança de cena, pausa, suspensão fora da tela e layouts responsivos.
- `social-delivery-build.log` e `social-delivery-windows-build.log`: interface e executável Windows compilados.
- `social-delivery-native-storage.log`: aplicativo real no Windows 11, armazenamento positivo incluindo o próprio executável, sem avisos de leitura.
- `social-delivery-native-ui.log`: painel de 320 px, navegação para adicionar amigo e resposta positiva do Discord via IPC real.
- `social-delivery-offline-navigation.log`: a tela de adicionar amigo permanece aberta com o serviço social indisponível; diagnóstico, busca de comandos e recomendação de RAM no Windows continuam funcionando.
- `social-delivery-native-sync.log`: sincronização do aplicativo com o portal aprovada após as migrações pendentes. `portal-delivery-avatar-count.log` confirma um avatar publicado no banco.

Executável entregue: `release-windows/Luxmc-2.0.2-final.exe`. A compilação usa o perfil de desenvolvimento do Rust com a interface de produção embutida; não é um instalador assinado.

O funcionamento entre dois computadores reais na mesma sala continua dependendo de um teste com outro participante. Alterações de avatar da sala são aplicadas ao entrar novamente. Servidores pagos 24 horas continuam reservados para uma etapa futura, conforme solicitado.
