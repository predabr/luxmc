# Ajustes do launcher e do site — 7 de outubro de 2026

A versão continua **3.1.0** e o nome continua **Luxmc**. O redesign do site e o serviço de perfis foram publicados. As alterações novas do launcher estão em uma compilação local de revisão; o instalador público original 3.1.0 foi preservado para o teste do atualizador. O aplicativo instalado no PC não foi atualizado por esta tarefa.

## Notificações

- Cartões com bordas discretas, indicador lateral da categoria, ícone e fechamento compacto.
- Texto legível nos temas claro e escuro, com espaço para mensagens longas e ações.
- Mensagens idênticas em exibição não se acumulam novamente.
- Erros e notificações com ação ficam disponíveis por dez segundos; mensagens simples por 4,5 segundos.
- Erros usam o anúncio acessível `alert`; as demais mensagens usam `status`.
- Animações respeitam a preferência do sistema por movimento reduzido.

## Biblioteca de mods, shaders, resource packs e datapacks

- A primeira listagem retorna os nomes e metadados antes de abrir os arquivos compactados.
- As imagens são solicitadas conforme os itens se aproximam da área visível, em lotes de até 32 arquivos.
- Imagens extraídas ficam em cache no disco. A listagem evita transmitir todas as imagens em base64 de uma biblioteca grande; recupera as necessárias sob demanda.
- O cache identifica caminho, tamanho e data de alteração do arquivo. Alterar `pack.png` de uma pasta também invalida a imagem anterior.
- A busca de logos dentro de qualquer dependência do arquivo foi removida. Arquivos `.jar` não usam o `pack.png` genérico de Minecraft como logo do mod.
- Quando não há imagem interna, a consulta ao Modrinth usa o hash do arquivo. Imagens do CurseForge usam a identidade registrada e o hash correspondente.
- Duas cópias do mesmo conteúdo podem receber a mesma imagem correta; nomes iguais em instâncias diferentes permanecem independentes.
- Um conteúdo sem imagem própria recebe um marcador discreto. Nenhuma imagem aleatória é inventada para ele.
- A troca rápida de mundo na seção de datapacks ignora respostas do mundo anterior.

## Instalação de modpacks

- Estado e eventos da instalação passaram para um controlador global. Trocar de página preserva a operação e seu painel.
- O painel pode ser recolhido, mostra progresso e mantém o cancelamento acessível.
- Ao terminar, oferece abrir a instância. Em caso de falha, mantém o diagnóstico disponível para leitura.
- Clientes HTTP de downloads são reutilizados, permitindo reaproveitar conexões.
- Metadados do CurseForge são obtidos em paralelo.
- A consulta alternativa ao Modrinth ocorre após uma tentativa de download principal, em vez de atrasar cada arquivo antes do download.
- O processamento de arquivos de override usa concorrência limitada e calcula hashes fora da execução assíncrona principal.
- Importações novas deixam os registros dos mods para a transação final, evitando tentativas de registrar arquivos antes de existir a instância.
- Verificação de hashes, escrita atômica, limites de extração, compatibilidade e cancelamento foram mantidos.

Essas alterações removem trabalho e esperas desnecessárias. O tempo final ainda depende do tamanho do modpack, disco, conexão e disponibilidade dos provedores. Não foi prometida uma porcentagem de aceleração sem medição equivalente em dois computadores.

## Amigos e perfil público

- A página recebeu cartões com melhor contraste, hierarquia e organização responsiva. O fundo personalizado continua visível ao redor da interface.
- Clicar no nome ou avatar de um amigo abre seu perfil, incluindo quando ele está desconectado.
- “Editar meu perfil” permite descrição opcional, imagem de fundo, imagem de perfil e até oito mods ou modpacks favoritos.
- Imagens estáticas são ajustadas automaticamente. GIFs animados de até 190 KB são aceitos.
- Descrição e favoritos vazios não criam seções vazias no perfil.
- O compartilhamento depende do que a pessoa escolhe salvar; a lista privada de arquivos instalados não é enviada automaticamente.
- As informações persistem no portal. Apenas amigos aceitos podem consultar o perfil; bloqueios e remoções revogam esse acesso.
- A barra lateral manteve a estrutura existente e ganhou acesso ao perfil público e às salas, além do clique nos perfis dos amigos.

## Menu de “Jogar agora”, login e atualização

- O menu dos três pontinhos recebeu largura adequada, espaçamento, fundo legível e ações alinhadas.
- A entrada principal e a janela de adicionar conta distinguem Microsoft, Luxmc e perfil local.
- Os fluxos existentes de autenticação foram preservados. A janela de conta ganhou contenção do foco e identificação acessível.
- A janela de atualização ficou mais compacta. As novidades são apresentadas como texto formatado, com HTML inseguro removido.
- Progresso, erros, nova tentativa e versões instaladas/disponíveis têm apresentação clara.
- As verificações deixam de reutilizar respostas do cache do navegador.
- O seletor de instalador reconhece também `Lux.MC.Launcher.exe`, nome normalizado pelo GitHub, mantendo a rejeição do executável portátil simples.

## Site publicado

- Endereço: https://luxmc-r92.pages.dev/
- Implantação: https://1d144617.luxmc-r92.pages.dev
- Nova apresentação com azul, contraste entre áreas claras e escuras, captura real do launcher e chamada de download direta.
- Navegação, seleção dos recursos, catálogo, área da conta e cartões de download receberam o mesmo acabamento visual.
- Desktop e celular foram verificados, incluindo as páginas inicial e de conta.
- A migração `0008_public_profiles.sql` criou apenas a tabela adicional para os perfis.
- O serviço de perfis exige autenticação e autorização de amizade.
- O download Windows público foi verificado integralmente. Continua com 20.479.497 bytes e SHA-256 `2dc2228e8c595e06a8042e88ee193fc9a146230f1a287d9fe16fe4c3346231aa`.

## Verificação

- `pnpm check`: zero erros e zero avisos.
- Vitest: 22 arquivos, 129 testes aprovados.
- `cargo check`: aprovado; permanecem cinco avisos já existentes de itens não utilizados.
- Rust: 133 testes aprovados e dois testes externos ignorados na execução completa; os testes de conteúdos incluem rejeição de logos de dependências e cache de ausência.
- Portal/site: 40 testes aprovados, incluindo persistência de perfis, amizade necessária, bloqueio e rejeição de imagens inválidas.
- Navegação: instalação preservada ao ir para Personalização; conclusão acessível; perfis offline; salvamento do perfil; solicitação limitada de imagens em uma lista com mil itens; temas claro/escuro.
- Instâncias: listas com mil mods, busca, último item, ativação/desativação e configuração integrada em larguras de 1440 e 960 pixels.
- Amigos: estados vazio e preenchido, favoritos, busca, convites, cópia do código e fundo personalizado em desktop e janela estreita.
- Login: aparência e passagem pelo manipulador real da interface com credenciais fictícias em 1440 e 960 pixels.
- Atualizador: escolha do instalador, tratamento de jogo aberto, nova tentativa, apresentação das novidades e remoção de HTML inseguro.
- Consulta pública real: versão simulada 3.0.2 detectou 3.1.0 e selecionou um instalador publicado, sem executar instalação.

Os testes de interface usam dados e respostas controlados; a autenticação Microsoft não foi refeita com credenciais reais. Os dois testes de rede externa ignorados não constituem validação universal de P2P em qualquer conexão.

## Evidências visuais

- `friends-dark.png` e `friends-light.png`: página dos amigos.
- `profile-dark.png` e `profile-light.png`: perfil de um amigo desconectado.
- `download-dark.png` e `download-light.png`: instalação visível fora do catálogo.
- `content-dark.png` e `content-light.png`: biblioteca com imagens carregadas sob demanda.
- `login-1440.png` e `login-960.png`: login.
- `home-menu.png`: menu da instância.
- `updater.png`: nova janela de atualização.
- `../site-revision/hero-1440.png` e `../site-revision/hero-390.png`: site.

A compilação do instalador local e seu checksum estão registrados em `artifact.json` após a conclusão do empacotamento. Nenhuma nova tag ou release foi criada para estes ajustes.
