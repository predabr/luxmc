# Ajustes finais — Luxmc 3.1.0

O nome Luxmc permanece igual. A pedido do usuário, esta revisão passa a ser a versão 3.1.0 e será publicada no site e no GitHub. A instalação 3.0.2 do PC do usuário não será modificada, para permitir testar o atualizador.

## Mudanças

- Criação de instâncias com janela mais larga, edição de ícone em uma faixa organizada, nome com mais espaço e loaders alinhados. O rodapé permanece acessível durante a rolagem.
- Instalação de modpacks com título completo, cabeçalho legível, controles de memória agrupados e área de conteúdo que cabe em janelas menores.
- Logos do catálogo fora da área que recorta o banner. Pacotes que só têm ícone usam a imagem inteira, sem corte nem gradiente por cima. Títulos podem ocupar duas linhas; a altura dos cartões preserva descrição, autor e botão.
- Recuperação de logos de instâncias com ícone padrão, consultando os catálogos por título exato e salvando o ícone no perfil. Ícones personalizados e instâncias Vanilla são preservados. Sem correspondência segura, o ícone não é substituído.
- Mais linhas preparadas antes de entrarem na tela, imagens carregadas conforme a proximidade da área visível e isolamento da pintura dos cartões. A rolagem continua nativa, sem interceptação do movimento da roda.
- Filtros mais largos, nomes de conteúdo sem truncamento e indicação acessível da opção selecionada.
- Após importar uma skin do NameMC, um aviso informa que as alterações ainda não foram aplicadas e orienta a clicar em Aplicar e sincronizar. A barra mantém o aviso enquanto houver mudanças pendentes.
- Shaders e datapacks sem linha atravessando os cantos arredondados. Cabeçalhos de seções vazias não desenham uma separação sem conteúdo abaixo.
- Removida a opção de instalar pacotes de mods de performance das configurações e da criação de instâncias. RAM, Java, argumentos e demais otimizações permanecem disponíveis. Mods já instalados não são removidos.
- Nova notícia interna sobre estes ajustes, com textos em português, inglês e espanhol.

## Verificação

- `pnpm check`: zero erros e zero avisos.
- Testes unitários: 22 arquivos, 128 testes aprovados, incluindo recuperação de logos, preservação de ícone escolhido durante uma consulta e nova tentativa após falha de rede.
- Criação de instâncias: temas claro e escuro, com e sem wallpaper; seleção legível, criação Vanilla e janela reduzida.
- Catálogo: logos inteiras e separadas do título, rodapé dentro do cartão, 36 resultados com renderização limitada, roda do mouse percorrendo 1.600 pixels e instalador acessível em 960 × 720.
- Catálogo nativo real: Vulkan Optimized encontrado no Modrinth, projeto `u76hn1pF`, com sua logo oficial. A recuperação não depende de uma imagem inventada ou de um nome de arquivo local.
- Filtro selecionado: contraste mínimo de 4,5:1 verificado nos temas claro e escuro.
- NameMC: importação preserva a coleção, evita duplicatas, mostra aviso persistente e não aplica a skin sem clicar no botão.
- Conteúdos: instalação compatível de mods, resource packs, shaders e datapacks, escolha do mundo para datapacks e tratamento de incompatibilidade.
- Instância com mil mods: navegação, busca, operações em lote, configurações, recuperação de erro ao salvar e ferramentas verificadas em 1440 e 960 pixels.
- Notícias oficiais: atualização automática, retenção sem conexão e recuperação ao reconectar.

As capturas e os resultados da revisão visual estão em `docs/validation/final-polish/` e `docs/validation/final-polish-ui.json`. A validação de rolagem confirma funcionamento e quantidade limitada de elementos; não constitui um benchmark de FPS em todos os computadores.

A recuperação automática depende de o catálogo fornecer uma correspondência exata. Pacotes importados com um nome personalizado e sem identificação de origem podem precisar de um ícone escolhido manualmente.


## Imagens dos conteúdos e recuperação

- Mods e packs usam miniaturas próprias quando disponíveis, com decodificação limitada, carregamento lazy e cache em memória/disco. Downloads do catálogo preservam a URL da imagem do projeto. A chave de cache inclui caminho, tamanho e data do arquivo; nomes iguais em instâncias diferentes não reutilizam imagens de packs indevidamente.
- Resource packs e datapacks extraem pack.png de arquivos ZIP ou de pastas. Imagens inválidas e conteúdos sem imagem exibem o símbolo da categoria.
- Datapacks são listados na pasta saves/<mundo>/datapacks, com seleção do mundo e opções para abrir a pasta, importar ou excluir um arquivo do mundo escolhido.
- No Windows, os atalhos usam um verificador independente, que confere hashes SHA-256 antes de abrir o launcher. Uma cópia local válida pode reparar o executável e o desinstalador sem modificar contas, mundos ou o projeto.
- Na primeira instalação com este sistema, a cópia de recuperação é da própria versão instalada. Em atualizações futuras, a versão anterior válida é preservada. Se o verificador não puder executar ou nenhuma cópia for válida, é necessário reinstalar. A restauração não é uma garantia contra qualquer dano no computador.

Verificações adicionais locais: cargo check aprovado; 131 testes Rust aprovados e dois testes de serviços externos ignorados; oito testes de recuperação aprovados; 38 testes do site e nove testes de manifesto aprovados. A suíte Linux de instalação usa comandos Unix e será executada pelo runner Linux; não foi executada com sucesso no Windows por depender de links simbólicos e utilitários Unix.

A CI prepara Javas 8, 17 e 21 com a ação oficial [setup-java](https://github.com/actions/setup-java) para os testes isolados de loaders. Isso evita que a preparação do ambiente de testes dependa de consultas anônimas limitadas pelo GitHub.
