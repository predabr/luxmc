# Revisão da release 4.0.0

Publicação suspensa por pedido do usuário em 14/10/2026. Não fazer push, iniciar workflows, publicar release ou implantar o site durante esta revisão. A versão pública continua a 3.9.5 original.

## Concluído no código

- [x] Versão 4.0.0 consistente em package, Cargo/lock, Tauri, Arch, instalador, traduções, website e workflows manuais.
- [x] Atualizador e site distinguem novas compilações da mesma 4.0.0; tags de build não substituem tags publicadas.
- [x] Revisão da importação: downloads oficiais, hashes, cancelamento, overrides, reparação e metadados de mods.
- [x] Importadores validam Minecraft e loader antes de baixar/extrair; não adivinham Minecraft 1.21.1 nem escolhem Fabric automaticamente sem informação.
- [x] Atualização reserva toda a operação no backend, inclusive o download; erros liberam a reserva. Teste nativo de falha/conflito passou no Windows.
- [x] Atualização verifica o projeto de origem e valida novamente os arquivos depois de preservar mods pessoais, antes da transação final.
- [x] Mod desativado continua desativado mesmo com novo nome de arquivo, por identidade exata dos módulos.
- [x] Mundos existentes não são mesclados com mundos padrão da atualização; a cópia padrão fica recuperável. Opções e servidores pessoais têm prioridade.
- [x] Proteção de destinos inclui links e reparse points do Windows; caminhos inválidos são rejeitados.
- [x] Histórico mantém versões anteriores e acrescenta notas disponíveis das releases recentes.
- [x] Estado global conserva instalações, atualizações e limpeza ao navegar; conferência no navegador da interface compilada.
- [x] Verificação de interface, site e metadados disponível localmente.

## Ainda obrigatório antes de lançar

- [x] cargo check --lib --locked e 48 testes Rust selecionados passaram no Windows na nova tentativa solicitada pelo usuário. O bloqueio 4551 deixou de ocorrer. Nenhuma proteção foi alterada pelo agente.
- [ ] Validar instalação, reparo, atualização e início real de Vanilla/Fabric/Forge/NeoForge/Quilt e modpacks representativos em dados descartáveis, incluindo os JARs reais do Terrorcraft. Testes de navegador usam backend simulado.
- [ ] Confirmar preservação de mundos e opções após atualização, falha e cancelamento com executável nativo da 4.0.
- [ ] Gerar instaladores, conferir versão e build embutidos, assinaturas de manifestos, hashes, instalação/atualização/desinstalação. A assinatura dos manifestos não é Authenticode.
- [ ] Finalizar notas da 4.0 e substituir o instalador do website: o arquivo local atual ainda é da 3.9.5. Não implantar o site com o número 4.0 apontando para esse arquivo antigo.
- [ ] Retomar GitHub/publicação somente após autorização do usuário e validações pendentes.

## Evidências locais desta revisão

- pnpm check: 0 erros, 0 avisos.
- Vitest: 215 testes passaram.
- Site: 93 testes passaram, incluindo identidade exata de builds 4.0.
- Manifestos/assinaturas: 13 testes passaram.
- pnpm build: concluído; avisos já conhecidos de divisão de chunks/importações.
- Rustfmt: os quatro módulos Rust alterados foram analisados sem erro de sintaxe. Isso não substitui cargo check/test.
- Interface compilada: limpeza durante navegação, recontagem, opção de animações, atualização de mesma versão e wallpaper passaram; histórico de 55 versões e progresso de modpack entre páginas também passaram.

Detalhes e continuidade em docs/validation/launcher-refinement/work-state.md. Não afirmar compatibilidade universal ou ganho de tempo de abertura sem medições nativas da versão revisada.

- Navegador em produção: controles do dono e conteúdo com 1000 mods em 1440/960 também passaram, incluindo falha ao salvar configurações, recuperação e troca de painéis.
- Animação 3D: teste em Vite de desenvolvimento passou para andar/correr/voar, modo de desempenho e jogo em execução. A instrumentação desse teste exige módulos de desenvolvimento; não executa sobre os chunks de produção.
- Teste de anúncio de instaladores no website passou em Python UTF-8. A suíte de instaladores Linux não executa neste Windows: fixtures exigem symlinks e ferramentas POSIX, falhando antes dos cenários com WinError 1314. Continua pendente em Linux.

## Nova tentativa nativa — 14/10/2026

- cargo check --lib --locked: passou, com avisos de código não utilizado.
- 48 testes selecionados passaram: atualização de modpacks (5), seleção de versões/loaders (1), histórico (1), auditoria de arquivos (12), download/hashes/cancelamento (11), armazenamento Windows (6), Shield (10), e dois testes explícitos dos JARs reais do Terrorcraft (2).
- O histórico legado contém duas entradas 0.2.0 com datas distintas. A asserção foi corrigida para conferir a chave versão/data, preservando ambos os registros.
- Arquivos reais GeckoLib e Schizo Cave Dweller conferidos novamente por SHA-256. Shield leu os dois arquivos sem alteração, com zero avisos; isso não certifica todos os mods nem substitui teste de abertura do jogo.
- Instalador local 4.0.0 gerado, build de teste 20261014.1 (revisão 20261014001), em release-windows/4.0.0-build.20261014.1/Lux MC Launcher.exe. Não publicado nem instalado. GitHub continua suspenso.

- pnpm tauri build --bundles nsis concluiu: executável otimizado e instalador NSIS com ProductVersion 4.0.0. O script NSIS inclui o executável principal e o bootstrapper WebView, sem Lux Repair.
- SHA-256 do instalador: e616c5108709b8428a5b203116eba7f24ac3d94b2d6f839c80bed30a49203e48. Executável: 95e914bc10ec52d01b3730ac912a98c7eed76aafd2eccc832beffbb264d8de3c.
- Auditoria de metadados em docs/validation/launcher-refinement/windows-4.0.0-local-audit.json. Authenticode: NotSigned. A varredura Defender não foi executada porque AMServiceEnabled e AntivirusEnabled estão false. A auditoria automática parou antes da varredura; não declarar o candidato aprovado pelo antivírus ou Windows.
- Ainda faltam a varredura do instalador e os testes reais de instalação/atualização/desinstalação e abertura dos jogos. O launcher instalado e o instalador do website continuam intactos.
