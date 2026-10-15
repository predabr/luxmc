# Revisão da 4.0 — 15/10/2026

O usuário autorizou novamente a publicação da 4.0 no GitHub, após os ajustes desta revisão. Não substituir uma compilação pública sem mudar sua identidade de build.

## Correções

- Medição de armazenamento usa identidades de arquivos obtidas por diretório no Windows, com fallback para sistemas sem suporte. A leitura roda fora da interface; hard links são deduplicados e o relatório é reutilizado por até dois minutos.
- Instalação e reparo atualizam a quantidade real de JARs e invalidam o relatório. A interface conserva o relatório entre páginas; a limpeza não reinicia ao navegar.
- Modrinth ordena as versões por publicação. A verificação de atualização considera a edição de Minecraft, loader e versão instalada; não oferece um downgrade como atualização. A conclusão da instalação invalida respostas antigas em andamento.
- Ícones locais são antecipados em lotes, com duas tarefas, cache limitado a 6 MiB e pausa enquanto o jogo inicia ou está aberto. O primeiro resultado da lista já pode trazer até 1 MiB de miniaturas salvas.
- O seletor usa a camada de popover quando disponível, posição limitada à janela e portal dentro do diálogo quando necessário para preservar o foco.
- A seleção de skin abre confirmação central sem aplicar automaticamente. Continuar editando preserva a alteração pendente. O renderizador e as animações 3D existentes foram mantidos.
- Login, consulta de plano e desinstalador receberam correções de apresentação e preservação de dados. A identidade social é incluída nos próximos backups SQLite.

## Evidência real local

Os quatro modpacks recuperados tinham manifestos e configurações, mas os JARs não estavam completos após a reinstalação. O reparo nativo foi concluído para ATM10, Zombie Invade, RLCraft e Better MC. Essa conferência não executa o Minecraft e não substitui a confirmação do menu de cada jogo.

Após o reparo, o ATM10 contém 4.944 arquivos, 490 JARs diretamente em `mods` e 3.065.272.807 bytes únicos por identidade de arquivo. A nova leitura nativa encontrou esse valor em 2.236 ms; uma leitura independente em Python confirmou os mesmos bytes. O caminho antigo que abre cada arquivo individualmente ficou bloqueado na comparação e foi interrompido; não foi calculado um percentual de ganho.

Uma cópia privada do banco foi salva antes de atualizar os tamanhos e contagens das instâncias recuperadas. Contas, amizades, mundos existentes e assinatura não foram apagados. Os arquivos dos mundos anteriores que já estavam ausentes após a desinstalação não foram recuperados por este reparo.

## Verificação do launcher e da publicação

- `pnpm check`: sem erros ou avisos.
- Vitest: 35 arquivos e 216 testes passaram após corrigir o fallback do seletor sem popover.
- Site: 101 testes passaram, incluindo os oito cenários de login Microsoft.
- Manifesto, assinatura e profundidade de padrões: 16 testes passaram.
- Teste visual novo: seletor longo dentro da janela, seleção por teclado e confirmação da skin apenas com ação explícita, em 1440 e 960 pixels.
- A bateria de navegador passou em execuções consecutivas retomadas após as correções: os 36 casos anteriores e os dois novos casos de recuperação e revisão da 4.0. O caso de wallpaper também foi repetido com o original indisponível, cópia interna persistida e nova porta após reiniciar. O teste separado de NameMC passou com a confirmação central.
- Rust: 265 testes passaram, nove integrações optativas ficaram ignoradas. O teste nativo explícito de armazenamento confirmou os mesmos 3.065.272.807 bytes em 1.095 ms com o cache do sistema aquecido. Os três testes específicos de seleção de cliente/servidor e comparação de atualizações também passaram após a última alteração Rust.
- Compilação de produção do frontend concluída. Os instaladores finais foram publicados e a conferência pública de arquivos, assinaturas e downloads do site foi concluída.

O armazenamento também inclui agora os arquivos mantidos no diretório de dados local do sistema, como wallpapers. Eles não pertencem ao cache descartável. O registro de origem dos wallpapers é interno e contém apenas a referência ao nome da cópia e seu tamanho; ele permite resolver um caminho antigo cujo original já foi removido. O teste nativo usa um MP4 real e confirma os bytes da cópia após remover o original e o cache de teste.

O login Microsoft do site depende das configurações Entra/Cloudflare descritas em `docs/plans/login-microsoft-site.md`; testes simulados não comprovam o login de produção. Nenhuma configuração de proteção do Windows foi alterada nesta revisão. Assinatura de manifesto não é assinatura Authenticode.


## Conferência do aplicativo instalado

A compilação Windows `20261015.2`, do commit `c0de29fc9300e171517ce7729449fd994f550386`, foi instalada e reaberta em 15/10/2026. O executável instalado corresponde integralmente ao binário compilado com a alteração esperada do marcador de pacote NSIS. O arquivo de autenticação permaneceu idêntico; uma nova cópia privada preservou as sete instâncias presentes antes da instalação.

Com o vídeo original ausente de Downloads, a interface nativa exibiu o wallpaper recuperado do armazenamento interno, OWNER, Pro e os três amigos. A grade exibiu o aviso de atualização do ATM10 em linha própria, com os controles dentro do cartão. A página de armazenamento mostrou 2,85 GiB para o ATM10 e sete instâncias.

A consulta nativa aos provedores encontrou uma atualização real de ATM10 8.2 para 8.3. Zombie Invade, RLCraft, Better MC, Fabulously Optimized e Pixelmon não receberam aviso de atualização. Essa consulta não instala atualizações e não comprova a abertura do menu do jogo.

## Login Microsoft em produção

O retorno Web foi cadastrado no aplicativo existente do Entra, preservando o retorno nativo. Uma credencial de seis meses, autorizada pelo usuário, foi armazenada exclusivamente como segredo de produção no Cloudflare Pages. A migração Microsoft foi aplicada; o site foi publicado com o botão habilitado sem anunciar antecipadamente o novo instalador 4.0.

O usuário identificou a conta do Spect3rBW. A tentativa real, iniciada a partir da sessão original OWNER/Pro, passou pela Microsoft e Xbox/XSTS. O endpoint Minecraft recusou a conexão do servidor Cloudflare com HTTP 403 e uma resposta de bloqueio do provedor. O login Microsoft completo e o vínculo em produção continuam pendentes; a configuração do Entra e os testes simulados não foram apresentados como prova de sucesso.


## Abertura real de jogos na versão instalada

A checagem nativa usou a conta selecionada e as instâncias existentes, sem entrar em mundos. Todos os processos iniciados pela checagem foram encerrados ao final, e os sete IDs de perfil foram preservados. A detecção de recursos nos logs e a permanência do processo por mais oito segundos não comprovam todos os menus; o menu do ATM10 foi também observado na janela real.

| Instância | Retorno da chamada de abertura | Recursos inicializados |
| --- | ---: | ---: |
| Vanilla 26.3 | 19.6 s | 21.8 s |
| Fabulously Optimized | 28.3 s | 31.4 s |
| RLCraft | 75.7 s | 143.6 s |
| Zombie Invade | 184.0 s | 248.9 s |
| Zombie Invade warm | 45.6 s | 139.6 s |
| ATM10 NeoForge | 100.9 s | 240.4 s |

O tempo de retorno da chamada inclui preparação e espera pela janela, não representa exclusivamente CPU do launcher. O primeiro Zombie Invade ainda precisava da preparação do Forge; a repetição com os arquivos presentes foi mais curta. Não há comparação equivalente com CurseForge neste computador e não foi afirmado um percentual de ganho. Os pacotes pesados continuam levando minutos para carregar seus recursos. Better MC e Pixelmon não fizeram parte desta checagem de abertura; o primeiro teve reparo real de arquivos e ambos tiveram consulta real de atualização.


## Publicação e instalação oficial concluídas

A release [v4.0.0](https://github.com/predabr/luxmc/releases/tag/v4.0.0) foi publicada em 15/10/2026 pela [execução 38781690526](https://github.com/predabr/luxmc/actions/runs/38781690526), tentativa 2. Todos os jobs passaram, incluindo a bateria completa de 38 casos de navegador, Rust nas plataformas exigidas, varredura do Defender e ciclo de instalação/reinstalação/desinstalação no Windows. O manifesto e os checksums foram assinados com a chave oficial de release.

A verificação pública confirmou os sete instaladores e os quatro arquivos de manifesto/assinatura. Os links das seis plataformas do site apontaram para esta release. O arquivo Windows servido pelo site tem 21.594.688 bytes e SHA-256 `12c11cc8dd77590003d1662f6449a2d277cd94e9d7bf3627c60be05b5921a7ab`, idêntico ao GitHub e ao arquivo usado no teste de instalação do runner. O relatório público está em `docs/validation/release-4.0-public.json`.

O site foi publicado no deployment `94f1d26a`, com o instalador oficial hospedado localmente, metadados 4.0.0 e a mensagem explícita do bloqueio do serviço Minecraft na autenticação web. A página pública foi conferida no navegador.

O instalador oficial foi aplicado neste PC e o launcher reaberto. Uma nova cópia privada preservou banco, autenticação, preferências e identidade social. A autenticação permaneceu byte a byte idêntica durante a instalação; os sete perfis, OWNER e Pro permaneceram presentes. O programa instalado corresponde ao executável auditado no runner, considerando apenas a alteração documentada do marcador Tauri para NSIS. O wallpaper interno continua disponível, com o original de Downloads ausente.

A varredura do Defender encontrou zero ameaças no instalador e programa. Ambos continuam `NotSigned` em Authenticode; esse resultado não comprova aceitação pelo Smart App Control nem por outros fornecedores de antivírus. Nenhuma proteção do Windows foi alterada.

As correções posteriores exclusivas do site passaram por `pnpm check` sem erros/avisos e 104 testes do site, incluindo redirecionamentos rejeitados sem envio de credenciais e bloqueio explícito sem criar identidade não verificada. O usuário confirmou que não dispõe de outro backend. O login Microsoft web continua pendente de liberação do serviço Minecraft ou de infraestrutura apropriada autorizada; não foi apresentado como concluído.
