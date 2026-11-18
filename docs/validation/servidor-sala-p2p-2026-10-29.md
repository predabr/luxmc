# Servidor dedicado na sala P2P do Luxmc

Revisão `.911` instalada e aberta em 29/10/2026 às 20:11 (São Paulo).

O dono inicia o servidor e escolhe **Compartilhar por P2P**. Depois copia o convite ou abre **Ver sala e participantes**. O amigo entra em **Jogar juntos** no Luxmc, cola o convite e abre uma instância Minecraft compatível. O proxy do convidado anuncia o servidor na lista LAN do Minecraft, sem exigir que o dono abra um mundo para LAN. Copiar o endereço e adicionar à instância continuam disponíveis como alternativa à descoberta automática. Ambos precisam manter a conexão do Luxmc ativa.

O jogo e o mundo continuam em um processo dedicado no computador do dono. A sala usa a conexão criptografada existente, com caminho direto quando disponível e relay automático quando necessário. Não exige uma conta adicional de hospedagem, domínio próprio ou alterações manuais no roteador. A criação e administração de servidores continuam restritas ao Pro; as regras da LAN do Plus foram preservadas.

Foi corrigida a separação entre os processos: a sala do servidor pertence ao serviço de servidores, enquanto as salas LAN comuns pertencem à janela principal. A consulta unificada encontra a sessão existente e encaminha os comandos ao seu proprietário. O painel de Jogar juntos agora apresenta também a sala dedicada, com participantes, renovação do convite, controle de novas entradas e encerramento do compartilhamento. Encerrar somente a sala não para a JVM nem apaga o mundo.

A pré-checagem impede iniciar uma segunda sala enquanto já existe uma sessão. Falhas na consulta são exibidas, com tempo limitado, em vez de ocultadas como se não houvesse sala. Consultar uma sala dedicada inexistente não inicia o serviço nem cria outra cópia do executável para quem só usa LAN.

Testes da interface reproduzem o estado real do problema: consulta da janela vazia e sala presente no serviço separado. Conferem abrir o painel, trancar e destrancar a sala, parar apenas o compartilhamento, manter o servidor em execução e compartilhar novamente.

O teste real isolado usou Minecraft Vanilla 1.21.1, criou endpoints distintos com transporte IP direto desativado e confirmou a resposta do protocolo Minecraft pelo relay público, junto com os metadados da sala dedicada. Também confirmou comandos, salvamento, parada e exclusão do servidor temporário. Este teste não equivale a uma partida entre dois PCs em redes domésticas diferentes. A conexão remota depende da disponibilidade de transporte direto ou do relay utilizado pelo protocolo.

Não fornece domínio público para clientes que não estejam conectados pela ponte Luxmc. Esta revisão atende à escolha do usuário por P2P integrado ao launcher.

Validação final: 360 testes Rust aprovados, 18 optativos ignorados; 239 testes do frontend; Svelte sem erros nem avisos; interface em 1440, 1366 e 960 pixels e no tema claro. Teste de servidor real e relay em 65,26 segundos. Instalador .911 concluído e conferido contra 559 fontes congeladas.

Instalação com backup `backup-20261029-201054`: 2 contas, 4 perfis, 60 arquivos protegidos e 17 tabelas preservados. Conteúdo do executável instalado conferido contra o payload NSIS. GUI aberta e serviço de servidores na revisão `40255548682911`; catálogo nativo retornou 100 versões. Nenhum servidor pessoal foi iniciado durante a instalação.

Instalador: `C:/Users/preda/Downloads/Luxmc 4.0 - sala p2p.exe`. SHA-256: `c8efadbf496a6831e713e4a016da063028c9e9e8adfda062d3020e32e108f7ec`. Executável instalado SHA-256: `c13c3a674e878954c921e1baddcdbe8680313db5b935cdcd94afb77366b48dde`. Recibo completo privado: `%LOCALAPPDATA%/Luxmc-build/requests/local-server-20261026/room-delivery.json`. Nenhuma publicação no GitHub ou site foi feita neste pedido.
