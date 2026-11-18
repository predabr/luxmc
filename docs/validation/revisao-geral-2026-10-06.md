# Luxmc — revisão geral de estabilidade, conteúdo e interface

Versão mantida: **3.0.2**. Nome do aplicativo: **Luxmc**. Nome do instalador: **Lux MC Launcher.exe**. Esta revisão não autoriza mudanças de versão nem publicação no GitHub.

## 1. P2P: reconexão, troca de mundo e erros

- A tela detecta a perda de uma sala de convidado e oferece **Reconectar à sala**, reutilizando o convite guardado em memória. Uma saída voluntária limpa esse convite.
- A descoberta LAN tenta outras portas locais disponíveis quando a porta do mundo anterior deixou de responder. Isso evita ficar preso ao mundo antigo enquanto existe um novo mundo aberto.
- O monitor mantém a publicação de mundos por participante e ignora os anúncios retransmitidos pelo próprio Luxmc para evitar duplicação.
- Erros de sala cheia, bloqueada, convite expirado ou participante removido preservam a causa retornada pelo backend. A tela tem espera limitada e opção de tentar novamente.
- A consulta da sala fica menos frequente com a janela oculta e após erros.

Validação: testes nativos de túnel criptografado, troca automática de mundo, mundo de convidado acessível aos outros participantes, retirada do anúncio e transferência pelo relay público. O teste de interface também simula perda da sala, reconecta com o mesmo convite e verifica retorno, saída e recuperação de consulta travada.

A correção anterior do erro **Invalid session** continua incluída. Ela corrigiu a busca da conexão do servidor integrado pelo agente. Um teste local com Minecraft 26.3 e uma cópia de mundo recebeu sucesso de login e terminou com código 0. Evidência: [teste de login e encerramento](C:/Users/preda/OneDrive/Documenti/Luxmc/docs/validation/p2p-login-shutdown-2026-10-06.md).

**Limite:** teste local e relay público não substituem uma partida completa entre os dois PCs de vocês, com a mesma versão e o mesmo conjunto de mods. Uma conexão Minecraft já derrubada exige voltar a entrar no mundo; reconectar a sala não recupera automaticamente uma conexão TCP encerrada. Servidores externos continuam seguindo sua autenticação oficial.

## 2. Loaders e modpacks

O teste nativo abriu Minecraft real, aguardou inicialização de áudio e atlas de texturas e confirmou que o processo continuava ativo. Os perfis temporários de teste foram removidos; mundos existentes não foram abertos pelo teste de matriz.

A matriz inclui Vanilla 1.8.9, Vanilla 26.3, Fabric 1.20.1, Quilt 1.20.1, Forge 1.20.1, NeoForge 1.21.1 e os seus modpacks **Vanilla Perfected** e **Vulkan Optimized**. Versões e resultados exatos estão em [evidência da matriz](C:/Users/preda/OneDrive/Documenti/Luxmc/docs/validation/general-launch-updated-2026-10-06.json).

O launcher já escolhe Java conforme os metadados da versão, conserva a versão fixada do loader de um modpack e reaproveita bibliotecas e natives preparados. A revisão também remove variáveis herdadas que poderiam injetar argumentos indevidos no Java: `JAVA_TOOL_OPTIONS`, `_JAVA_OPTIONS`, `JAVA_OPTIONS`, `JDK_JAVA_OPTIONS` e `CLASSPATH`.

**Limite:** esta matriz testa combinações específicas. Não comprova todas as versões históricas ou todos os modpacks publicados. O teste de Vulkan Optimized valida a abertura do pack com o backend padrão, sem forçar o modo Vulkan do launcher.

## 3. Preparação mais eficiente

A validação dos arquivos oficiais grava um recibo em disco contendo tamanho, data de modificação e hash esperado. Nos próximos inícios, um arquivo inalterado com recibo válido evita nova leitura integral para calcular seu hash. Se o arquivo mudar, a verificação volta a ocorrer — inclusive quando a alteração mantém o mesmo tamanho.

O teste de regressão modifica um arquivo sem mudar seu tamanho e confirma que o cache o rejeita. A preparação continua verificando os arquivos novos e os arquivos que perderam seu recibo.

Isso reduz trabalho repetido de disco; não elimina o tempo de inicialização dos próprios mods, renderização, shaders e Java. Os tempos da matriz de depuração foram coletados com outras verificações em andamento e não devem ser usados como comparação de desempenho com o instalador otimizado.

## 4. Skins, capas e wallpapers

- O carregamento de preferências consulta tanto o armazenamento local quanto as configurações nativas. Uma falha de um dos dois não impede aproveitar o outro.
- A conclusão de uma aplicação de skin compara a identidade canônica da conta. Uma normalização do ID durante a operação não descarta indevidamente a atualização visual.
- O botão Aplicar continua disponível para reaplicar uma aparência já selecionada, com bloqueio de chamadas concorrentes e nova tentativa de sincronização Microsoft.
- A importação de wallpaper mantém uma cópia gerenciada pelo launcher. A biblioteca e a seleção sobrevivem à exclusão do arquivo original.
- O teste de vídeo reproduz mídia real, simula inicialização atrasada do servidor local, limpa o cache visual e muda a porta após reiniciar. Também verifica recuperação usando as configurações nativas quando o armazenamento local falha.
- O pacote de skin usa o formato informado pelo `version.json` do arquivo oficial do Minecraft. Versões recentes usam `min_format` e `max_format`; versões antigas conservam `pack_format`.

O Minecraft mudou essas regras a partir dos novos formatos de pacote; declarar apenas campos antigos nas versões recentes pode fazer o jogo rejeitar o pacote. [Notas oficiais do Minecraft 1.21.9](https://www.minecraft.net/en-us/article/minecraft-java-edition-1-21-9).

**Limite:** visibilidade de capas importadas e skins de contas Luxmc depende do mecanismo de aparência do Luxmc nos participantes. Um resource pack sozinho não transforma uma conta local em perfil oficial Microsoft nem adiciona autenticação oficial.

## 5. Conteúdo e dependências

O botão normal de instalar mods agora passa pelo resolvedor de dependências. Foram corrigidos:

- Consulta direta da versão escolhida do Modrinth, sem perder pins de `version_id`.
- Dependências obrigatórias com seleção da versão e do loader compatíveis; dependências opcionais e embutidas não são instaladas automaticamente.
- Conflitos declarados e exigências de versões diferentes do mesmo projeto são recusados antes da gravação.
- Dependências CurseForge obtidas dos metadados do arquivo selecionado.
- Mods Fabric podem ser considerados para Quilt; Forge e NeoForge não são tratados como equivalentes.
- Instância inexistente, Vanilla para instalação de mod e alteração de conteúdo durante a execução do jogo recebem erros explícitos.
- Os downloads do plano são preparados e verificados antes de promover os arquivos. Os downloads individuais também conferem tamanho e SHA-1, quando disponível.
- O mesmo arquivo de Fabric API vindo de Modrinth e CurseForge é reaproveitado, evitando duas entradas no banco para um único `.jar`.

O teste real instalou **Inventory Profiles Next** pelo Modrinth com três dependências obrigatórias, verificou pins e recusou uma versão incompatível preservando os arquivos. Depois instalou **Controlling** pelo CurseForge com suas dependências e confirmou seis registros para seis arquivos. [Evidência de instalação](C:/Users/preda/OneDrive/Documenti/Luxmc/docs/validation/content-dependencies-2026-10-06.json).

Resource packs podem ser usados em Vanilla. Mods precisam de loader. Shaders precisam de suporte de renderização compatível. Data packs pertencem à pasta `datapacks` de um mundo selecionado, não à pasta `mods`.

**Limite:** os metadados dos provedores não descrevem todos os possíveis conflitos entre mods. A instalação não oferece uma transação completa entre sistema de arquivos e banco em caso de queda de energia durante a promoção final.

## 6. Botões e tema claro: pesquisa do Modrinth

Consultei o código atual do componente de botão do Modrinth, incluindo tamanhos, variantes, estados de interação, foco, desabilitação e carregamento. O sistema usa hierarquia consistente, resposta curta de interação e estados acessíveis. [Button.vue](https://raw.githubusercontent.com/modrinth/code/main/packages/ui/src/components/base/buttons/Button.vue), [ButtonFrame.vue](https://raw.githubusercontent.com/modrinth/code/main/packages/ui/src/components/base/buttons/ButtonFrame.vue).

No Luxmc, a revisão aplica tipografia de interface ao sistema compartilhado, ícones com área de clique de 44 px, bordas e sombras discretas, ações principais destacadas, ações secundárias mais limpas e foco visível. As transições respeitam a preferência de movimento reduzido.

No tema claro, botões secundários com wallpaper têm um piso de opacidade para conservar leitura. Entradas de diálogos e mensagens do diagnóstico usam as cores do tema. Também foi corrigido um fundo invisível de menu que recebia indevidamente o estilo de botão de ícone.

Foram revisadas visualmente Home, Mods, Amigos e Configurações em tema claro e escuro, com e sem wallpaper, e testados overflow e navegação por teclado. [Evidência da interface](C:/Users/preda/OneDrive/Documenti/Luxmc/docs/validation/general-ui-2026-10-06.json).

**Limite:** contraste sobre uma imagem arbitrária varia com a imagem e a opacidade escolhida. Os testes de contraste dos controles de criação cobrem os cenários utilizados, não cada pixel de todo wallpaper possível.

## 7. Traduções e acessibilidade

Os dicionários português, inglês e espanhol são verificados para paridade de chaves e parâmetros. Foram traduzidos os novos estados do atualizador, reconexão e diagnósticos. O tempo desde a última atividade de um amigo agora usa pluralização do idioma em vez de inserir “dia”, “dias”, “hora” ou “horas” no inglês/espanhol.

Os seletores aceitam setas, Home, End e Escape e devolvem o foco ao controle. O diálogo de diagnóstico tem nome acessível, foco contido, fechamento por Escape e descrição traduzida, conservando o trecho original do log.

Nomes de mods, descrições dos autores e mensagens originais do jogo continuam sendo conteúdo do provedor. Paridade de chaves não é uma certificação de acessibilidade integral nem uma revisão linguística humana de cada frase.

## 8. Atualizador

O download do instalador faz até três tentativas com intervalo limitado. Uma transferência interrompida reinicia o arquivo da tentativa seguinte, evitando juntar bytes parciais incorretamente. Há validação de origem e redirecionamentos, tamanho máximo, corpo vazio, tamanho anunciado e tempo sem progresso. Downloads incompletos são descartados.

O download do site também usa o instalador hospedado quando a consulta de metadados ao GitHub fica indisponível, conservando HEAD e requisições parciais. Dois novos testes cobrem essa falha externa.

A hospedagem ignorava `Range` no arquivo estático. A rota de download agora entrega intervalos de bytes por streaming, aceita intervalos abertos e sufixos e recusa intervalos inválidos. Também confere `If-Range`: se o instalador mudou desde o download anterior, entrega o arquivo inteiro para evitar misturar partes de instaladores diferentes. Essa condição segue a semântica de [If-Range no RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html#name-if-range).

O teste nativo usa um servidor que interrompe a primeira resposta e entrega a segunda completa. A interface testa seleção do instalador, falha de consulta e nova tentativa após bloqueio por jogo aberto.

**Limite:** o download do instalador reinicia a transferência; não implementa retomada por HTTP Range. HTTPS, origem permitida e tamanho conferido não equivalem a assinatura Authenticode ou a um manifesto criptográfico assinado. Mantendo a mesma versão, o hotfix do site exige baixar/reinstalar o arquivo atualizado; a comparação de versão não anunciará um número novo.

## 9. Instalação, atualização e desinstalação

O teste isolado recusava substituir uma instalação de usuário existente, usou uma pasta temporária única, conferiu o registro de desinstalação silenciosa, reinstalou a mesma versão e executou o desinstalador oficial. **Instalação, reinstalação e desinstalação terminaram com código 0.** O aplicativo e o registro foram removidos da pasta de teste. Os 11 arquivos protegidos encontrados, incluindo arquivos do projeto e dados, mantiveram seus hashes.

O resultado e o hash estão na [evidência do ciclo do instalador](C:/Users/preda/OneDrive/Documenti/Luxmc/docs/validation/general-installer-lifecycle-2026-10-06.json). A reinstalação da mesma versão valida o hotfix; uma migração de todas as versões antigas do aplicativo não é comprovada por esse único ciclo.

## 10. Segundo plano e diagnóstico

O wallpaper em shader respeita perda de foco, janela oculta, execução do jogo e modo de desempenho. O pré-carregamento de rotas pausa quando o launcher está oculto ou o jogo está rodando. Atualizações de presença de menu evitam trabalho desnecessário em janela oculta; a presença do jogo continua ativa.

Os diagnósticos agora distinguem sessão inválida, watchdog no encerramento, falha de DNS e conexão recusada. O log original permanece disponível. A otimização não força IPv4 globalmente e remove injeções externas de opções Java. O texto enviado sobre Sodium e DNS foi tratado como relato de terceiros; não foi aplicado um downgrade geral do Sodium sem reproduzir a combinação de mods.

**Limite:** não foi medido um percentual de redução de CPU/RAM em toda máquina. O launcher mantém serviços necessários à sala e à presença enquanto permanece em segundo plano.

## Comparação com as notas do SKlauncher enviadas

Esta tabela compara o código e os testes do Luxmc com os recursos descritos nas suas notas. Não é uma auditoria do binário do SKlauncher. A página oficial foi consultada como referência de produto. [SKlauncher](https://next.skmedix.pl/).

| Recurso citado | Situação no Luxmc |
| --- | --- |
| Exportar/importar instância | Exportação ZIP e modpack, importação `.mrpack`; não há compatibilidade declarada com `.skpack` |
| Pins e dependências por loader | Corrigidos e testados com Modrinth e CurseForge nesta revisão |
| Ações em lote | Há ações em lote para instâncias; seleção Ctrl/Shift de mods não foi identificada como fluxo completo |
| Galeria de skins | Coleção de skins, importação e integração NameMC já presentes; aplicação revisada |
| Editor de controles | Foi retirado da tela por solicitação anterior; não foi recolocado |
| Ornithe e Legacy Fabric | Não adicionados; escopo de teste foi Vanilla, Fabric, Quilt, Forge e NeoForge |
| Cache de arquivos e ícones | Cache existente, com recibo de verificação persistente ampliado nesta revisão |
| Visualização de logs | Fluxo limitado e interface existente; novas categorias de diagnóstico nesta revisão |
| Migração de todos os formatos de outros launchers | Não há comprovação completa para todo formato citado nas notas |
| Java e argumentos JVM | Seleção por versão existente; isolamento de variáveis externas reforçado |
| Aparência com preview | Já presente; restart do vídeo e preferências revisados |
| Salas P2P e mundos de convidados | Presentes e testados; esse recurso não foi descrito nas notas fornecidas |

Os filtros e o resolvedor foram conferidos com a documentação da API do Modrinth: versões de jogo, loaders e tipos de dependência fazem parte dos metadados usados para escolher arquivos. [API de versões do Modrinth](https://docs.modrinth.com/api/operations/getprojectversions/). A compatibilidade Quilt/Fabric continua sujeita a exceções reais dos mods. [FAQ do Quilt](https://quiltmc.org/en/about/faq/).

## Resultado das verificações

- `pnpm check`: zero erros e zero avisos.
- Vitest: **125 testes aprovados**, em 21 arquivos.
- `cargo check`: aprovado; quatro avisos existentes de código/importação não utilizado no Windows.
- Rust: **130 testes aprovados**, dois testes externos ignorados na execução padrão.
- Relay público: o teste ignorado de transporte por relay foi executado separadamente e **passou**.
- Site: **38 testes aprovados**.
- Idiomas no navegador: sete cenários aprovados, incluindo português de Portugal, espanhol do México, fallback para inglês, preferência manual e retorno ao idioma do sistema.
- Instalação real de conteúdo: Modrinth e CurseForge aprovados.
- Interface: temas, wallpaper, teclado, skin, catálogo, reconexão e atualizador com cenários simulados; vídeo do wallpaper com mídia real.

## Matriz do executável final

Build otimizado concluído. Arquivos do jogo e Java já presentes em cache; compilador encerrado durante a rodada final. A coluna de preparação termina na criação do processo Java. A segunda coluna aguarda áudio, atlas de texturas e mais cinco segundos com o processo ativo; não mede abertura de um mundo nem FPS.

| Instância / loader | Minecraft | Loader | Preparação | Recursos prontos + confirmação | Resultado |
| --- | --- | --- | --- | --- | --- |
| Vanilla 1.8.9 | 1.8.9 | Vanilla | 2.84 s | 12.88 s | Aprovado |
| Vanilla Perfected | 26.2 | 0.19.3 | 4.33 s | 43.31 s | Aprovado |
| Vulkan Optimized | 26.1.2 | 0.19.3 | 3.03 s | 26.41 s | Aprovado |
| Vanilla | 26.3 | Vanilla | 3.53 s | 20.58 s | Aprovado |
| Fabric | 1.20.1 | 0.19.5 | 2.28 s | 42.97 s | Aprovado |
| Quilt | 1.20.1 | 0.24.0 | 2.11 s | 45.45 s | Aprovado |
| Forge | 1.20.1 | 47.4.10 | 1.91 s | 22.98 s | Aprovado |
| Neoforge | 1.21.1 | 21.1.256 | 2.41 s | 22.47 s | Aprovado |

[Evidência da rodada final](C:/Users/preda/OneDrive/Documenti/Luxmc/docs/validation/general-launch-release-2026-10-06.json).

Tempos observados neste PC, nesta rodada, com cache. Não são promessa de desempenho em outro hardware ou numa primeira instalação.

## Arquivo de entrega

- Nome: **Lux MC Launcher.exe**.
- Versão: **3.0.2**, preservada.
- Tamanho: **20.329.717 bytes**.
- SHA-256: `8c11f5866c9dbc78713e356482b7243f1bb5f90e6cde0114d1775979c6efea60`.
- [Instalador local](<C:/Users/preda/OneDrive/Documenti/Luxmc/release-windows/Lux MC Launcher.exe>).
- [Cópia em Downloads](<C:/Users/preda/Downloads/Lux MC Launcher.exe>).
- GitHub: nenhum push, tag ou release desta revisão.

## Publicação conferida

O site foi atualizado e o arquivo público foi baixado integralmente para conferência, sem executar esse download. A resposta GET retornou HTTP 200, o nome esperado e **20.329.717 bytes**. Seu SHA-256 coincide com o instalador aprovado no ciclo de instalação. HEAD também passou. As respostas parciais HTTP 206 foram conferidas no começo, no meio e no final do arquivo, comparando os bytes com a cópia local.

- [Baixar o instalador atualizado no site](https://luxmc-r92.pages.dev/download/windows).
- [Evidência da entrega pública](C:/Users/preda/OneDrive/Documenti/Luxmc/docs/validation/general-delivery-2026-10-06.json).
- [Deploy conferido](https://15fe745b.luxmc-r92.pages.dev).

Como o número permanece 3.0.2, quem já está nessa versão deve baixar e reinstalar o arquivo atualizado para receber este hotfix. O instalador preserva os dados; não foi criado um anúncio automático de versão superior.

Esta entrega corrige os pontos descritos e registra os testes realizados. Não comprova ausência de qualquer falha em todos os PCs, versões, servidores e modpacks. A confirmação restante mais importante é uma partida real entre os dois PCs de vocês usando o instalador atual e reiniciando o Minecraft nos dois.
