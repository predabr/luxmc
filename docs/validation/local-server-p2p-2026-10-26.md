# Hospedagem local P2P — 26/10/2026

## Implementação

- Gerenciador nativo com configurações persistentes isoladas, um processo ativo e console limitado a 500 linhas com cursor por execução.
- Criação automática de servidores Vanilla pelo manifesto oficial, integridade do JAR e Java compatível reutilizado.
- Consentimento explícito à EULA, autenticação oficial, loopback e portas efêmeras sem redirecionamento de roteador.
- Iniciar, parar com salvamento, enviar comandos, salvar o mundo, abrir pasta e editar configurações com o servidor parado.
- Compartilhamento do servidor gerenciado ou da porta de um servidor Java existente, com suporte a loopback IPv4/IPv6.
- Metadados de servidor no protocolo da sala, instruções de Conexão direta, anúncio LAN e gravação opcional no Multijogador do convidado.
- Cancelamento do compartilhamento quando o processo para; encerramento de um servidor não fecha uma sala sem vínculo com ele.
- Retirada do anúncio quando a porta deixa de responder e retorno do anúncio se o servidor externo reabrir na mesma porta.
- Verificação nativa de capacidade Free/Plus/Pro; ausência do serviço usa o limite gratuito.
- Atualização do launcher bloqueada enquanto há servidor gerenciado ativo.
- Encerramento normal no Tauri e no Electron; Job Object no Windows e sinal ao filho no Linux para evitar processos abandonados.
- Servidores classificados como conteúdo de jogo no armazenamento, separados dos dados do launcher e das pastas de cache.
- Interface nova em português, inglês e espanhol, usando os tokens e controles existentes.

## Verificações

| Verificação | Resultado |
| --- | --- |
| `cargo check --locked` no Windows | Aprovado; avisos de código não utilizado |
| `cargo test --lib --locked` | 305 aprovados, zero falhas, 13 ignorados |
| `pnpm check` | Zero erros e zero avisos |
| `pnpm test:run` | 230 testes aprovados em 39 arquivos |
| `pnpm build` | Aprovado, incluindo a rota `/hosting/server` |
| `node electron/build.js` | Compilação aprovada |
| Verificações de idiomas e planos | 2 testes aprovados; mesmos campos e variáveis em português, inglês e espanhol |
| `local-server.browser.cjs` | Aprovado em 1440 e 960 px |
| `catalog-room-polish.browser.cjs` | Aprovado; navegação, sala e presença preservadas |
| `instance-settings-repair.browser.cjs` | Aprovado; categorias, rascunho, cancelamento, gravação e layout |
| `git diff --check` | Aprovado |

A suíte nativa verifica consentimento antes de downloads ou gravações de mundo, propriedades restritas a loopback, preservação do mundo ao editar, limite de capacidade, cancelamento da preparação, console limitado e ocultação de credenciais. Um processo filho de teste recebe `stop`, grava um marcador de salvamento, simula travamento e saída com erro; o supervisor é exercitado com Job Object no Windows. Os túneis de fixture transmitem bytes e metadados do servidor e retiram o anúncio quando a porta fecha, sem exigir um mundo LAN.

O teste de navegador cobre criação com EULA inicialmente desmarcada, memória, jogadores, iniciar/parar, console, compartilhamento, edição sem troca de versão, porta externa e adição ao Multijogador do convidado. O teste passou novamente após ajustar a pausa de consultas durante o compartilhamento. As capturas usam dados simulados e ficaram em `C:/Users/preda/AppData/Local/Luxmc-build/requests/local-server-20261026/browser-final`.

Os 13 casos ignorados não são contados como aprovados: 12 exigem dados ou infraestrutura específicos e um é o processo auxiliar chamado explicitamente pelos testes do supervisor. Nenhum cliente Minecraft real nem servidor Minecraft real foi iniciado. A partida real entre duas redes, a autenticação do jogo nesse cenário e a execução em Linux/macOS continuam pendentes de validação nos respectivos ambientes. Os últimos ajustes de coordenação com o atualizador e de limpeza de cache também passaram por `cargo check` após a suíte nativa.

O build de produção do frontend foi repetido após o último ajuste da interface. Os avisos sobre imports dinâmicos compartilhados e tamanho de chunks continuam informativos; o build terminou com sucesso.

## Disponibilidade

Esta revisão está no código local. Nenhum site, release pública, conta, perfil, mundo existente ou instalação do usuário foi alterado por estes testes. A criação automática é de servidores Vanilla; para modpacks usa-se o pacote de servidor do autor e o compartilhamento da sua porta local.
