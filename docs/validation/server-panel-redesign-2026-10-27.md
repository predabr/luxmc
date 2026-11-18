# Reforma do painel de servidores — 27/10/2026

## Escopo aprovado

O mantenedor interrompeu a abordagem com Playit e pediu um painel mais simples, exclusão de servidores, correção da importação de instâncias e visualização efetiva dos conteúdos. A referência visual examinada foi a página pública da [EnxadaHost](https://enxadahost.com/), sem copiar sua identidade ou acessar painel privado. A revisão local `40255548682.904` foi instalada e reaberta em 27/10/2026, após o mantenedor confirmar que encerrou o launcher e os servidores.

## Resultado implementado

- Navegação lateral com visão geral, conteúdos, jogadores, console, configurações e backups. A visão geral reúne os controles de conexão e salvamento; o botão principal usa cor sólida e contraste do tema.
- Exclusão exige servidor parado e confirmação pelo nome exato. Os arquivos saem da lista ativa e ficam em recuperação, com restauração acessível no próprio painel. Não há exclusão definitiva automática de mundos.
- O navegador de arquivos mostra pastas e tamanhos reais, incluindo o mundo existente. A API recusa travessia de diretórios, links e pastas internas com credenciais do launcher.
- O relatório de importação separa o que foi preparado dos mods exclusivos do cliente. O original permanece na instância. A ausência de declaração de ambiente não prova compatibilidade de qualquer mod com servidor.
- Metadados com caracteres de controle dentro de strings são lidos de forma compatível, sem regravar os JARs. O bootstrap incorporado do Essential é reconhecido pela declaração de ambiente no manifesto interno. O limite de entradas de um JAR foi ajustado para aceitar o Pixelmon real, mantendo validações de caminhos e tamanho.
- O fluxo de vínculo e conexão Playit foi retirado. Pedidos de vínculo ou conexão pela API antiga são recusados; a parada de um agente legado continua disponível. A rede P2P integrada permanece separada do endereço público para clientes Minecraft padrão.

## Validação

`pnpm check` terminou sem erros ou avisos; 231 testes de frontend, 337 testes nativos e 143 testes do site passaram. A suíte nativa mantém 14 auxiliares ignorados. O navegador de produção passou em 1440×900, 1366×768 e 960×600, incluindo confirmação de exclusão, restauração, arquivos, mundo, moderação, configurações e ausência do cadastro Playit. A conferência foi repetida com os arquivos finais de interface, incluindo o token de contraste da ação principal. A suíte nativa foi repetida após o ajuste que mantém o serviço aberto enquanto existir uma operação em andamento.

Duas inspeções adicionais usaram arquivos instalados reais e destino temporário: Pixelmon preparou 706 arquivos e excluiu o Essential; a pasta contendo o Entity Model Features com JSON não padronizado preparou 237 arquivos e excluiu 46 mods declarados de cliente. Nenhum Minecraft ou servidor Java real foi iniciado nessas inspeções. As partidas e a compatibilidade completa dos modpacks dependem de teste em jogo.

## Preços publicados

Plus passou a R$ 30,25 e Pro a R$ 43,95 por 30 dias, correspondendo a US$ 5,50 e US$ 7,99, respectivamente, pela PTAX de venda de R$ 5,5007 em 26/10/2026. [Fonte do Banco Central](https://ptax.bcb.gov.br/ptax_internet/consultarTodasAsMoedas.do?method=consultaTodasMoedas). São valores fixados para novos pedidos, sem conversão dinâmica no momento do pagamento. Um teste garante que um pedido antigo mantém a quantia já apresentada ao comprador.

A publicação `https://56c30cc1.luxmc-r92.pages.dev` foi concluída e os valores foram conferidos na página pública de planos. Não foram criados pedidos reais nem alteradas assinaturas existentes nessa validação.

## Dependência de infraestrutura

O mantenedor confirmou possuir somente o site atual. Cloudflare Pages atende o site e suas funções HTTP, sem fornecer o ingresso TCP necessário para um endereço público universal de Minecraft. Não foi contratado domínio, VPS ou relay. O painel informa essa pendência; nenhum endereço fictício foi apresentado como operacional. O P2P atual requer Luxmc nos participantes. O PC anfitrião precisa permanecer ligado para hospedar localmente.

Os logs, capturas, hashes, recibos e backups ficam na pasta privada `%LOCALAPPDATA%/Luxmc-build/requests/local-server-20261026/`, com prefixos `redesign-` e `pricing-`. Nenhum commit ou push foi realizado nesta reforma.

## Entrega instalada

O instalador terminou com código zero às 12:41. A conferência do executável instalado confirmou o payload NSIS; 17 tabelas SQLite e 58 arquivos protegidos permaneceram preservados. O backup anterior à instalação é `backup-20261027-124144`, na pasta privada de recibos. O instalador também está em `%USERPROFILE%/Downloads/Luxmc 4.0 - novo painel de servidores.exe`.

A janela Luxmc reaberta respondeu à consulta do sistema. O teste de comunicação com o serviço separado da revisão 40255548682904 passou para listagem de servidores, recuperação e retirada do fluxo externo. Nenhum Java ou Minecraft foi iniciado.

A preparação opcional do Pixelmon existente foi interrompida antes da importação porque seu identificador já estava em `.luxmc-deleted`, fora da lista ativa. Nenhum servidor foi restaurado ou recriado automaticamente; a inspeção anterior em destino temporário permanece como evidência da cópia de conteúdos. O teste de partida real continua pendente do mantenedor.
