# Verificação consolidada — 18/09/2026

Código preparado para publicação posterior, sem deploy ou alteração de banco remoto. As evidências desta sessão estão em [evidencias/](evidencias/) e [visual/](visual/).

## Trabalho realizado na conversa

| Área | Resultado |
| --- | --- |
| CurseForge | Chave original restaurada e preservada, incluindo configuração por ambiente/keyring; API real validada. Header enviado somente ao host da API, três CDNs, três tentativas com espera exponencial e timeout de 35 s. Sem exclusão de JAR por nome/prefixo. |
| Modrinth | Manifestos em raízes internas, normalização de barras, seis downloads concorrentes, cancelamento, validação de tamanho/hash, overrides recursivos e precedência de client-overrides. |
| Integridade | Auto-heal antes da JVM, recuperação de JAR vazio/ausente e de overrides, respeito aos mods desativados. Importações simultâneas não alteram o cancelamento uma da outra. |
| Loader | Correção de bug reproduzido: “CurseForge” no nome de um perfil Fabric fazia o launcher tentar Forge. Agora usa manifesto e descritores internos dos JARs; nomes de perfis/mods não decidem o loader. Loader primário do manifesto é respeitado. |
| Arquivos | Bloqueio de traversal, nomes Windows inválidos/reservados e symlinks, limites de extração, escrita atômica. Novos perfis usam pastas UUID, evitando escapes e colisões por nome. |
| Atualização de packs | Preparação e validação em outra pasta, preservação dos dados pessoais/mods não gerenciados, troca em transação SQLite e retenção da pasta anterior como backup. |
| Interface | Tema obsidian azul e tokens semânticos, botões com estados/microinterações, grid/lista, banners e estatísticas, hero da instância, RAM/Java/JVM integrados, mundos/backups e screenshots preservados, cards de conteúdo com mesh/banner e instalação. |
| Acessibilidade | Foco contido/restaurado em modais, respeito a modais empilhados, botões com loading/disabled, links desativados sem navegação, identificação de ações por ícone. Lista virtual tipada e ajustada à altura real/filtros. |
| Skins | Cabeças Minecraft nos amigos, fallback de avatar, correção do canvas do card de jogador e de erro de clipboard. Falha de WebGL agora mantém o estúdio utilizável com textura e explicação. |
| Social | API Pages/D1, tokens nativos por instalação e hashes no servidor, convites com aceite, presença com expiração, busca limitada, favoritos, sidebar recolhível, LAN/UPnP e entrada em instância compatível. Cadastro concorrente idempotente, validação de corpo/endereço e rate-limit. |
| Portal | Hero obsidian, iluminação ambiente sem grade pesada, mockup com abas, detecção de SO, versão/tamanho reais e downloads via edge com seleção segura de assets. |
| Segurança | DOMPurify em HTML externo, restrições de IPC/navegação/arquivos no Electron, CSP no Tauri, remoção da desativação automática do sandbox WebKit e correções de dependências. |
| Publicação | Migração D1 e instruções de ativação, configuração separada de testes, correção do comando Pages para usar --cwd; CI preparado para Linux e Windows com lockfiles congelados. |

Nenhuma rota ou comando do produto foi removido. A chave CurseForge não foi rotacionada, apagada novamente ou publicada neste relatório.

## Validação executada

| Verificação | Resultado |
| --- | --- |
| `pnpm check` | 0 erros, 0 avisos |
| `pnpm test:run` | 30 testes aprovados |
| `pnpm test:website` | 6 testes aprovados, com SQL real em SQLite |
| `pnpm build` | Build estático concluído; permanecem avisos de chunk grande/imports SSR externos não usados |
| `pnpm electron:compile` | Compilação concluída |
| `cargo check` em src-tauri | Concluído sem erros |
| `cargo test --lib --test glib_variant_regression` | 49 testes de biblioteca + 1 regressão GLib aprovados; inclui API CurseForge real |
| `node tests/native-smoke.mjs --launch` | Importações e inicialização real do Minecraft Linux pelos dois provedores aprovadas |
| `node tests/social-http.mjs http://127.0.0.1:8795` | Dois clientes independentes aprovados no runtime Pages + D1 local |
| Auditorias npm e Rust | 0 vulnerabilidades conhecidas nas dependências auditadas; sete avisos Rust de manutenção |

O teste nativo usa Fabulously Optimized 5.4.1 para Minecraft 1.20.1 como fixture publicada: 51 arquivos do Modrinth verificados por hash; 46 entradas CurseForge e 48 JARs preservados. A seleção dessa versão é uma fixture reproduzível, não uma recomendação de versão atual do pack. Testa JAR zerado, mod desativado, caminhos aninhados com barras Windows, client-overrides, cancelamento e exclusão mútua. Ambos os jogos chegaram à inicialização de renderização/recursos e foram encerrados pelo teste. Contas offline, banco, downloads e perfis de teste ficaram em `/tmp`; perfis reais do usuário não foram usados.

O teste HTTP exercita cadastro, busca, convite, privacidade antes do aceite, aceite pelo destinatário, atividade, IP/porta compartilhados, offline e remoção. Não é um teste de tráfego Minecraft entre computadores físicos.

A revisão visual inclui início, instâncias grid/lista, detalhe/configurações, mods, amigos, configurações gerais, skins, logs/histórico, notícias, screenshots e portal desktop/mobile; também inclui instâncias/configurações/amigos no tema claro. As capturas da interface usam respostas nativas demonstrativas. Arte de skins e logos conserva suas cores próprias.

## Correções das dependências Rust

- GLib: backport do conserto oficial do out-pointer mutável, preservando a ABI GTK3 exigida pelo Tauri. Versão permanece 0.18.5; não foi inventada uma versão corrigida nem suprimido o advisory. O patch local é verificado por comparação de código e teste de regressão; `cargo audit` sozinho não certifica dependências vendorizadas. [Patch aplicado](evidencias/glib-backport.diff), [correção upstream](https://github.com/gtk-rs/gtk-rs-core/pull/1343), [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html).
- RSA: a resolução do driver MySQL opcional, sem uso no aplicativo SQLite, foi removida das cópias compatíveis SQLx/SQLx-macros-core. A API SQLite e as consultas continuam preservadas. RSA/sqlx-mysql saíram do lockfile. Não se afirma que a implementação criptográfica RSA foi corrigida. [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html).
- rustls foi atualizado para 0.23.45. A cadeia vulnerável node-vibrant/Jimp foi substituída pelos módulos granulares Vibrant, além das demais atualizações JavaScript.
- Permanecem avisos de manutenção de paste, proc-macro-error e cinco crates unic. Não são sete vulnerabilidades corrigidas. São dependências transitivas registradas em [dependency-audit.json](evidencias/dependency-audit.json).

As cópias em `src-tauri/vendor/` preservam licenças, versão e documentação do delta; precisam ser revistas nas próximas atualizações. Zero advisories conhecidos não garante ausência de falhas ainda desconhecidas.

## Limites e publicação

- Windows: workflow preparado; nenhum instalador ou jogo Windows foi executado nesta máquina Linux.
- Social: teste local com dois clientes concluído; produção Cloudflare e teste entre dois computadores/roteadores permanecem dependentes da publicação e do ambiente real.
- UPnP requer roteador compatível e endereço alcançável; não há relay que atravesse CGNAT. Identidade social é por instalação, sem comprovação de titularidade Microsoft.
- Os testes de jogo verificam inicialização, não horas de gameplay nem todos os packs/loaders existentes.
- RAM abaixo de 100 MB e comparação com outros launchers não foram medidas nesta sessão; o portal identifica esses números como metas/referências.

A ativação remota continua adiada por instrução do usuário. Procedimento em [PUBLICACAO.md](PUBLICACAO.md).
