# Revisão da release 4.0.0

Publicação suspensa por pedido do usuário em 14/10/2026. Não fazer push, iniciar workflows, publicar release ou implantar o site durante esta revisão. A versão pública continua a 3.9.5 original.

## Concluído no código

- [x] Versão 4.0.0 consistente em package, Cargo/lock, Tauri, Arch, instalador, traduções, website e workflows manuais.
- [x] Atualizador e site distinguem novas compilações da mesma 4.0.0; tags de build não substituem tags publicadas.
- [x] Revisão da importação: downloads oficiais, hashes, cancelamento, overrides, reparação e metadados de mods.
- [x] Importadores validam Minecraft e loader antes de baixar/extrair; não adivinham Minecraft 1.21.1 nem escolhem Fabric automaticamente sem informação.
- [x] Atualização reserva toda a operação no backend, inclusive o download; erros liberam a reserva. Teste nativo de falha/conflito acrescentado, execução pendente.
- [x] Atualização verifica o projeto de origem e valida novamente os arquivos depois de preservar mods pessoais, antes da transação final.
- [x] Mod desativado continua desativado mesmo com novo nome de arquivo, por identidade exata dos módulos.
- [x] Mundos existentes não são mesclados com mundos padrão da atualização; a cópia padrão fica recuperável. Opções e servidores pessoais têm prioridade.
- [x] Proteção de destinos inclui links e reparse points do Windows; caminhos inválidos são rejeitados.
- [x] Histórico mantém versões anteriores e acrescenta notas disponíveis das releases recentes.
- [x] Estado global conserva instalações, atualizações e limpeza ao navegar; conferência no navegador da interface compilada.
- [x] Verificação de interface, site e metadados disponível localmente.

## Ainda obrigatório antes de lançar

- [ ] Compilar e executar os novos testes Rust da 4.0.0 em ambiente autorizado. O cargo check local foi bloqueado pelo Controle de Aplicativos do Windows, erro 4551; não desativar nem contornar a proteção.
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
