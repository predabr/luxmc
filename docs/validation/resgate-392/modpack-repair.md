# Reparo de modpacks após o teste da 3.9.2

Data: 13/10/2026.

O teste do usuário revelou outra ocorrência de `0xc00000fd` no launcher instalado, durante o reparo. Os comandos de reparo e importação ainda construíam futuros grandes no despacho do Tauri, apesar da correção anterior de `launch_game`. Esses comandos agora despacham tarefas pequenas; seus núcleos e a recuperação dos manifests usam futuros alocados na heap.

O ATM10 8.2 instalado contém duas entradas CurseForge para `computercraft`: projetos 282001/1676502, arquivos 5714512/8778582, versões declaradas 1.113.1/1.120.2. Ambas eram consideradas preferidas pela base de mods. A resolução considera as versões declaradas nos metadados, somente para arquivos fixados pelo pack, com um único ID, loaders iguais e uma versão semântica estável mais recente inequívoca. Arquivos ambíguos continuam bloqueados para revisão. A versão substituída e o manifesto original ficam em `.luxmc/removed-duplicates`; a entrada removida sai do manifesto ativo para que o reparo não recrie o conflito.

No RLCraft instalado, todos os arquivos previstos no manifesto estão presentes, mas o client e as natives do Minecraft 1.12.2 estão ausentes. Esse é um cenário necessário para o teste do reparo completo.

O reparo cria a pasta de mods quando ela está ausente, recupera os arquivos, resolve duplicatas e executa a auditoria antes de concluir. Falhas dos mods deixam de ser convertidas em um resultado aparentemente bem sucedido. O marcador `.healed` somente permite reaproveitamento quando o fingerprint da instância também está íntegro.

A conferência de arquivos usa até oito trabalhadores para leituras curtas, com progresso ordenado. A verificação CRC profunda continua serial. Foram adicionados tempos de cada etapa da inscrição e registros para arquivos cuja conferência leva mais de 500 ms. A medição anterior ao ajuste, no ATM10 instalado, levou 2,943 s para a auditoria completa de 491 JARs; o histórico da instalação, porém, registrou 310,973 s de inscrição/conferência. Uma medição em arquivos já acessados não explica nem comprova a melhora do cenário frio.

A barra integrada usa etapas emitidas pelo processo e tempo decorrido. O percentual fixo de 85% foi removido. O componente respeita redução de movimento e mantém o estado ao sair e voltar à instância.

A abertura captura a instância escolhida antes de aguardar as verificações. Navegar para outra instância não altera a versão solicitada, o perfil registrado no histórico nem os dados enviados ao Discord. A inicialização das estatísticas, que ocorria duas vezes nessa rota, agora acontece uma vez. O teste de interface inclui a troca para outra instância antes de concluir a abertura.

## Verificação inicial

- `pnpm check`: nenhum erro ou aviso.
- `pnpm build`: concluído.
- `cargo check`: concluído, com avisos de código existente não utilizado.
- Testes de navegador `launch-progress`, `launch-session` e `launch-startup`: passaram, com fixtures. Eles verificam navegação, cancelamento, espera pela janela, recuperação de erro e redução de movimento; não abrem modpacks reais.
- O executável dos testes Rust em release foi compilado, mas o Windows bloqueou sua execução: política `VerifiedAndReputableDesktop`, status `0xc0e90002`. Portanto os testes desse executável não foram executados localmente.
- O backup consistente da base local foi criado antes de qualquer reparo dos packs.

O roteiro `scripts/verify-installed-modpacks.mjs` executa os comandos de reparo e abertura das instâncias existentes, verifica a janela nativa, procura a inicialização dos recursos no log novo, observa o processo por 60 segundos e encerra somente o jogo iniciado pelo próprio teste. Não abre mundos nem exclui instâncias. A janela e os recursos inicializados não comprovam, por si só, funcionamento de todos os mods, skins, LAN ou mundos.

Os resultados de execução real e a entrega do candidato serão registrados depois da compilação e da tentativa de validação. Não há garantia de compatibilidade universal de modpacks nem de aprovação pelo Controle Inteligente de Aplicativos.
