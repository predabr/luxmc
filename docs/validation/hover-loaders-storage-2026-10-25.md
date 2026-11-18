# Hover, loaders antigos e armazenamento — 25/10/2026

## Causas e alterações

O Windows usa movimento reduzido, mas o usuário configurou animações expressivas e desativou o respeito a essa preferência no launcher. O hover dos cartões estava dentro de uma consulta de mídia que impedia a animação mesmo com essa escolha explícita. Os cartões de início, instâncias e pastas agora compartilham elevação curta, borda e sombra no hover, com resposta de foco para teclado. Ícones do início recebem movimento leve; banners de paisagem recebem aproximação discreta. Ícones quadrados usados como fundo não recebem zoom. Animações desativadas, modo de desempenho e movimento reduzido respeitado continuam interrompendo o movimento.

Legacy Fabric e Babric não suportam Minecraft 26.3 nas listas oficiais consultadas. O endpoint de Legacy Fabric respondia HTTP 400 para essa combinação, exibido como erro bruto. O backend consulta e conserva por cinco minutos a lista de versões do jogo declarada pelo provedor antes de consultar as versões do loader. A criação de instância oferece somente versões declaradas; o seletor inclui as versões Beta quando apropriado. A incompatibilidade mostra orientação e opções de jogo válidas. Falhas reais de consulta mantêm uma ação de nova tentativa. O editor de instância informa a incompatibilidade e valida alterações de loader antes de gravar, sem trocar a versão do jogo ou dos mundos automaticamente.

Referências: [API Legacy Fabric](https://meta.legacyfabric.net/), [versões declaradas pelo Legacy Fabric](https://meta.legacyfabric.net/v2/versions/game), [Babric](https://babric.github.io/), [versões declaradas pelo Babric](https://meta.babric.glass-launcher.net/v2/versions/game) e [especificação do Fabric Meta](https://github.com/FabricMC/fabric-meta/blob/master/README.md).

## Armazenamento observado

Medição de leitura no computador do autor: aproximadamente 6,83 GiB de instâncias, 2,40 GiB na pasta de mods, 1,27 GiB de recursos do Minecraft, 0,69 GiB de bibliotecas, 0,42 GiB de Java e 0,79 GiB de cache de modpacks. O executável instalado ocupa aproximadamente 42 MiB. Os valores são desta medição, variam com o uso e não incluem as ferramentas de desenvolvimento.

A tela separa aplicativo, conteúdo dos jogos e cache recriável, ordena as categorias por tamanho e explica a diferença entre esses dados. Instalações concluídas pela fila descartam o pacote temporário ZIP/MRPACK somente depois de importar e vincular a instância. A limpeza é limitada ao arquivo de pacote dentro do cache; falhas e pausas preservam o download para retomada. O cache antigo continua acessível pela limpeza existente, sem apagar mundos, mods instalados ou Java. Nenhum arquivo do usuário foi apagado durante a análise.

## Planos no site

Os textos em português, inglês e espanhol descrevem funções implementadas: cores e bordas do avatar, GIFs, fundo dos cartões de amigos, favoritos com capas, configurações visuais salvas e capacidade das salas. Nomes de efeitos foram substituídos por descrições visíveis; os limites e preços não mudaram. As imagens fornecidas pelo usuário permanecem no site.

## Validação

Checagem Svelte: zero erros e avisos. Checagem Rust aprovada com seis avisos preexistentes. Doze verificações de benefícios e idiomas do site aprovadas. As regressões de consulta de metadados e isolamento da limpeza de pacotes passaram, usando somente dados de exemplo e arquivos temporários.

Build local 4.0.0, identificação 40133561068.1, compilada e instalada depois do encerramento confirmado pelo usuário. Instalador terminou com código zero; comparação do binário instalado aprovada. Dez perfis, duas contas, arquivos protegidos e wallpapers preservados na comparação anterior e posterior. O launcher foi reaberto para o teste do usuário. Nenhuma partida de Minecraft foi executada pelo agente. A revisão nativa permanece local; os textos dos planos, README e ajustes de apresentação foram publicados no site e no GitHub.
