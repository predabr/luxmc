# Obsidian · revisão visual

A chave CurseForge original foi restaurada como fallback e validada contra a API real. O tema padrão agora usa azul diamante; a preferência de cor já salva continua respeitada. A paleta extraída das capas ilumina o fundo, sem alterar a cor dos controles.

## Telas renderizadas

As capturas do launcher usam um perfil temporário com instâncias, mods e amigos demonstrativos. São a interface Svelte real, renderizada em 1440 × 1000; não são testes de jogo ou presença entre máquinas.

- [Biblioteca em grid](visual/instances.png): banners, loaders, tempo jogado, RAM, mods e botão Jogar.
- [Biblioteca em lista](visual/instances-list.png): informações compactas e as mesmas ações do grid.
- [Detalhe da instância](visual/instance.png): hero, conteúdo instalado, atalhos preservados e sidebar social.
- [Configurações integradas](visual/instance-settings.png): RAM mínima/máxima, Java e presets JVM; opções avançadas continuam disponíveis no modal.
- [Central de conteúdo](visual/mods.png): banner oficial ou iluminação derivada da logo, ícone flutuante, downloads e ação de instalação.
- [Amigos](visual/friends.png): cabeças reais, status, favoritos e entrada em mundo compartilhado.
- [Portal desktop](visual/website-desktop.png): hero, mockup com abas, release e tamanho do download.

- [Portal mobile](visual/website-mobile.png): botões, tipografia e mockup adaptados a 390 px.

## Revisão adicional

- [Início](visual/home.png), [configurações](visual/settings.png), [estúdio de skins](visual/skins.png), [logs](visual/logs.png), [histórico](visual/logs-history.png), [notícias](visual/news.png) e [screenshots](visual/screenshots.png).
- Tema claro: [instâncias](visual/instances-light.png), [configurações](visual/settings-light.png) e [amigos](visual/friends-light.png).
- Modais mantêm o foco no diálogo ativo e o devolvem à ação de abertura. A lista virtual recalcula a área visível ao redimensionar ou filtrar. O estúdio preserva suas ações quando WebGL está indisponível.

## Comportamentos preservados

Importação .zip/.mrpack, migração de launchers, seleção em lote, favoritos, edição, duplicação, notas, diagnóstico, exclusão explícita, exportação, backups, arquivos, mundos, galeria, contatos locais e opções de tema continuam acessíveis. Os cards usam botões de detalhe e ações separados para permitir navegação por teclado.

Os botões compartilhados têm estados de foco, loading, disabled e pressão; as superfícies, sombras e cores derivam dos tokens. As animações respeitam a preferência de movimento reduzido.

Os comandos e limites de validação estão em [VERIFICACAO.md](VERIFICACAO.md). A API social permanece preparada para ativação futura em Cloudflare D1 conforme [PUBLICACAO.md](PUBLICACAO.md).
