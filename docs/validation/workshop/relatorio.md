# Luxmc — manutenção, downloads e diagnóstico

Revisão local de 07/10/2026. O nome Luxmc e a versão 3.1.0 foram mantidos. Esta revisão não foi publicada no site ou no GitHub e não foi instalada no computador do usuário.

## Recursos implementados

| Pedido | Resultado e como conferir |
| --- | --- |
| Histórico de desempenho | Ajustes → Central de manutenção. Guarda até 500 registros de preparação e sessões em SQLite; compara a preparação média e o maior pico de RAM por instância. A tabela mostra as medições recentes. |
| Temas compartilháveis | Importar/exportar `.luxtheme` na central. Inclui cores, densidade, animações, transparência e wallpaper local de até 64 MB, com SHA-256. O arquivo importado copia o wallpaper para o armazenamento do launcher. Contas, Java e argumentos de execução ficam fora do tema. |
| Diagnóstico com privacidade | Preparar diagnóstico mostra problemas, recomendações e uma prévia. A exportação usa o seletor de arquivos e um comando nativo. Remove tokens reconhecidos, e-mails, IPs IPv4, UUIDs e nomes em caminhos pessoais; a prévia deve ser revisada antes de compartilhar. Não envia relatórios automaticamente. |
| Notas de atualização visuais | O diálogo apresenta destaques agrupados por títulos, imagens HTTPS de origens aprovadas quando incluídas nas notas e o texto completo sanitizado. Mantém progresso, erros e nova tentativa. |
| Integridade dos downloads | Verifica tamanho, hash disponibilizado pelo provedor e estrutura dos pacotes. O atualizador exige SHA-256 do manifesto `SHA256SUMS` da mesma release oficial antes de executar o instalador. Arquivos divergentes são descartados com uma mensagem de recuperação. |
| Pausa e retomada | O painel global permite pausar e retomar instalações. A fila é persistida em SQLite; após reabrir, tarefas interrompidas aparecem pausadas. Os bytes parciais são mantidos e retomados com Range e If-Range quando suportados. Se o servidor ignorar Range, o arquivo recomeça sem concatenar dados incompatíveis. |
| Fila de instalações | Até 50 tarefas, executadas uma por vez. Permite adicionar outro modpack durante uma instalação, priorizar, remover e retomar falhas. A versão escolhida é fixada no registro da tarefa. O estado continua visível ao navegar entre páginas. |
| Repetição de arquivos que falharam | A instalação mantém uma identidade de trabalho e reaproveita os arquivos completos que passam na verificação. Retomar não precisa baixar novamente cada mod já validado. Instalações concluídas são reconhecidas antes de criar outra instância quando a tarefa é retomada. |
| Pré-verificação P2P | Na lista de mundos da sala, selecione a instância e clique em Verificar antes de entrar. Compara versão, loader e assinatura dos arquivos JAR anunciados por cada participante. Mostra informações ausentes, divergências, transporte, ping e orientação de autenticação. Mantém anúncios compatíveis com o formato anterior. |
| Validação de modpacks | A central verifica arquivos ZIP/JAR, CRC, hashes conhecidos, IDs duplicados e dependências declaradas em metadados Fabric, Quilt, Forge e NeoForge. Também examina resource packs, shaders e datapacks dos mundos na verificação profunda. A preparação do jogo usa uma verificação estrutural quando os arquivos mudam, preservando o caminho rápido de instâncias já verificadas. |
| Backups automáticos | Opção configurável na central. Antes de iniciar o Minecraft, cria um backup dos mundos que não possuem uma cópia recente nas últimas 24 horas. Mantém até cinco por mundo, aplica limite de espaço por instância e preserva a cópia mais recente mesmo se superar o limite. |
| Restauração | Descompacta e valida em uma pasta temporária; só depois substitui o mundo. Preserva a pasta anterior em `saves/.before-restore-*` e tenta reverter se a troca falhar. Backup/restauração bloqueiam o início concorrente do Minecraft. Não seguem links simbólicos dentro do mundo e recusam caminhos de extração que escapem da pasta. |
| Recomendações dos amigos | Os favoritos dos perfis públicos viraram ações que abrem a busca do catálogo. A instalação continua sendo escolhida e confirmada no catálogo; nomes antigos não são tratados como IDs de projetos. |
| Design | Nova central responsiva; painel de fila com ações e estados explícitos; comparação P2P; destaques visuais nas atualizações; foco de teclado, superfícies, formulários e contrastes refinados globalmente. Preserva a estrutura da barra lateral. Novos textos em português, inglês e espanhol. |

## Validação

- `pnpm check`: zero erros e zero avisos.
- Vitest: 23 arquivos, 131 testes aprovados.
- Rust: 141 testes aprovados; dois testes externos permanecem ignorados (NameMC público e infraestrutura de relay pública).
- `cargo check`: aprovado; cinco avisos de código não usado já existentes.
- Navegador, 1440 e 960 px, temas claro e escuro: histórico, fila persistente, prioridade, remoção, pausa/reabertura/retomada, validação, exportação de diagnóstico, importação/exportação de tema e confirmação de restauração.
- Pré-verificação P2P: versões/loaders/mods iguais, metadados ausentes, versão diferente e aviso de conta Luxmc.
- Regressão: lista de mil mods, filtros, alternância e congelamento de atualizações; perfis offline; imagens visíveis em lotes; instalação ao trocar de página; tela de amigos em quatro cenários; menu inicial e sanitização das notas de atualização.
- Testes nativos com HTTP real em loopback: continuação por Range, reinício quando Range é ignorado e retenção de dados após interrupção do atualizador.
- Backup: ida e volta de arquivos de mundo, preservação do original e recusa de arquivo com caminho `../`.

O instalador final e seu SHA-256 são registrados em `artifact.json`. A compilação de produção inclui esta revisão; o pacote é para teste local, sem atualização da instalação existente.

## Limites importantes

- O tempo medido é até a criação do processo do jogo. O carregamento até o menu do Minecraft pode levar mais tempo.
- RAM só aparece depois de existir uma sessão medida; não foram inventadas medições anteriores.
- A retomada depende da resposta do servidor. Pausar uma importação interrompe a operação e a retomada reaproveita os arquivos validados; não congela o processo de extração no meio de uma instrução.
- A assinatura dos mods compara arquivos inteiros. Mods apenas do cliente e configurações diferentes podem exigir uma avaliação adicional; igualdade de arquivos não garante compatibilidade de todo o ambiente.
- A checagem não cria uma sessão oficial Microsoft para contas locais ou Luxmc. Servidores que exigem autenticação oficial continuam aplicando suas regras.
- Metadados e dependências internas variam entre loaders. Dependências não confirmadas são informadas; a resolução final de versões e JARs internos é feita pelo loader.
- A verificação profunda limita o conteúdo descompactado de cada arquivo a 1 GiB e a coleta inicial de pastas de mundos a 128 entradas. Uma verificação automática completa de CRC em cada abertura prejudicaria o início rápido.
- Backups são opcionais; a cópia mais recente de cada mundo é protegida, portanto o limite de espaço não é um limite destrutivo absoluto.
- Os testes de rede usam participantes locais reais e streams criptografados; não constituem uma garantia para todos os roteadores, firewalls e servidores externos.

## Divulgação e Google

O projeto já contém `robots.txt`, sitemap XML, URL canônica, descrição e metadados sociais. Para pedir indexação, use a propriedade de prefixo de URL `https://luxmc-r92.pages.dev/` no [Search Console](https://search.google.com/search-console), confirme a propriedade, envie `sitemap.xml`, inspecione a página inicial e escolha Solicitar indexação. O Google informa que o rastreamento pode levar dias ou semanas e que a solicitação não garante inclusão imediata. [Documentação oficial](https://developers.google.com/search/docs/crawling-indexing/ask-google-to-recrawl).

Minha sugestão para divulgação: publicar demonstrações curtas reais do launcher, tutoriais de instalação e modpacks, páginas públicas úteis sobre erros frequentes e demonstrações do multiplayer. Coloque o link oficial nas descrições dos vídeos e na apresentação do projeto. Convide criadores pequenos de Minecraft para testar e acompanhe cliques, instalações e retenção. O Google recomenda conteúdo útil e promoção para comunidades interessadas; popularidade não vem apenas de indexar a página. [Guia oficial de SEO](https://developers.google.com/search/docs/fundamentals/seo-starter-guide).

## Linguagens e tecnologias

| Parte | Recomendação |
| --- | --- |
| Rust no backend | Manter. Já integra arquivos, downloads, processos e rede com o Tauri. Uma reescrita teria custo alto e precisa de evidência de ganho. |
| Svelte 5 + TypeScript | Manter. Continuar medindo renderização, cache, listas e trabalho em segundo plano. |
| JavaScript nas APIs do site | A migração gradual para TypeScript é a troca mais útil para manutenção: contratos e tipos compartilhados podem antecipar erros. Não é uma promessa de execução mais rápida. |
| C++ auxiliar | Considerar Rust caso simplifique compilação e manutenção, somente após comparar os resultados dos auxiliares existentes. |
| Java do agente Minecraft | Manter pela integração e pelas versões antigas atendidas. |
| Python de ferramentas | Manter para automação; a linguagem dos scripts de build não define a velocidade do launcher em uso. |
| Electron/Tauri | São tecnologias de interface, não linguagens. Avaliar concentrar distribuição em Tauri onde os testes de compatibilidade permitirem, mantendo alternativas onde forem necessárias. |

A avaliação acima é uma recomendação baseada na estrutura atual do projeto. O Tauri utiliza Rust e a webview do sistema; Cloudflare Pages Functions aceita TypeScript. [Arquitetura do Tauri](https://v2.tauri.app/concept/architecture/), [TypeScript no Cloudflare Pages](https://developers.cloudflare.com/pages/functions/typescript/).
