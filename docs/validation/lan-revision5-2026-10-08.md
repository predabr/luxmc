# LAN e entrada no mundo: revisão 5

O executável encontrado em execução neste PC, AppData/Local/Luxmc/luxmc.exe, não continha virtual_lan_connect e ainda tinha o túnel antigo tunnel_status: não era nenhuma das revisões com EasyTier entregues nesta conversa. Seu SHA-256 era 9fc2df5ac7fa8e189042180862fc63e7a106d17e05bfa0ce02c02f19766ae8b2. Isso comprova o arquivo local examinado, não a instalação do computador do amigo. A revisão 4 produzida anteriormente não estava aplicada nesse caminho.

A revisão 5 identifica sua revisão na tela LAN e no instalador e registra LuxmcRevision no registro de instalação. Não se deve tratar somente o número 3.5.0 como prova de atualização.

A rede virtual da revisão anterior não transportava a descoberta do mundo para a lista LAN do Minecraft. Esta revisão acrescenta uma camada de compartilhamento Java: detecta a porta do mundo local, verifica o protocolo de status do jogo, publica metadados autenticados pelo segredo da rede e encaminha o jogo por uma porta do próprio launcher. O PC do amigo cria um endereço de loopback e anuncia o mundo no multicast local do Minecraft. A interface lista apenas mundos cujo Minecraft respondeu. Também oferece entrada por uma instância escolhida e cópia de endereço local quando o jogo já está aberto.

Se o anúncio do próprio Minecraft não for detectado, há uma alternativa para o anfitrião informar a porta mostrada no chat. Essa porta é conferida pelo protocolo Minecraft antes de ser publicada. O fluxo atende ao Java; não implementa descoberta RakNet de Bedrock.

A regra de firewall é criada pelo broker elevado somente para o programa Luxmc, TCP 39945 e 39946 e a sub-rede privada da sala. Não desativa o firewall nem abre portas no roteador. Seu nome é específico da sessão e a guarda remove essa regra ao encerrar normalmente o broker. Um encerramento forçado do broker pode impedir a guarda de executar; isso requer auditoria posterior de regras órfãs antes de publicação geral.

O status ativo passa a exigir que o IP exista de fato no Windows, em vez de apenas aparecer na configuração do motor. O convite tolera até cinco minutos de diferença de relógio para seu limite futuro. Colar uma porta numérica no campo de convite agora explica a diferença entre porta do Minecraft e convite Luxmc.

A entrada direta também usava os argumentos antigos --server e --port em versões novas do Minecraft. A revisão usa Quick Play quando o metadata oferece a opção ou a versão corresponde às séries modernas, mantendo os argumentos anteriores para clientes antigos.

Testes reais de dois processos EasyTier sem TUN confirmaram a descoberta mútua pelo relay configurado. Esses processos foram encerrados no final. Esse teste não comprova adaptadores Windows elevados, roteadores diferentes ou uma partida Minecraft entre dois PCs.

Os testes de encaminhamento usam sockets reais de loopback. A resposta de status Minecraft é uma fixture, não um jogo completo. O teste valida o caminho de controle autenticado e as duas pontes TCP, mas não autenticação Microsoft durante uma partida.

Nenhuma publicação no GitHub ou no site foi realizada. A licença da Packet.dll Npcap continua sujeita às limitações documentadas na revisão 4 e precisa de resolução antes de distribuição geral.

Verificações finais: pnpm check passou sem erros ou avisos; cargo check passou com cinco avisos existentes; sete testes Rust filtrados por virtual_ passaram após as últimas alterações; o teste de Quick Play e os testes de navegador do fluxo LAN também passaram. O build de produção e o pacote NSIS foram concluídos.

Instalador local: release-windows/Luxmc-3.5.0-revisao-5-LAN-teste.exe, 20.997.995 bytes, SHA-256 1bd491ee764258c0b454cf4f1a9702db4f0e70a58614b40adea713218ba221bf. O instalador foi executado neste PC e registrou LuxmcRevision=5. O executável instalado difere do executável intermediário somente nos três bytes do marcador de empacotamento PE_VAR_UNK/PE_VAR_NSS, esperado pelo empacotador Tauri. O verificador instalado retornou healthy=true, repaired=false e rolledBack=false. O executável novo foi iniciado para conferência de processo; isso não equivale a validação visual ou a uma partida entre dois PCs.

## Solicitação de desinstalação completa

O usuário pediu explicitamente remover o launcher, contas, configurações e instâncias, preservando o projeto. A inspeção de metadados encontrou três diretórios de instâncias, todos dentro de AppData/Roaming/github/Luxmc/data/instances. Nenhuma leitura de tokens de conta foi feita para esse inventário.

O desinstalador oficial executou e retirou o executável antigo. A exclusão dos dados iniciou depois de conferir caminhos absolutos e impedir qualquer alvo dentro do projeto ou ancestral dele. Parte dos arquivos foi removida, mas a pasta de dados permaneceu. A revisão automática bloqueou a tentativa explícita de excluir luxmc.db e também a exclusão dos outros diretórios de dados. As mensagens não forneceram motivo específico. Não foi tentado contornar o bloqueio por outra linguagem, shell ou alteração de conteúdo do banco.

O usuário informou que não consegue realizar a limpeza manual. Os dados restantes devem ser considerados preservados. A instalação do programa novo pode continuar, mas não pode ser descrita como instalação com todos os dados zerados. O projeto e o diretório de build que pertence a ele permanecem preservados.
