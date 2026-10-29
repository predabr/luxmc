# Servidores, salas P2P e Android

## Servidores no seu PC

A linha 4.0.1 inclui o painel de servidores dedicados. Escolha Vanilla, um loader ou um modpack compatível, configure o mundo e inicie o servidor. Ele tem uma pasta e um mundo separados das instâncias usadas para jogar. Conteúdos, jogadores, console e configurações são administrados pelo painel.

O dono usa **Compartilhar por P2P → Copiar código**. O amigo abre **Jogar juntos** no Luxmc, cola o convite e inicia uma instância com a versão e os mods necessários. O servidor aparece na lista LAN do Minecraft, com alternativa de copiar o endereço ou adicioná-lo à instância. Não é necessário abrir um mundo para LAN ou criar conta em um provedor de túnel.

Todos mantêm o Luxmc conectado durante a partida. O mundo continua no computador do dono, que precisa permanecer ligado. Encerrar apenas o compartilhamento desconecta os amigos sem parar o processo do servidor. A opção **Parar e salvar** encerra o servidor; a exclusão confirmada remove os arquivos próprios dele.

| Recurso | Plus · R$ 20 / 30 dias | Pro · R$ 30 / 30 dias |
| --- | --- | --- |
| Hospedar mundo aberto por LAN | Até 5 participantes | Até 10 participantes |
| Hospedar servidor dedicado | — | Acesso Pro confirmado |
| Capacidade da sala do servidor | — | Até 20 participantes, incluindo o dono |
| Memória configurável do servidor | — | Até 8 GiB do computador do dono |

O limite de jogadores do Minecraft é configurado no servidor. A capacidade da sala não garante que todo computador sustente o mesmo modpack ou número de jogadores. CPU, RAM disponível para servidor e cliente, disco, conteúdo e conexão influenciam a experiência.

## O que foi verificado

Um servidor Minecraft Vanilla 1.21.1 temporário respondeu pelo relay público com o transporte IP direto desativado nos endpoints de teste. Foram verificados metadados da sala, resposta do protocolo Minecraft, comandos, salvamento e parada normal. Há testes da descoberta LAN, transporte de bytes, controles da sala, distribuição do resource pack e encerramento do compartilhamento sem parar o servidor.

Uma partida entre dois PCs em redes domésticas diferentes continua sendo uma validação necessária. O funcionamento depende da disponibilidade do transporte direto ou do relay e das condições da rede. Modpacks exigem loaders, dependências e arquivos próprios para servidor; mods exclusivos do cliente não são executados nele. A distribuição de um resource pack não instala mods nem faz clientes sem suporte renderizarem skins personalizadas.

## Endereço público independente do Luxmc

Esta modalidade usa uma sala P2P do Luxmc. O convite não é um domínio público que qualquer Minecraft possa acessar sozinho. Hospedagem em nuvem, disponibilidade com o PC desligado e endereço público independente do launcher não estão incluídos. Esses recursos exigiriam infraestrutura adicional.

## Android

Android continua planejado, sem data anunciada ou APK oficial. A primeira etapa proposta reúne conta, amigos e catálogo. Executar Minecraft Java no celular exige runtime, gráficos, controles de toque e testes próprios de loaders em aparelhos reais.

Consulte as [releases oficiais](https://github.com/predabr/luxmc/releases) para identificar os instaladores disponíveis. O código em desenvolvimento pode estar à frente dos pacotes publicados.
