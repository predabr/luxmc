# LAN: correção da revisão 3

A revisão 3 falhou no computador do usuário porque o executável oficial EasyTier 2.6.4 importa packet.dll. A exclusão dessa biblioteca da preparação foi incorreta. A revisão 4 conserva o motor e inclui Packet.dll do mesmo arquivo ZIP oficial, com SHA-256 fixado em c7c03a87eac7243ccbe331554624b18803010b740e311fc8cfddb573096eacac. WinDivert64.sys continua fora da lista e não é extraído. Não há instalação de WinPcap/Npcap nem execução de um instalador desses drivers. A rede usa Wintun.

O carregamento PE dos quatro arquivos foi examinado. O core depende diretamente de packet.dll; Packet.dll e Wintun dependem apenas de bibliotecas do Windows. Core e CLI executaram --version com código zero usando apenas esses quatro arquivos em uma pasta isolada.

O teste de processo real iniciou o core com --no-tun, sem criar adaptador, e consultou o RPC. Isso identificou outra falha: peer retorna um array, enquanto node info retorna os dados do próprio computador. O código passa a consultar ambos, normaliza somente IP e nome, descarta configuração e segredo e exclui o próprio computador da lista dos amigos. O segredo privado fornecido pelo ambiente foi conferido em memória e aplicado pelo core. Os fixtures salvos contêm apenas nome e IP.

Criar ou entrar prepara os componentes automaticamente no backend. A interface remove o botão de instalação e mostra as etapas, bytes recebidos e porcentagem quando o tamanho é conhecido. O estado de preparação é mantido no backend e pode reaparecer depois da navegação. Há uma verificação --version antes de pedir autorização para o adaptador. Uma falha no carregador encerra a tentativa com código do Windows e não abre a caixa de sistema que deixava o usuário preso.

O broker registra o código de saída, tem guarda para encerrar seu filho em retornos antecipados e interrompe a tentativa depois de 60 segundos sem IP. A saída da rede permanece disponível mesmo quando falta um componente. A tela recebe cartões separados para criar e entrar, etapas acessíveis, movimento reduzido e lista de computadores com IP copiável.

O antivírus bloqueou um comando PowerShell longo usado para editar o componente Svelte por um script Python inline. O texto informado pelo usuário coincide com essa edição local. O comando não foi repetido, não houve liberação de antivírus e as alterações foram feitas pela ferramenta de arquivos. Isso não classifica genericamente outros alertas como falsos positivos.

## Verificações

- pnpm check: zero erros e zero avisos.
- cargo check --offline --jobs 1: aprovado, cinco avisos existentes.
- Quatro testes unitários de rede aprovados, incluindo o formato real do CLI e descarte de configuração privada.
- Teste de interface: preparação automática sem botão de instalação, progresso de 50%, convite, navegação, saída, entrada e armazenamento aprovado.
- Processo real: carregamento, inicialização sem TUN, RPC e aplicação do segredo privado aprovados.
- Regressão do erro original: uma cópia isolada sem Packet.dll terminou com 0xC0000135; o modo de erro suprimiu a caixa de sistema. Com os quatro arquivos, o core iniciou e respondeu. Nenhum driver foi criado por esses testes.
- Não foram testados o adaptador elevado desta revisão, o transporte de Minecraft entre dois PCs nem a travessia dos roteadores dos usuários. Esses resultados não são substituídos pelos mocks do navegador ou pelo teste sem TUN.
- Versão local 3.5.0, revisão 4; nenhuma publicação em GitHub ou site nesta tarefa.

Instalador concluído: release-windows/Luxmc-3.5.0-revisao-4-LAN-teste.exe, 20.954.251 bytes. SHA-256: 0b92a8d31d9eda4131493e6688d54babb4c9e0f38661f948251e4ab5f371c6e8. O build NSIS terminou e o binário foi examinado para confirmar o hash da Packet.dll, a licença Npcap e o comando automático de rede. O arquivo continua sendo um teste local, não uma versão publicada ou uma comprovação de LAN entre dois PCs.

## Identificação da biblioteca

A versão PE da Packet.dll do pacote oficial é Npcap 1.79, Insecure.Com LLC. Sua licença específica acompanha a preparação, além das atribuições WinPcap. Não deve ser identificada somente como WinPcap/BSD. O instalador local não contém binários Npcap; a preparação busca o pacote externo ao conectar. A licença Npcap tem limites de uso pessoal e restrições de integração e redistribuição. A preparação automática da revisão 4 permanece experimental para o teste local e precisa de revisão da licença ou substituição desta dependência antes de uma publicação geral. Não foi obtida licença OEM para o Luxmc.
