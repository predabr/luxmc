# Luxmc — vitrine de modpacks

Revisão visual local de 10 de outubro de 2026. Versão mantida em 3.7.0.

O cabeçalho com uma arte fixa foi substituído por uma vitrine interativa. Ela mostra até três modpacks dos resultados atuais, com capa, descrição, autor e quantidade de downloads. A seleção é manual; não há rotação automática. O botão Explorar modpack abre os detalhes do projeto escolhido.

Os cards agora têm capas maiores, etiquetas legíveis sobre as imagens, versão do Minecraft na própria capa e ação de instalação separada. O catálogo usa três colunas quando há espaço. Os filtros avançados ficam recolhidos inicialmente. Atalhos de exploração, magia, otimização, multiplayer e tecnologia aplicam os filtros reais de categoria. A vitrine sai da tela durante uma busca textual para dar espaço aos resultados.

As transições respeitam movimento reduzido e economia. Busca, paginação, virtualização, seleção de versão e instalação continuam usando os fluxos existentes. O site, as outras telas e o personalizador não receberam alterações nesta revisão.

## Verificações

- `pnpm check`: zero erros e zero avisos.
- `pnpm build`: compilação de produção concluída.
- Onze testes existentes de capas e correspondência de projetos passaram.
- Teste da vitrine: seleção do destaque e abertura dos detalhes; filtros de provedor, categoria e loader; botões de instalação sem corte; busca textual; temas claro/escuro; larguras de 1600, 1080, 800 e 640 px; movimento reduzido. Passou.
- Regressão do catálogo: rolagem, virtualização, resposta de busca atrasada, paginação, instalação pelo teclado e cancelamento. Passou.
- Regressão de imagem sem galeria, acesso à biblioteca, retorno à sala e contador de jogadores do Discord. Passou.
- `git diff --check`: sem problemas.

As capturas usam dados de demonstração, com as imagens reais que já existem no projeto. Nenhuma instalação de jogo ou alteração de conta foi necessária.

## Prévias

- [Vitrine — escuro](catalog-vitrine/vitrine-dark.png)
- [Cards — escuro](catalog-vitrine/cards-dark.png)
- [Vitrine — claro](catalog-vitrine/vitrine-light.png)
- [Janela compacta](catalog-vitrine/vitrine-compact.png)

Alterações locais: sem publicação no GitHub/site e sem abrir o launcher instalado.
