# Luxmc — refinamento da 3.9.5

## Escopo
Contagem e limpeza de armazenamento, conflitos de mods, transparência com wallpaper, animações configuráveis, ícones progressivos, avisos de atualização de modpacks, remoção acessível, GIFs Plus/Pro, atualizador por compilação e propostas de logo sem aplicação.

## Validação local
- pnpm check: zero erros e avisos.
- Vitest: 215 testes passaram.
- Site: 90 testes passaram; saída gerada de membership atualizada.
- Manifestos e assinaturas: 13 testes passaram.
- Navegador: mil mods em telas 1440/960; histórico de 55 versões; progresso de atualização entre páginas; GIFs Free/Plus/Pro; limpeza com execução suspensa até conclusão e recontagem forçada; mesma versão com compilação diferente, sem aviso repetido após instalação; painéis respeitam opacidade do wallpaper.
- cargo check debug falhou ao carregar dependências existentes; tentativa com alvo separado foi bloqueada por Controle de Aplicativo (erro 4551) no build-script do crossbeam-utils. cargo check --release confirmou o mesmo bloqueio em zerofrom_derive. Não desativamos a proteção. Rustfmt validou a sintaxe. Ainda falta compilação e testes nativos em ambiente de CI.

## Entrega
Nenhuma nova versão publicada ou instalada neste refinamento. Versão pública continua a 3.9.5 original, commit 772931fed5c7a7f5ae9d7e893620a66f29464dbb. Propostas de logo em docs/design/logo-proposals, fora dos assets do produto.

## Próximas verificações
Concluir suíte de navegador e CI Windows/Linux; analisar divergências e corrigir antes de preparar um instalador. Testar os JARs reais do Terrorcraft em cópia isolada, sem modificar as instâncias do usuário. Não prometer início instantâneo nem compatibilidade universal sem medir cada pacote.

## Continuação da validação
- Primeiro commit: 8e89c102a26abe94ae3be96cc6c1eb6213728137; branch codex/launcher-refinement-builds.
- Primeira CI: https://github.com/predabr/luxmc/actions/runs/38665330636. Linux passou nos 35 testes anteriores de navegador; o novo teste encontrou seleção de plataforma pelo navegador em vez do backend. Corrigido no código seguinte.
- A suíte local passou até catalog-navigation. social-presence falhou com o servidor Vite que havia recebido HMR durante a edição; repetir com servidor novo para evitar módulos duplicados por timestamp. Não marcar a suíte completa como aprovada antes disso.
- Limpeza de armazenamento agora usa estado global: preserva progresso ao sair da página e voltar, testado no navegador.
- Shield evita cópia temporária de JARs já conhecidos, mantendo hash do conteúdo e bloqueio de alteração com mesmo tamanho/mtime. Acrescentados tempos internos de Java/loader, nativos e aparência/Shield.
- Novos testes nativos ignorados por padrão têm execução explícita na CI com os JARs reais GeckoLib 3.1.40 e Schizo Cave Dweller 1.6.4 do provedor, verificados por tamanho e SHA-1 antes de gravar. Nunca executam os JARs.
- Novos arquivos deste bloco: src/lib/stores/storageActivity.svelte.ts e scripts/prepare-real-mod-fixtures.mjs.
- Os seis screenshots preexistentes foram copiados em Local/Luxmc-build/backups/refinement-existing-screenshots antes dos casos de catálogo e restaurados depois; capturas novas em requests/refinement-catalog-captures. Não incluir screenshots antigos nem mudanças vendor no commit.

## Novo ponto de verificação
- Segundo commit: db583945100005112fa8387cf672b897c301cbba.
- CI https://github.com/predabr/luxmc/actions/runs/38666709652: cargo check passou em Windows e Linux; cargo test em andamento na última consulta.
- pnpm check passou após o estado global e a escolha da plataforma nativa. pnpm build concluiu com sucesso.
- Servidor Vite reiniciado: social-presence voltou a passar, seguido de recuperação de perfil/GIF, aparência em três idiomas, perfis visíveis e partículas. O erro anterior era relativo ao servidor com HMR; a CI Linux limpa já havia passado nesse teste.
- GeckoLib e Cave Dweller reais conferidos independentemente por Python/tomllib, tamanhos e SHA-256 em real-archives.json. Ainda aguarda execução dos testes Rust reais na CI.
- Preparado scripts/verify-storage-native.mjs, restrito a GITHUB_ACTIONS Windows: testa o executável, revisão compilada, contagem com hardlinks, limpeza real em dados efêmeros e preservação de mundo/mod/opções. Não executar no PC do usuário.
- Workflow de release agora fixa o identificador de compilação como output de create-release para todos os SOs, inclusive em nova tentativa de jobs parcialmente falhos. Não alterar tags publicadas nem reutilizar instaladores com outra identidade.

## NOVA INSTRUÇÃO DO USUÁRIO — GitHub parado / preparar 4.0.0
O usuário pediu: parar o GitHub, revisar tudo principalmente modpacks, e preparar 4.0 como release estável para continuar entregando novas compilações na mesma versão. NÃO publicar/push/reativar workflows durante essa revisão.
- Execução 38667188974 cancelada e status confirmado cancelled. As anteriores 38666709652 e 38665330636 também canceladas.
- Último commit enviado: f53ad35323cdb03e969f459dfa3738171ff06102. Nenhuma release nova nem deploy de site neste refinamento; público permanece 3.9.5 original.
- Teste de interface de produção passou para armazenamento, navegação, animações, updater na mesma versão, modpacks e transparência. GIF Free/Plus/Pro também passou na versão compilada.
- Vite preview do build de produção permanece aberto em sessão exec 64868 (porta1420). Parar antes de novo pnpm check/build.
- Novo escopo: revisão de instalação/extração/validação/reparo/atualização/cancelamento/abertura de modpacks, compatibilidade loaders/Java, preservação de mundos/mods/opções, e metadados de versão 4.0.0. Não substituir a logo sem avaliação do usuário.
- GitHub é necessário só quando o usuário voltar a permitir os builds remotos; compilação local Rust continua bloqueada por Controle de Aplicativo do Windows (4551). Não contornar proteções.


## Revisão local 4.0.0 — sem envio ao GitHub

- Metadados locais alterados para 4.0.0. Workflows manuais preparados, sem executá-los. Não houve push, release, deploy nem instalação após a ordem de parada.
- Importação Modrinth deixa de usar 1.21.1 quando falta versão e rejeita loaders simultâneos/aliases com versões conflitantes antes de baixar ou gravar arquivos. CurseForge valida loader principal e usa Vanilla quando não há declaração nem escolha explícita.
- Atualização confirma o projeto vinculado, preserva opções/servidores, mantém mods desativados por identidade e valida a combinação final com os mods pessoais antes da transação. Mundos padrão com nomes iguais são separados para não misturar regiões com mundos do usuário. A pasta original permanece preservada.
- Atualização recalcula a contagem de JARs finais e invalida o tamanho antigo no perfil. Novos testes Rust cobrem opções gerenciadas, mundos com nomes iguais, troca de nome de mod desativado, conflitos com mod pessoal, seleção de loader e destinos com links.
- Histórico do launcher acrescenta notas locais das releases recentes sem remover o legado. As notas 4.0 continuam marcadas em preparação.
- Site reconhece identidade completa -build.run.attempt: um instalador hospedado de outra compilação não substitui o arquivo da compilação publicada.
- Verificações 4.0: pnpm check passou (0/0); 215 Vitest, 93 do site e 13 de manifestos/assinaturas passaram; pnpm build passou. Rustfmt analisou os quatro módulos alterados.
- cargo check --lib da 4.0 foi bloqueado ao carregar serde_with_macros no build script: Controle de Aplicativos, erro 4551. Os novos testes Rust NÃO foram executados. As aprovações da CI anterior não cobrem essas alterações.
- Interface de produção 4.0: launcher-refinement e modpack-followup passaram. Artefatos novos em Local/Luxmc-build/requests/review-4.0-browser e review-4.0-modpacks; não sobrescrever as capturas antigas do usuário.
- Teste personalizer-motion depende de importações de desenvolvimento (/src e nome do módulo skinview3d); tentativa sobre a saída de produção parou na instrumentação, com "Skinview3d não foi carregado". Reexecutar em Vite dev novo; não tratar como teste de movimento aprovado em produção.
- Plano detalhado em docs/plans/revisao-release-4.0.md. Website e instalador NÃO devem ser publicados ainda: website/releases/Lux MC Launcher.exe continua 3.9.5 apesar dos metadados locais 4.0.

- Encerramento dos testes de navegador 4.0: owner membership passou na interface de produção; instance-content passou com 1000 mods em 1440/960, ajustes integrados, filtragem, recuperação de erro ao salvar e painéis isolados.
- personalizer-motion reexecutado em Vite browser-test recém-aberto: PASS para poses de andar/correr/voar, modo de desempenho e jogo em execução. O erro anterior foi da instrumentação sobre bundles de produção. Ambos servidores de teste foram encerrados.
- A atualização agora reserva import_job durante TODO o fluxo, inclusive download, para impedir concorrência com outra instalação e não resetar cancelamento na passagem para o importador. Finalização libera a reserva também após erro; teste nativo acrescentado, ainda sem executar devido ao bloqueio 4551.
- tests/installers.test.py: cenários Linux não executaram neste Windows por WinError 1314 em symlinks de fixtures e encoding padrão cp1252. Teste isolado de anúncios do website passou usando Python -X utf8. Não mudar a segurança do sistema nem declarar a suíte Linux aprovada.
- Próximo passo obrigatório: validar a fonte atual da 4.0 nativamente em ambiente autorizado antes de gerar/distribuir instalador. GitHub continua parado. Não usar as CI antigas como aprovação das mudanças novas; não publicar o instalador 3.9.5 com rótulo 4.0.

## Nova tentativa após alteração informada pelo usuário — 14/10/2026
- cargo check --lib --locked continuou bloqueado por Controle de Aplicativos (4551), DLL serde_with_macros no build script.
- Consulta somente de leitura: Get-MpComputerStatus retornou SmartAppControlState=On, AntivirusEnabled=True e RealTimeProtectionEnabled=False. Eventos locais CodeIntegrity 3077/3033 confirmaram bloqueio da DLL ao ser carregada pelo rustc, política 0283ac0f-fff1-49ae-ada1-8a933130cad6.
- Nenhuma proteção foi alterada pelo agente. GitHub/publicação continuam suspensos; nenhum instalador 4.0 gerado ou instalado.


## Tentativa seguinte liberou a compilação — 14/10/2026

- A pedido do usuário, cargo check --lib --locked foi repetido e PASSOU. Nenhuma configuração de proteção foi alterada pelo agente.
- O executável dos testes Rust foi compilado e executado. 48 testes selecionados passaram: modpack_update 5, pack targets 1, changelog 1, metadata audit 12, pack_download 11, storage 6, Shield 10 e fixtures reais explícitas 2.
- Única falha inicial: o teste de histórico assumia versões únicas, mas o histórico anterior já tinha duas 0.2.0 com datas diferentes. Ajustado o teste para unicidade de versão/data, conforme a chave do frontend, preservando as duas entradas; passou na repetição. Nenhum dado histórico foi removido.
- Os hashes dos JARs reais do Terrorcraft coincidem com real-archives.json. Os dois testes ignorados por padrão foram executados explicitamente contra requests/refinement-real-mods, sem executar os JARs. Resultado Shield: 2 arquivos, zero avisos; primeira leitura 349 ms e com cache 98 ms (apenas esse fixture).
- Logs das suítes em Local/Luxmc-build/requests/review-4.0-native/results.log; esse arquivo mantém também a falha inicial já corrigida. Windows cargo check foi repetido após a correção e passou.
- pnpm tauri build --bundles nsis em andamento na sessão 21596. CARGO_TARGET_DIR=Local/Luxmc-build/target; LUXMC_BUILD_ID=20261014.1. Build local de teste, não um release publicado. Frontend de produção recompilado com sucesso.
- GitHub permanece parado. Não instalar automaticamente, não atualizar website/releases nem publicar esse candidato antes das validações nativas de abertura e instalador.

## Instalador local 4.0.0 concluído — 14/10/2026

- Sessão 21596 concluiu com sucesso: release otimizado (9m55s) e NSIS. ProductVersion/FileVersion do executável e instalador: 4.0.0.
- Cópia isolada em release-windows/4.0.0-build.20261014.1/Lux MC Launcher.exe. Não sobrescreve o instalador 3.9.5 de release-windows nem website/releases; não foi instalado nem iniciado.
- Build local 20261014.1; revisão 20261014001. SHA-256 instalador e616c5108709b8428a5b203116eba7f24ac3d94b2d6f839c80bed30a49203e48; binário 95e914bc10ec52d01b3730ac912a98c7eed76aafd2eccc832beffbb264d8de3c.
- NSIS gerado conferido: somente principal e bootstrapper/runtime WebView nas diretivas File. Sem inclusão do antigo luxmc-repair.exe que ainda existe como artefato histórico no diretório de build; esse arquivo histórico não foi alterado.
- scripts/audit-windows-release.ps1 tentou validar e parou ANTES de varrer: Defender indisponível. Get-MpComputerStatus confirmou AMServiceEnabled=false, AntivirusEnabled=false, RealTimeProtectionEnabled=false. Nenhuma alteração de proteção foi feita pelo agente.
- Get-AuthenticodeSignature: NotSigned para instalador e executável. Não há confirmação de confiança do Windows ou aprovação de antivírus. Relatório de metadados/pendências salvo em windows-4.0.0-local-audit.json.
- GitHub segue parado. Nenhuma publicação, deploy, instalação ou abertura de jogos. Próximas validações: varredura em ambiente com antivírus ativo, instalação/atualização/desinstalação e início real dos jogos; o candidato local não é uma release pronta para distribuição.

## Instalação autorizada pelo usuário e limpeza — 15/10/2026

- O usuário pediu instalar a versão mais nova e tirar os antigos. A instrução anterior de não instalar automaticamente foi superada por esse pedido explícito.
- Instalador conferido por SHA-256 e copiado para Downloads/Lux MC Launcher - 4.0.0.exe. Upgrade executado com /S /UPDATE no diretório já registrado, preservando os dados. Nenhum launcher/jogo estava em execução no início.
- Registro e PE do programa instalado confirmam 4.0.0; atalhos da área de trabalho e menu Iniciar apontam para AppData/Local/Luxmc/luxmc.exe.
- Binário instalado SHA-256 c7052576fde927d062d941860e4199e1ac0b3bbe19d37891486c2f0692535dd8. A diferença de 3 bytes para o binário bruto é EXATAMENTE o marcador __TAURI_BUNDLE_TYPE_VAR_UNK para __TAURI_BUNDLE_TYPE_VAR_NSS inserido pelo empacotamento Tauri; equivalência integral verificada após normalizar apenas esse marcador. Não modificar o executável instalado para remover o marcador.
- Banco permaneceu byte a byte igual; 5 perfis e 2 contas preservados. Dois level.dat verificados por SHA-256 também permaneceram iguais.
- Após confirmar a instalação, foram removidos 13 instaladores antigos do Luxmc em Downloads, validados por nome, ProductName, versão e hash. Mantido o novo 4.0.0 e arquivos pessoais, incluindo GIFs. Removido luxmc.repair-stage antigo depois de conferir sua cópia de recuperação.
- Recuperação privada em Local/Luxmc-build/backups/upgrade-4.0.0-20261014: programa e desinstalador anteriores, estágio antigo de reparo, backup SQLite consistente e relatórios before/after/cleanup. Não publicar o banco nem esses arquivos privados.
- Evidência resumida em installed-4.0.0-local.json. A auditoria windows-4.0.0-local-audit.json registra o estado do candidato ANTES da instalação; este registro atualiza o estado instalado.
- GitHub e site continuam sem publicação. Não foram alteradas proteções, executados jogos nem declarada aprovação do antivírus; Defender continua com varredura pendente.
