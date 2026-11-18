# Luxmc — catálogo e acabamento visual

10 de outubro de 2026. Versão 3.7.0 preservada. Alterações locais; nenhuma publicação, instalação ou execução do launcher nativo.

## Catálogo

Cabeçalho com arte local, seleção dos provedores e acesso à biblioteca. Navegação de conteúdo acima da busca, reunindo modpacks, mods, pacotes de recursos, shaders, datapacks e mundos. Filtros de versão, loader e categoria em painel próprio, recolhido inicialmente em janelas menores. Cards com capas mais visíveis, ícones separados da arte, títulos com espaço para duas linhas, bordas e estados de foco. Carregamento inicial com esqueletos de cards.

A lista conserva virtualização, paginação, busca com proteção contra respostas antigas, instalação e cancelamento. O ícone da visualização selecionada recupera contraste no tema claro. As preferências de movimento e economia continuam aplicadas.

## Outras telas

Acabamento comum de painéis, cabeçalhos, estados selecionados e cards na página inicial, instâncias, amigos, configurações, manutenção, notícias, capturas, logs, histórico, otimização, administração, Bedrock e sala. O personalizador não recebeu alterações nesta rodada. As rotas que apenas redirecionam conservam esse comportamento.

## Site

Hierarquia visual do início, capturas do launcher, tour de recursos, catálogo, downloads, perguntas frequentes e conta refinada. Cards do catálogo usam a galeria do Modrinth quando disponível e o ícone como alternativa. URLs de galeria aceitas somente por HTTPS no CDN do provedor. Falhas de imagem usam o logo local por um listener, sem exigir execução de eventos HTML inline ou alterar a política de segurança.

As capturas reais existentes continuam usadas pelo site. As imagens deste relatório têm dados de demonstração e não substituem fotos públicas do produto.

Referências de organização consultadas: [catálogo oficial do Modrinth](https://modrinth.com/modpacks) e [Prism Launcher](https://prismlauncher.org/).

## Verificação

- `pnpm check`: zero erros e zero avisos.
- `pnpm build`: produção do frontend compilada. Avisos anteriores sobre tamanho de chunks e importação de wallpaper continuam presentes.
- `pnpm build:website`: site compilado.
- `pnpm test:run`: 153 testes, 26 arquivos, todos passaram.
- `pnpm test:website`: 55 testes passaram, incluindo galeria, rejeição de URL inadequada e recuperação de imagem sem evento inline.
- Navegador do site: desktop, largura de 390 px, conta, preferências de movimento reduzido e fallback real de uma imagem que retorna 404 passaram.
- Navegador do launcher: as 18 suítes existentes passaram; as sete finais foram repetidas isoladamente depois que um recarregamento durante a compilação interrompeu a primeira execução. Cobertura de exclusão, cancelamento, salas, perfis, conteúdo, criação, temas, idiomas, Java e personalizador.
- Nova verificação visual do catálogo passou com temas claro/escuro, seis tipos de conteúdo, filtros, larguras de 1600, 1080, 900 e 700 px, além das telas inicial, instâncias, amigos, configurações, manutenção, notícias, capturas, logs, histórico e otimização. As capturas foram inspecionadas.
- `git diff --check`: nenhum erro de espaços. Git manteve a base e a versão.

## Capturas locais

- [Modpacks — escuro](design-catalog/modpacks-dark.png)
- [Modpacks — claro](design-catalog/modpacks-light.png)
- [Modpacks — janela compacta](design-catalog/modpacks-compact.png)
- [Instâncias](design-catalog/instances.png)
- [Amigos](design-catalog/friends.png)
- [Configurações](design-catalog/settings.png)
- [Site — desktop](design-catalog/site-desktop.png)
- [Site — catálogo](design-catalog/site-catalog.png)
- [Site — mobile](design-catalog/site-mobile.png)

Os testes de interface usam dados e comandos simulados. Esta rodada verifica o redesign e seus fluxos de interface; não comprova uma nova partida real, uma instalação real de modpack ou uma conexão P2P entre computadores.
