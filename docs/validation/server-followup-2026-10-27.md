# Revisão dos servidores — 27/10/2026

## Correções

- Serviço dedicado separado da interface, com processo próprio, autenticação privada em loopback, bloqueio de instância duplicada, limites de mensagens e encerramento por inatividade quando não há servidor ou sessão. Fechar o launcher não envia mais o encerramento ao servidor pertencente ao serviço. O PC continua precisando permanecer ligado.
- Formulário nativo de criação, botões específicos do painel, superfícies com maior contraste, acesso destacado à hospedagem e prioridade ao endereço público. Jogadores mostram avatar e nome, sem número de entradas ou UUID exposto na lista principal.
- Falhar ao sincronizar preferências visuais não invalida um plano Pro já confirmado. Trocar de conta continua limpando o estado; a autorização de hospedagem permanece verificada no backend.
- Criar a partir de uma instância prepara e aplica os conteúdos locais. A cópia exclui mods declarados exclusivos do cliente e arquivos de opções do jogo; não baixa novamente o pacote original. Mods sem metadados de ambiente continuam exigindo compatibilidade em jogo. Mundos existentes são importados separadamente, com proteção de `session.lock`, sem substituir o original automaticamente.
- A aba Modpack foi removida. Arquivos preparados por pacotes aparecem nas categorias de mods, datapacks, texturas e arquivos. A importação de pacotes permanece em Arquivos. Cópia incompleta impede iniciar um servidor vazio por engano.
- Configurações podem ser salvas durante execução para o próximo início. Modo de jogo tem opção explícita de aplicar ao entrar; dificuldade e modo padrão são enviados ao console após o servidor ficar pronto. A pausa quando vazio é desativada. PvP usa propriedade nas versões antigas e regra do mundo nas versões compatíveis.
- Integração opcional Playit com vínculo pelo fluxo oficial, credencial privada, agente oficial fixado em 1.0.10 com SHA-256, aviso de licença, destino em loopback e leitura do endereço retornado pelo provedor. A chave do agente é de leitura e não pode criar túneis da conta: a criação ocorre no painel oficial. Nenhuma compra automática.

## Evidências

Os logs e recibos desta rodada ficam na pasta privada `%LOCALAPPDATA%/Luxmc-build/requests/local-server-20261026/`, com prefixo `followup-`. A rodada final nativa passou com 334 testes, 13 ignorados e nenhuma falha; a suíte de frontend passou com 231 testes. O navegador de produção passou em 1440×900, 1366×768 e 960×600, incluindo fechamento/reabertura do formulário, navegação, avatar, ausência da aba Modpack e salvamento para reinício.

A consulta real ao Playit confirmou que o código de vínculo deve usar o formato oficial de dez caracteres hexadecimais e que a chave do agente é de leitura. O agente de teste foi vinculado após autorização expressa do mantenedor; o painel confirmou o túnel gratuito Minecraft Java com destino 127.0.0.1:64624 e endereço `malls-exploration.tun.ply.gg`. O binário oficial Windows apresentou assinatura Authenticode válida de Developed Methods LLC e hash igual ao release oficial.

O caminho público foi validado por uma resposta de status Minecraft com marcador aleatório, usando o registro SRV (porta 7578) e uma escuta temporária na porta local. A resposta percorreu o relay e voltou ao cliente de teste no mesmo PC; isso não substitui uma partida entre duas redes. Compilação final, instalação e reconexão do serviço devem ser registradas após verificação. Nenhum Minecraft ou servidor Java real foi iniciado pelo agente.

## Fontes oficiais

- [Mudanças nas propriedades e regras no Minecraft 1.21.9](https://www.minecraft.net/en-us/article/minecraft-java-edition-1-21-9).
- [Agente oficial Playit e versões suportadas](https://github.com/playit-cloud/playit-agent).
- [Guia oficial de servidor Minecraft Java com Playit](https://playit.gg/support/how-to-setup-a-mc-server/).
- [Domínio externo no Playit com Cloudflare](https://playit.gg/support/external-domain-cloudflare/).

## Limites

Teste real de partida e modpack entre duas redes permanece com o usuário. Processo do agente vivo não é prova de acesso externo. O endereço do site em Cloudflare Pages atende HTTP e não substitui a entrada TCP do Minecraft. Nome de domínio externo requer controle do DNS e os requisitos comerciais do provedor; nenhum domínio ou plano foi adquirido nesta rodada. Não houve publicação de código, site ou release pública.
