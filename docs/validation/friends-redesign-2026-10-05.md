# Tela Amigos — 5 de outubro de 2026

A tela foi reorganizada como um diretório: cabeçalho compacto, estado social em uma linha, filtros Todos/Online/Solicitações, busca, pedidos em linhas e perfil/código de amizade num painel contextual. O estado vazio traz dois passos curtos, sem banner nem grandes cartões de contagem. A ação Adicionar amigo aparece uma vez no cabeçalho, com 48 px de altura.

Os contatos usam linhas com avatar, atividade e ação contextual Entrar no mundo/Convidar. Favoritos vêm primeiro; depois os jogadores em partida e online. Remover, bloquear e copiar endereço ficam no menu secundário nativo, com estado expandido acessível e Escape no controle. A lógica P2P e o MeshPanel foram preservados.

A transparência das superfícies continua seguindo a opacidade configurada. Os testes mediram 18% no wallpaper. O texto mantém opacidade total e recebe uma sombra discreta para ajudar a leitura sobre fundos claros. Menus de ação mantêm a superfície de leitura dos popups existentes.

## Referências consultadas

- [Discord — guia oficial de amigos](https://discord.com/blog/beginners-guide-to-discord-friends): organização por lista, presença e solicitações pendentes. A hierarquia foi adaptada ao Luxmc.
- [Steamworks — Enhanced Rich Presence](https://partner.steamgames.com/doc/features/enhancedrichpresence): atividade e entrada contextual em jogos com amigos.

## Validação

- `pnpm check`: 0 erros e 0 warnings.
- `tests/friends-redesign.browser.cjs`: 4 cenários em Edge (960/1600 px; wallpaper/fundo padrão; vazio/lista). Copiar código, responsividade, favoritos, ordenar presença, filtrar online, aceitar solicitação, busca vazia, menu acessível e transparência passaram.
- `tests/multiplayer-friends.browser.cjs`: nickname duplicado protegido; código exato; convite enviado visível; pedido recebido via atualização; notificação e aceite passaram.
- `tests/friends-offline.browser.cjs`: indisponibilidade social preserva P2P e descoberta LAN.

As verificações de navegador usam a interface real com uma ponte nativa de testes. Os perfis LuxPlayer, Alex, Explorer, Builder e Miner são dados de demonstração; esses testes não representam uma sessão multiplayer entre dois computadores.

Capturas: `friends-redesign-1600-dark-empty.png`, `friends-redesign-1600-dark-list.png`, `friends-redesign-1600-wallpaper-empty.png`, `friends-redesign-960-wallpaper-list.png`.
