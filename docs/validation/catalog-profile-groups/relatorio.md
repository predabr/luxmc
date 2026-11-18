# Luxmc — capas, perfis, grupos e migração de fontes

Revisão local de 07/10/2026. O nome Luxmc e a versão 3.1.0 foram mantidos. A instalação existente no PC, o download do site e a release do GitHub continuam com seus arquivos anteriores.

## Implementação

| Área | Alteração |
| --- | --- |
| Capas do catálogo | A apresentação reconhece imagens quadradas e verticais para usar a imagem inteira, com fundo derivado da própria arte. Uma capa de outro projeto já não é escolhida apenas porque o título do pack contém palavras de um pack conhecido: a escolha de arte curada exige identidade exata do slug. As logos da biblioteca usam `object-contain`. |
| Primeira preparação | A instalação finaliza a recuperação e a verificação estrutural e grava o estado verificado da instância. O primeiro lançamento pode reaproveitar esse estado se os arquivos não mudarem. O pedido de verificação completa continua sendo respeitado. |
| Bibliotecas dos loaders | Fabric, Quilt, Forge e NeoForge podem preparar até seis bibliotecas simultaneamente. Mantêm a ordem do classpath, os hashes fornecidos, os fallbacks de Maven e a conclusão dos downloads antes de iniciar o jogo. Destinos repetidos são agrupados. Arquivos de override também recebem uma fila limitada de verificação/recuperação. |
| Perfil público | Banner maior, retrato, identidade, nome de exibição, nome da conta, status, bio e recomendações. Nome de exibição tem limite de 32 caracteres; status, 80; bio, 400. A mudança de nome visual não troca o nickname de autenticação do Minecraft. Ao trocar de perfil, os dados do perfil anterior são limpos durante o carregamento. |
| GIF | Banner e retrato aceitam GIF de até 700 KB. O arquivo permanece como GIF, preservando os bytes e a animação, e a prévia informa o formato. PNG/JPEG/WebP continuam sendo reduzidos para limitar o tamanho. O limite anterior era 190 KB. Clientes antigos podem salvar os campos que conhecem sem apagar o novo nome de exibição e status. |
| Grupos | A biblioteca inicial mostra áreas de grupo que recebem instâncias arrastadas. Existe também um seletor com o mesmo estilo dos botões do launcher para mover usando mouse ou teclado. Novos grupos são lembrados localmente e a associação de cada instância é salva no banco do launcher. |
| Separação de arquivos | Agrupar modifica somente a associação visual da instância. `gameDir`, mods, saves, configurações e demais arquivos continuam pertencendo à instância original. |
| Botões | Os controles novos usam os componentes e tokens existentes. Upload, remoção de imagem e escolha de grupo têm estados de foco e bloqueio durante operações. O seletor de grupo evita abrir a instância ao clicar em um controle interno. |
| Site | Refinamentos na composição da abertura, moldura da captura, cartões de recursos e downloads. Logos do catálogo são mostradas inteiras. Os efeitos respeitam a preferência de redução de movimento. |

## Linguagens

- Rust, Svelte 5 e TypeScript continuam no launcher.
- O auxiliar C++ foi substituído por `native_rust.rs`; foram removidos o build C++ e a dependência direta de `cc`. Diagnóstico por assinaturas, identificação de CPU e funções de memória mantêm os contratos usados pelo restante do launcher. Windows e Linux preservam a leitura do pico de memória do sistema operacional.
- As fontes próprias das APIs e utilitários do servidor estão em `site-server/`, em TypeScript.
- As fontes próprias das páginas, catálogo, traduções e prévia de skins estão em `site-client/`, em TypeScript.
- O site continua servindo JavaScript gerado, como exige o navegador. `pnpm build:website` compila essas fontes para os mesmos caminhos públicos.
- A checagem de tipos dos dois projetos está em modo estrito. Algumas respostas externas e estruturas dinâmicas ainda usam `any`; isso está registrado no guia de manutenção. Validadores de mídia e contratos de requisição/DOM têm tipos explícitos.
- Java e Python permanecem nas funções atuais.

TypeScript ajuda a detectar erros de integração; a troca não promete execução mais rápida no navegador. A remoção do auxiliar C++ simplifica a compilação. O ganho de abertura precisa ser medido em packs reais depois dos testes de regressão.

## Verificação concluída

- Checagem do frontend: zero erros e zero avisos.
- `cargo check`: aprovado; cinco avisos de código não utilizado já existentes.
- TypeScript do servidor e das páginas: aprovado em modo estrito.
- Suíte do site após a migração: 42 testes aprovados.
- Regressão final dos perfis e limites de mídia: 13 testes aprovados, incluindo preservação de nome/status ao salvar com cliente antigo.
- Arte do catálogo e banners existentes: 13 testes aprovados em um único worker.
- A checagem de tipos identificou caminhos de conta removida e elementos DOM ausentes; foram acrescentadas verificações de recuperação.

## Testes adicionais autorizados em 07/10/2026

- Frontend completo: 24 arquivos, 133 testes aprovados.
- Backend: 141 testes aprovados, zero falhas, dois ignorados por dependerem do endpoint público do NameMC e de infraestrutura pública de relay.
- Compilação de produção do frontend: aprovada; saída estática em `build/`.
- `cargo check` final: aprovado, com os cinco avisos de código não utilizado existentes.
- Grupos e perfis: aprovado nos temas claro e escuro. Arrastar/soltar e seletor persistem após recarregar; os caminhos das instâncias permanecem separados; nome/status e bytes do GIF são preservados.
- Amigos e downloads: aprovado nos dois temas. Perfil desconectado, edição, carregamento de ícones visíveis e instalação persistente ao trocar de aba.
- Menu da página inicial e atualizador: itens cabem nos botões; notas Markdown renderizam; HTML inseguro é removido.
- Site: aprovado nas larguras de 1440 e 390 pixels, sem erros de página ou transbordamento horizontal; tour e chamada de download funcionam.
- Fabric e Quilt: preparação real aprovada; cache corrompido recuperado.
- Integrações de loaders e versões: 11 testes aprovados, zero falhas. Fabric 1.21.4 com mod e Fabric API; Quilt com recuperação de cache e preparação sem rede; Forge 1.12.2 e 1.20.1; NeoForge 1.20.4 e 1.21.1 com JARs gerados válidos; metadados/argumentos/Java de Vanilla 1.8.9, 1.16.5, 1.21.4 e beta 1.7.3. Esses casos não iniciam uma sessão jogável do Minecraft.
- Quilt 1.20.1 / loader 0.28.1: download e verificação de 11 bibliotecas em 2,687 segundos; segunda preparação em 216 milissegundos com cliente HTTP apontando para proxy indisponível. Medição de preparação do loader neste PC, não tempo de abertura do Minecraft ou de um modpack completo.

Os testes usam demonstrações e diretórios temporários. A navegação inicial do teste de grupos passou a aguardar o DOM, evitando depender do carregamento de mídia para iniciar as verificações.

## Validação que exige ambiente adicional

- Medir abertura até o menu e instalação fria em packs reais representativos.
- Gerar e conferir o novo instalador Windows. O instalador `Luxmc-3.1.0-workshop.exe` da revisão anterior ainda não contém estas mudanças.
- Conferir o P2P entre PCs e redes diferentes quando os participantes estiverem disponíveis. Testes locais de túnel, mudança de mundo e anúncios não comprovam autenticação ou conectividade em todas as redes.

As propostas de próximos diferenciais e melhorias estão em `propostas.md`; elas não foram implementadas nesta revisão.

## Ordem para publicação futura

Antes de publicar o backend dos perfis, aplicar `website/migrations/0009_profile_identity.sql`. Ela acrescenta os campos de nome e status com valores vazios para perfis existentes. Publicar o backend antes de disponibilizar o novo instalador, para que o limite de GIF e os novos campos sejam aceitos pelo servidor.

O site mantém a metatag do Search Console já publicada. Esta revisão não alterou a versão, não substituiu os downloads públicos e não fez push ou release.
