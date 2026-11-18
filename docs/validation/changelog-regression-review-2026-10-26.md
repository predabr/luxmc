# Revisão de regressões — 26/10/2026

O histórico do SKlauncher fornecido pelo usuário foi usado como lista de cenários para conferir comportamentos existentes. As descrições de funcionalidades futuras não são requisitos desta revisão. Nenhuma lista de servidores, nova aba de servidores ou gerenciador de hospedagem foi implementado.

## Falhas confirmadas e correções locais

| Área | Problema | Correção |
| --- | --- | --- |
| Argumentos do Java | O editor removia as aspas na normalização, e o lançamento separava caminhos com espaços. | O editor e o lançamento compartilham o mesmo analisador. A normalização conserva espaços, aspas, caminhos Windows e UNC. Aspas incompletas impedem o salvamento. |
| Memória da instância | Argumentos de memória herdados podiam impedir a aplicação do limite escolhido ou deixar a memória inicial acima desse limite. | A memória máxima herdada é substituída pelo limite da instância, preservando a memória inicial compatível. Argumentos personalizados explícitos continuam tendo precedência no lançamento; ao salvar pelos controles de memória, esses controles atualizam somente os argumentos de heap. |
| Validação no daemon/Electron | O frontend envia `input`, mas o daemon lia apenas `jvmArgs`/`jvm_args`, validando uma string vazia. | O daemon reconhece `input`, mantém os aliases existentes e encaminha os limites de memória. |
| Catálogos Forge/NeoForge | Falhas de rede, HTTP e respostas inválidas podiam virar uma lista vazia e parecer incompatibilidade. | Erros são encaminhados à interface já existente de falha e nova tentativa. O Forge preserva o resultado das promoções quando sua outra fonte falha. Uma resposta válida sem versões compatíveis continua sendo uma lista vazia. |
| Teste de LAN | O teste dependia da posição dos transformadores Java. A inclusão de um transformador de capacidade deixou esse índice incorreto. | O teste aplica a cadeia registrada, como a instrumentação Java, e verifica descoberta LAN, autenticação privada e classes não relacionadas. Isso corrige o teste; não comprova uma sessão real entre dois computadores. |

## Proteções existentes conferidas no código

- O lançamento e os comandos Java removem `JAVA_TOOL_OPTIONS`, `_JAVA_OPTIONS`, `JAVA_OPTIONS`, `JDK_JAVA_OPTIONS` e `CLASSPATH` herdados.
- O atualizador bloqueia instalação quando há um jogo ativo ou uma preparação em andamento.
- Downloads com erro HTTP 404 interrompem as tentativas e removem o arquivo parcial correspondente.
- Downloads verificam tamanho/hash e preservam o destino anterior quando a transferência falha.
- A resolução de loaders de modpacks reconhece os manifestos e preserva versões fixadas.
- A verificação de mods tem cache e cancelamento; duplicatas removidas são guardadas em uma pasta de recuperação.

## Limites

A revisão não demonstra ausência de todos os bugs em todos os modpacks e sistemas. O erro anterior do seletor de versões no CI do Windows não foi reproduzido no teste local com Edge, em duas larguras; seu comportamento não foi alterado sem uma causa confirmada. Nenhum Minecraft de usuário foi iniciado, nenhum perfil real foi modificado e nenhuma publicação ou instalação é parte desta revisão.

## Validação

| Verificação | Resultado |
| --- | --- |
| `pnpm check` | Sem erros ou avisos |
| `cargo check --locked` | Aprovado, com avisos de código não utilizado |
| `cargo test --lib --locked` | 298 aprovados, zero falhas, 12 ignorados |
| `pnpm test:run` | 230 aprovados em 39 arquivos |
| `pnpm build` | Aprovado; avisos de empacotamento sobre imports e tamanho de chunks |
| `AgentHooksProbe` com Java 17 e o agente distribuído | Aprovado; falhava antes da correção do teste |
| `release-4.0-polish.browser.cjs` | Aprovado em 1440 e 960 px; seletor, teclado e confirmação de skin |
| `instance-settings-repair.browser.cjs` | Aprovado em 1440 e 1000 px; categorias, rascunho, cancelar e guardar |
| `jvm-settings.browser.cjs` | Aprovado; entrada preservada, limites encaminhados à validação nativa e falha impedindo gravação |
| `create-instance.browser.cjs` | Aprovado; seleção estável/prévia de loader |
| `java-runtime25.browser.cjs` | Aprovado; opções de runtime existentes |
| `launch-startup.browser.cjs` | Aprovado; estado de preparação, navegação e erro |
| `instance-preparation.browser.cjs` | Aprovado; primeira preparação e reparo explícito |
| `git diff --check` | Aprovado |

Os testes de navegador usam dados simulados e Edge em modo headless. Os 12 testes ignorados exigem arquivos instalados explicitamente selecionados ou serviços externos específicos; não são contados como aprovados. Os testes nativos de P2P utilizam conexões e protocolos em fixtures, sem abrir um Minecraft real.

As correções estão no código local. Esta revisão não gerou um novo instalador nem alterou GitHub, site ou instalação do usuário. Capturas e logs temporários ficaram na pasta privada `C:/Users/preda/AppData/Local/Luxmc-build/requests/changelog-review-20261026`.
