# Servidores e Android: próximos passos do Luxmc

Atualizado em 26/10/2026. Este documento descreve trabalho em desenvolvimento e planejamento; não anuncia uma nova release ou APK.

## Jogar pelo endereço

O objetivo do servidor aberto é simples: informar o endereço em **Multijogador → Adicionar servidor** e entrar, sem convite obrigatório do Luxmc. O dono poderá escolher nome, imagem e descrição do servidor, acompanhar jogadores e administrar acesso. Servidores privados poderão usar whitelist e convites opcionais.

Um nome como `owner.<domínio-do-projeto>` é apenas um exemplo. Nenhum domínio está sendo oferecido ou reservado por este documento. Entrada direta em Minecraft padrão exige DNS e uma entrada TCP pública que realmente alcance o PC de quem hospeda. O túnel P2P atual, com proxy local e launcher do participante aberto, não se transforma nesse serviço apenas por receber um nome. A definição de infraestrutura e sua validação fazem parte do trabalho pendente.

## Plus e Pro

| Recurso planejado | Plus | Pro |
| --- | --- | --- |
| Hospedar mundo aberto por LAN | Até 10 participantes | Até 20 participantes |
| Hospedar servidor dedicado pelo Luxmc | — | Até 20 participantes, incluindo o dono |
| Heap Java do servidor gerenciado | — | Configurável até 8 GiB, usando a RAM do PC |
| Administrar mods, modpack, datapacks, resource packs e mundos do servidor | — | Incluído no escopo em desenvolvimento |

Jogadores autorizados não precisam ter Pro para entrar em um servidor Pro. A assinatura do anfitrião não inclui um computador em nuvem, armazenamento de mundos ou garantia de desempenho. O PC que hospeda precisa permanecer ligado.

A LAN existente está em revisão, incluindo os problemas relatados em modpacks. Antes de anunciar a correção como concluída, é necessário validar descoberta e conexão direta, Vanilla e modpacks, entre dois computadores. A implementação de servidor dedicado já existe em uma revisão local experimental, com falhas relatadas em acesso e compartilhamento; o novo painel e os benefícios acima ainda não foram entregues nos downloads públicos. Não escolha um plano contando com uma função futura como se já estivesse disponível.

## Controle do servidor

O painel planejado reúne a lista de servidores, estado, endereço e abas de conteúdo, jogadores, console, configurações e backups. O dono poderá buscar ou importar conteúdo compatível, revisar dependências e aplicar mudanças com backup e reinício quando necessário. Resource packs serão distribuídos aos clientes por um mecanismo acessível e validado; não são mods executados no servidor.

O painel de jogadores distinguirá **online agora**, **perfis únicos que já entraram** e **jogadores conhecidos offline**. Haverá ações de expulsar, banir e desbanir, com identidade Minecraft validada e persistência após reinício. Pessoas que nunca entraram não poderão ser contadas como jogadores offline conhecidos.

Autosave, parada limpa, cópias consistentes e retenção de backups fazem parte dos critérios de entrega. A revisão também deve preservar a instância original e os mundos ao preparar um servidor de modpack.

## Android

Uma versão Android está planejada, sem data anunciada e sem APK oficial disponível. O [plano técnico](../plans/android-apk.md) prevê começar com conta, amigos e catálogo; executar Minecraft Java no celular exige uma etapa própria de runtime, gráficos, controles de toque e testes de loaders em aparelhos reais.

O site continuará mostrando Android como planejamento até existir um pacote oficial verificado. APKs de terceiros não são distribuídos nem endossados pelo projeto.

## Como acompanhar

Consulte as [releases oficiais](https://github.com/predabr/luxmc/releases) para saber o que foi efetivamente publicado. Um item só muda de planejado para disponível após implementação e validação. Resultados de conectividade dependem da rede, do firewall e de serviços de relay; não há promessa de funcionamento universal ou de zero erros.
