# Reparo de modpacks após o teste da 3.9.2

Data: 13/10/2026.

O teste do usuário revelou outra ocorrência de `0xc00000fd` no launcher instalado, durante o reparo. Os comandos de reparo e importação ainda construíam futuros grandes no despacho do Tauri, apesar da correção anterior de `launch_game`. Esses comandos agora despacham tarefas pequenas; seus núcleos e a recuperação dos manifests usam futuros alocados na heap.

O ATM10 8.2 instalado contém duas entradas CurseForge para `computercraft`: projetos 282001/1676502, arquivos 5714512/8778582, versões declaradas 1.113.1/1.120.2. Ambas eram consideradas preferidas pela base de mods. A resolução considera as versões declaradas nos metadados, somente para arquivos fixados pelo pack, com um único ID, loaders iguais e uma versão semântica estável mais recente inequívoca. Arquivos ambíguos continuam bloqueados para revisão. A versão substituída e o manifesto original ficam em `.luxmc/removed-duplicates`; a entrada removida sai do manifesto ativo para que o reparo não recrie o conflito.

No início da revisão, todos os arquivos previstos no manifesto do RLCraft estavam presentes, mas o client e as natives do Minecraft 1.12.2 estavam ausentes. Esse cenário foi usado para o teste do reparo completo.

O reparo cria a pasta de mods quando ela está ausente, recupera os arquivos, resolve duplicatas e executa a auditoria antes de concluir. Falhas dos mods deixam de ser convertidas em um resultado aparentemente bem sucedido. O marcador `.healed` somente permite reaproveitamento quando o fingerprint da instância também está íntegro.

A conferência de arquivos usa até oito trabalhadores para leituras curtas, com progresso ordenado. A verificação CRC profunda continua serial. Foram adicionados tempos de cada etapa da inscrição e registros para arquivos cuja conferência leva mais de 500 ms. A medição anterior ao ajuste, no ATM10 instalado, levou 2,943 s para a auditoria completa de 491 JARs; o histórico da instalação, porém, registrou 310,973 s de inscrição/conferência. Uma medição em arquivos já acessados não explica nem comprova a melhora do cenário frio.

A barra integrada usa etapas emitidas pelo processo e tempo decorrido. O percentual fixo de 85% foi removido. O componente respeita redução de movimento e mantém o estado ao sair e voltar à instância.

A abertura captura a instância escolhida antes de aguardar as verificações. Navegar para outra instância não altera a versão solicitada, o perfil registrado no histórico nem os dados enviados ao Discord. A inicialização das estatísticas, que ocorria duas vezes nessa rota, agora acontece uma vez. O teste de interface inclui a troca para outra instância antes de concluir a abertura.

## Verificação inicial

- `pnpm check`: nenhum erro ou aviso.
- `pnpm build`: concluído.
- `cargo check`: concluído, com avisos de código existente não utilizado.
- Testes de navegador `launch-progress`, `launch-session` e `launch-startup`: passaram, com fixtures. Eles verificam navegação, cancelamento, espera pela janela, recuperação de erro e redução de movimento; não abrem modpacks reais.
- O executável dos testes Rust em release foi compilado, mas o Windows bloqueou sua execução: política `VerifiedAndReputableDesktop`, status `0xc0e90002`. Os testes desse executável não foram executados localmente. Os mesmos testes passaram no Windows e no Linux no CI dos commits `4bb66e6` e `d9bd095`, incluindo o despacho dos reparos em uma thread com pilha de 1 MiB.
- O backup consistente da base local foi criado antes de qualquer reparo dos packs.

O roteiro `scripts/verify-installed-modpacks.mjs` executa os comandos de reparo e abertura das instâncias existentes, verifica a janela nativa, procura a inicialização dos recursos no log novo, observa o processo por 60 segundos e encerra somente o jogo iniciado pelo próprio teste. Não abre mundos nem exclui instâncias. A janela e os recursos inicializados não comprovam, por si só, funcionamento de todos os mods, skins, LAN ou mundos.

## Execução real no Windows

O candidato `4bb66e6` executou reparos e abriu os jogos pelos comandos nativos, com a conta selecionada na instalação local. Os processos iniciados pelos testes foram encerrados ao fim da observação. As três instâncias originais foram preservadas.

| Instância | Resultado observado |
| --- | --- |
| ATM10 8.2, Minecraft 1.21.1, NeoForge 21.1.251 | Reparo em 19,12 s; uma duplicata substituída com backup. Janela em 113,41 s na preparação inicial do loader, recursos em 292,45 s; permaneceu aberto por mais 60 s. O log também confirma a tela de título e o término da inicialização pelo ModernFix. |
| RLCraft, Minecraft 1.12.2, Forge 14.23.5.2860 | Client e natives recuperados no reparo de 22,41 s. Na repetição, janela em 15,64 s, mods completamente inicializados e recursos em 317,47 s; permaneceu aberto por mais 60 s. Havia uma compilação Rust concorrente, portanto esse tempo não representa uma medição isolada de desempenho. |
| Vanilla 26.3 | Janela em 20,22 s, recursos em 22,26 s; permaneceu aberto por mais 60 s. |
| Fabulously Optimized 6.5.0, Minecraft 1.21.1, Fabric | No candidato `d9bd095`, instalação em 6,36 s, 50 arquivos verificados por hash, nenhum bloqueio ou aviso de prontidão. Preparação inicial em 78,65 s, recursos em 82,72 s; permaneceu aberto por mais 8 s. A instância temporária foi removida e as originais preservadas. |

O primeiro roteiro interrompeu o RLCraft ao encontrar uma mensagem `FATAL` de integração opcional com JourneyMap. A mensagem não encerrava o jogo. O critério foi corrigido para reconhecer falhas efetivas de resolução/carregamento e exigir também os marcadores de inicialização dos mods; o teste completo do RLCraft passou depois dessa correção.

Uma importação nova do arquivo oficial do ATM10, com 495 entradas, concluiu em 236,97 s: arquivos em 33,83 s e inscrição/conferência em 203,12 s. A duplicata foi resolvida antes de concluir a instalação, o manifesto ativo ficou com 494 entradas e a instância temporária foi removida. O histórico anterior registrava 310,97 s nessa etapa; a redução observada não resolve o atraso restante nem estabelece uma comparação de desempenho isolada. O daemon agora permite diagnóstico das etapas em stderr sem misturá-lo ao protocolo JSON, para medir a auditoria, o fingerprint e a gravação da biblioteca separadamente.

O roteiro `tests/modpack-repair-windows-native.cjs` também executou esses reparos pelos handlers reais do Tauri, através do WebView do aplicativo empacotado. RLCraft concluiu em 3,34 s e ATM10 em 15 s, sem encerramento do launcher. Uma nova importação do ATM10 por esse caminho concluiu em 274,07 s, resolveu a duplicata e preservou as instâncias originais. A auditoria inicial levou 217,84 s; auditoria, resolução e fingerprint somaram 241,94 s; gravação dos essenciais e banco levou 0,39 s. Durante essa operação, uma amostra de 40 timers de 50 ms do WebView teve intervalo máximo de 53 ms. Essa amostra verifica responsividade nesse intervalo, não fluidez de todas as abas e animações.

Esse diagnóstico revelou a amplificação de leitura causada pelo `BufReader::seek`, que descartava o buffer mesmo nos pequenos saltos do índice ZIP. O leitor passa a manter a posição lógica e usar `seek_relative`, reaproveitando bytes já lidos. Saltos fora do buffer e relativos ao fim mantêm a semântica de arquivo, e continuam ativas as verificações de alteração, cancelamento, metadados e CRC profundo. Um teste verifica o reaproveitamento e posições de leitura no início, meio, fim e em deslocamentos inválidos.

No candidato `6608d52`, os reparos nativos passaram em 1,93 s para RLCraft e 3,58 s para ATM10. A importação, porém, ainda levou 236,32 s: auditoria em 199,39 s, auditoria/resolução/fingerprint em 204,49 s e essenciais/banco em 0,14 s. A interface registrou 4.723 ticks de 50 ms durante a importação, com intervalo máximo de 208 ms. O buffer melhorou os reparos, mas não resolveu a primeira leitura dos arquivos recém-gravados. Essa medição motivou outra alteração, em vez de declarar a otimização concluída.

A importação agora reaproveita, apenas em memória, os descritores extraídos dos mesmos bytes já verificados por hash e gravados atomicamente. Downloads CurseForge sem SHA-1 esperado continuam pelo caminho normal. O cache é limitado, inclui tamanho, modificação e criação do arquivo e é consultado somente na auditoria rápida de mods; alterações invalidam o reaproveitamento. A validação profunda ignora esse cache e continua conferindo CRC. A verificação dos hashes de download e a proteção do Windows não foram removidas. Os testes incluem invalidação após corrupção, troca por outro arquivo do mesmo tamanho com nova data de modificação e CRC profundo com dados previamente registrados no cache.

No candidato final `8e42e48`, o teste nativo repetiu a importação do mesmo arquivo oficial do ATM10. A operação inteira concluiu em 38,97 s. A auditoria levou 1,13 s; auditoria, resolução e fingerprint somaram 6,25 s; essenciais e banco levaram 0,33 s. A etapa final passou de aproximadamente 204,63 s no candidato anterior para 6,59 s neste teste. Essa é uma comparação local de imports do mesmo pacote com downloads reutilizados, não uma promessa de velocidade para qualquer rede ou computador.

O mesmo candidato reparou RLCraft em 1,84 s e ATM10 em 3,74 s pelos handlers nativos, sem encerrar o launcher. A duplicata do ATM10 foi resolvida durante a importação nova. A interface registrou 776 ticks de 50 ms durante toda a operação, com intervalo máximo de 140 ms. A instância temporária foi excluída e as três instâncias originais preservadas.

Uma importação nova do Fabulously Optimized 6.5.0 nesse candidato concluiu em 1,02 s, com 50 arquivos verificados por hash. O Minecraft Fabric abriu, confirmou os recursos em 77,93 s e permaneceu ativo na observação; a preparação até a janela levou 73,85 s. O ganho de importação não deve ser apresentado como uma redução equivalente no carregamento dos mods pelo jogo.

Uma execução do CI no Windows falhou na medição imediata do layout das capas; o mesmo teste passou localmente no build e no servidor de desenvolvimento. A regressão mantém a exigência de imagem preenchendo toda a área, sem padding e com `object-fit: cover`, mas agora aguarda também esses estilos e dimensões antes de medir. Não foi alterado o visual das capas para contornar o teste.

Na abertura final pelo candidato `8e42e48`, o Vanilla 26.3 confirmou a janela em 15,55 s e os recursos em 17,58 s. O RLCraft confirmou a janela em 19,36 s, mas precisou de 370,67 s até completar a inicialização dos mods e recursos. Os dois permaneceram ativos por mais 60 s. A demora dos mods do RLCraft continua existindo e não foi classificada como resolvida apenas porque o launcher deixou de encerrar.

O instalador desse candidato foi aplicado localmente com saída zero. O programa instalado corresponde ao executável extraído do instalador; os hashes dos arquivos de configuração e da base foram preservados durante a instalação. A verificação SQLite retornou `ok`, com três instâncias e duas contas. O backup anterior à instalação e o instalador de teste foram preservados. Os componentes, hashes e resultados estão em `candidate-modpack-repair.json` e `modpack-real-tests.json`. A versão continua sendo um candidato 3.9.2, sem nova publicação pública.

Não há garantia de compatibilidade universal de modpacks nem de aprovação pelo Controle Inteligente de Aplicativos. Esses testes não abrem mundos nem substituem a validação de LAN em dois computadores.
