# P2P, solicitações de amizade e console — 3 de outubro de 2026

## Correções

- O convidado não compara a validade do convite com o próprio relógio. O hospedeiro continua verificando o segredo e o prazo da sala, e informa quando o convite expirou ou foi substituído.
- O backend aceita o convite completo e o link `luxmc://join/world`. As restrições de tamanho, versão, lotação, bloqueio da sala e expulsão foram preservadas.
- A pesquisa de amigos aceita `Nickname#código`. Nicknames repetidos exigem escolha explícita; a primeira identidade encontrada não recebe convites automaticamente. A busca ordena os resultados pela atividade recente.
- Solicitações recebidas geram notificação, aparecem no painel lateral e podem abrir diretamente a aba de solicitações. Pedidos enviados permanecem visíveis como aguardando resposta.
- A lista atualiza ao retornar à janela. Sem conexão ao serviço de eventos, há atualização a cada cinco segundos; com conexão ativa, a presença é renovada a cada 25 segundos. Falhas iniciais de conexão têm nova tentativa automática.
- O endereço do serviço de eventos foi incluído na configuração do portal existente.
- O executável usa o subsistema gráfico do Windows em todas as compilações. Hooks, encerramento do jogo e processamento de wallpapers usam os helpers de processos sem janela de console.

## Evidências

- `multiplayer-check.log`: Svelte e TypeScript sem erros ou avisos.
- `multiplayer-rust-check.log`: Rust compilou com três avisos preexistentes.
- `multiplayer-tunnel-tests.log`: seis testes aprovados, incluindo conexão com relógio divergente, transferência de dados, renovação, expulsão e expiração validada pelo hospedeiro.
- `multiplayer-public-relay-test.log`: transferência aprovada usando exclusivamente o relay público, sem transporte IP direto.
- `multiplayer-portal-tests.log`: 17 testes aprovados, incluindo recebimento pelo destinatário, consentimento e nicknames repetidos.
- `multiplayer-friends-browser.log`: a interface protege nicknames ambíguos, envia pelo código correto, mostra pedido enviado, recebe a solicitação e aceita o convite.
- `multiplayer-live-portal.log`: duas identidades sintéticas testaram o portal real. O pedido chegou pelo WebSocket e o aceite atualizou as duas listas. As identidades e a relação de teste foram removidas depois.
- `multiplayer-pages-deploy.log`: atualização publicada em https://luxmc-r92.pages.dev, publicação identificada por https://81b53187.luxmc-r92.pages.dev.
- `multiplayer-delivery.json`: cópia em Downloads conferida por SHA-256; cabeçalho PE usa subsistema gráfico Windows, valor 2, em vez do subsistema de console.
- `multiplayer-native-storage.log` e `multiplayer-native-ui.log`: o executável entregue abriu no Windows, respondeu aos comandos nativos, contou o armazenamento e abriu a tela de adicionar amigo corretamente.

Arquivo entregue: `C:\Users\preda\Downloads\Luxmc-2.0.2-multiplayer.exe`. Os dois participantes devem atualizar o launcher e usar um convite novo.

As contas e solicitações existentes foram preservadas. Um pedido anteriormente enviado a um perfil antigo deve ser cancelado e reenviado ao código confirmado pelo amigo. O teste de relay transfere dados TCP; a entrada de dois jogadores no mesmo mundo Minecraft ainda depende do teste dos usuários.

O novo executável continua sendo uma compilação de teste sem assinatura digital. A correção das janelas de console não altera as exigências do Controle Inteligente de Aplicativos.
