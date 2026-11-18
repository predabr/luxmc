# Revisão visual do site LuxMC — 4 de outubro de 2026

Esta revisão substitui a direção visual anterior, rejeitada pelo usuário. A implementação atual usa grafite, dourado da marca, tipografia condensada e capturas reais do launcher. O portal voxel, os cubos, as cenas WebGL decorativas da home e o diagrama de jogadores foram removidos. O visualizador funcional de skins permanece no Skin Studio.

A home recebeu uma composição tipográfica com o produto em destaque. Os botões têm uma hierarquia comum, com ícones circulares nas ações principais, estados de foco e controles de toque maiores. As capturas do produto mudam por seleção explícita. Os downloads usam linhas por sistema e destacam a plataforma detectada. Conta, cadastro, recuperação, dashboard, estúdio, páginas legais e 404 seguem a mesma identidade.

## Verificação

- Build do site concluído: pnpm build:website.
- 26 testes do site passaram, sem falhas.
- 56 estados de página verificados em 320, 375, 430, 768, 1024, 1440 e 1920 px, sem overflow horizontal ou conteúdo cortado nos títulos e controles verificados.
- 21 combinações de idioma e largura na home verificadas: português, inglês e espanhol.
- Navegador: seleção e ampliação de capturas, pesquisa real de Sodium com Fabric e Minecraft 1.21.1, abertura e fechamento do menu mobile, modo de cadastro e preset Alex com modelo slim e rotação.
- 17 arquivos de Functions, bibliotecas do servidor e migrações continuam idênticos ao backup anterior a esta revisão; 0 alterados.
- A home não importa Three.js nem o motor de skins. Seu script de movimento tem 2729 bytes. A fonte de display local tem 22444 bytes e licença OFL incluída.

## Entrega

Prévia local: http://127.0.0.1:8790/. Reiniciar com pnpm preview:website. Capturas em docs/visual/2026-10-04-website-revision/. Dados completos em website-visual-revision-2026-10-04.json. Publicado no Cloudflare Pages em https://luxmc-r92.pages.dev/; deployment https://400f332e.luxmc-r92.pages.dev/. Publicação limitada ao site, sem launcher ou novos releases. Verificação em website-publication-2026-10-04.json.
