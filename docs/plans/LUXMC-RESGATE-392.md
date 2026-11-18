# LUXMC-RESGATE-392 — Recuperação completa da inicialização do Minecraft

Preparado em 13/10/2026, para retomada em 14/10/2026, quando Pedro solicitar a implementação.

Palavra-chave: **LUXMC-RESGATE-392**.

Pedido de retomada: **LUXMC-RESGATE-392: implemente o plano completo no projeto Luxmc.**

Ao retomar, ler este arquivo, conferir o estado atual do projeto e da instalação e continuar a partir das evidências. Este documento organiza a implementação futura; sua criação não inicia uma execução agendada.

## Objetivo e prioridade

Corrigir o encerramento do Luxmc ao clicar em Jogar e comprovar a abertura real do Minecraft pelo launcher instalado. Começar pelo caso Vanilla 26.3 da versão 3.9.2 e depois exercitar as famílias de versões, loaders e modpacks suportados.

O launcher deve permanecer disponível durante a preparação, apresentar erros úteis e manter a sala P2P. A correção precisa preservar mundos, contas, perfis, skins, capas, modpacks e configurações. Melhorias estéticas adicionais ficam depois da recuperação da inicialização.

O critério de sucesso é o Minecraft chegar ao menu e abrir um mundo por meio do executável empacotado. PID vivo, resposta do comando, testes simulados ou dois segundos de espera não comprovam esse resultado.

## Evidências já obtidas

- Pedro atualizou para 3.9.2, criou Vanilla 26.3 e relatou que clicar em Jogar fecha o launcher sem abrir o jogo. Também relata falha com loaders e modpacks.
- O executável instalado informa versão 3.9.2.
- O log Application do Windows registrou falha de `luxmc.exe` em 12/10/2026 às 21:55:09, horário de São Paulo. Código de exceção: `0xc00000fd`. Módulo: `luxmc.exe`. Deslocamento: `0x1b01957`.
- Segundo a [documentação da Microsoft](https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/debugging-a-stack-overflow), esse código representa `STATUS_STACK_OVERFLOW`, esgotamento da pilha de execução. O evento identifica a categoria de falha, mas não determina a função responsável.
- A instância Vanilla 26.3 foi criada às 21:55:07, dois segundos antes desse evento. A proximidade temporal é compatível com o relato, mas deve ser confirmada na reprodução instrumentada.
- Na inspeção, a pasta dessa instância continha `.luxmc`, `resourcepacks` e `options.txt`; não havia pasta de logs do jogo. Isso sugere falha antes de um início normal do Minecraft, sem provar a etapa exata.
- Não foi encontrado `panic.log` nos diretórios de logs inspecionados. Um crash nativo pode encerrar o processo sem passar pelo tratamento de erros e eventos da interface.
- O SHA-256 do instalador oficial disponível em Downloads é `e651194377fee9c1c97c3c4814979d16a6e8261ebf8ca71cc4049ef1c8ed69ee`.
- O SHA-256 do executável instalado é `b17782ec396c1ab834fc2f3aafc160516052e57fe60a6f9dcc580da4328c54cc`. A leitura do `luxmc.exe` dentro desse instalador, sem executá-lo, produziu exatamente o mesmo hash e 43.317.760 bytes. Portanto, o binário inspecionado corresponde ao conteúdo do pacote oficial.
- O relatório anterior do Defender identifica o executável da pasta de compilação com outro hash. Na próxima validação, conferir também o executável efetivamente empacotado e instalado, documentando diferenças introduzidas pelo empacotamento.
- Em `src-tauri/src/core/launcher/startup.rs`, `confirm_startup` considera bem-sucedido um processo que não encerra dentro do período de tolerância. Em `core/launcher/mod.rs`, esse período é de dois segundos antes de emitir `LaunchStage::Running`.
- `src/lib/utils/launcherLifecycle.ts` já espera uma janela antes de ocultar o launcher no Windows. `core/process.rs` reconhece qualquer janela visível pertencente ao PID; isso exige revisão para evitar confundir uma janela de erro com o jogo pronto.
- A 3.9.2 passou pelos testes automatizados de código e instalação. Essa cobertura não detectou o crash observado na utilização real. O plano deve fechar essa lacuna antes de chamar outra versão de corrigida.

## 1. Preservar dados e reproduzir o erro real

1. Conferir novamente versão, hash do pacote, hash do executável instalado, caminho da instalação e PID. Considerar atualizações feitas após a elaboração deste plano.
2. Criar backup consistente do banco, configurações e metadados de instâncias, tratando SQLite e seus arquivos associados corretamente. Preservar os mundos e recursos; testar em cópias ou instâncias dedicadas.
3. Registrar uma linha de base da instalação existente. Evitar limpeza de caches, reinstalação em massa ou redefinição de perfis antes de localizar a falha.
4. Reproduzir Vanilla 26.3 pelo launcher nativo, primeiro mantendo sua janela aberta. Comparar com a opção de ocultar durante o jogo para distinguir encerramento de processo, ocultação e falha do Java.
5. Acompanhar o PID do launcher e a árvore de processos Java. Correlacionar horário, etapa, evento do Windows e existência dos logs da instância.
6. Produzir um teste de regressão que reproduza a falha original. Se ainda não reproduzir, continuar a investigação e registrar essa limitação antes de alterar o fluxo por hipótese.

Saída desta fase: reprodução identificada, dados protegidos e evidências que indiquem onde o processo morre.

## 2. Localizar e corrigir o estouro de pilha

1. Gerar uma build diagnóstica com símbolos, mantendo as otimizações e os recursos relevantes da distribuição Windows. Comparar também desenvolvimento e release, sem presumir que o resultado de uma vale para a outra.
2. Capturar a pilha da falha com um depurador apropriado ou um dump local limitado ao Luxmc. Caso seja necessário configurar captura, limitar a alteração ao aplicativo e restaurar a configuração após o diagnóstico.
3. Manter dumps e logs com dados de contas em armazenamento privado. Remover tokens, credenciais, informações de sessão e dados pessoais dos relatórios compartilháveis.
4. Acrescentar logs persistentes e descarregados para disco antes das etapas críticas. A investigação precisa continuar útil quando a interface e o processo encerram abruptamente.
5. Investigar primeiro o caminho compartilhado por todos os tipos de instância: comando Tauri, entrada de `launch_game_core`, preparação de perfil, verificações, construção do future de lançamento e criação do processo Java.
6. Verificar futures grandes, dados volumosos alocados na pilha, recursão, desserialização e chamadas encadeadas. Medir e usar a pilha capturada para decidir a correção; esses itens são hipóteses, não causas já confirmadas.
7. Conforme a evidência, dividir a preparação em etapas menores, transferir dados grandes para o heap, limitar recursão ou corrigir o agendamento do trabalho. Isolar operações bloqueantes para preservar a interface e o serviço LAN.
8. Avaliar tamanho de pilha de threads apenas quando a medição demonstrar necessidade. Um aumento isolado não será aceito como solução definitiva para uma recursão ou alocação descontrolada.
9. Reproduzir o caso com os limites de pilha usados pela aplicação Windows empacotada. Acrescentar um teste que falhe com a implementação antiga e passe com a correção.

Saída desta fase: função ou cadeia causadora identificada, correção explicável e regressão protegida por teste.

## 3. Tornar o início e o encerramento confiáveis

1. Organizar a sessão de lançamento no backend com estados claros: preparando, iniciando Java, aguardando o jogo, aberto, falhou, cancelado e encerrado.
2. Associar cada tentativa a uma identidade de sessão e à identidade do processo. Rejeitar eventos antigos e impedir que um PID reutilizado seja confundido com a sessão atual.
3. Tratar os dois segundos atuais como uma verificação de falha precoce, e não como confirmação de abertura do Minecraft.
4. Confirmar a abertura com sinais adequados à versão: processo correto, janela de renderização do Minecraft e progresso consistente. Janelas de erro do Java ou de auxiliares não devem autorizar a ocultação do launcher.
5. Durante a validação, comprovar visualmente o menu e a abertura de um mundo. Um teste que só recebe um heartbeat ou cria um processo simulado continua sendo teste de infraestrutura.
6. Manter o launcher aberto durante a preparação. Ocultá-lo apenas depois da confirmação adequada e conforme a preferência do usuário, mantendo o backend e a sala P2P ativos.
7. Garantir restauração da janela e diagnóstico quando o Java falhar, quando o monitor falhar ou quando o jogo encerrar durante a preparação. Persistir o erro antes de notificar a interface.
8. Se o jogo demorar, apresentar a etapa atual, o tempo decorrido e acesso aos logs. Considerar progresso recente e as características do modpack antes de declarar timeout; não encerrar automaticamente um modpack pesado que está avançando.
9. Permitir cancelar a abertura. O cancelamento deve atingir somente os processos e tarefas pertencentes àquela sessão, preservar arquivos válidos e deixar a instância pronta para nova tentativa.
10. Manter o monitor e o progresso ao trocar de aba. Impedir lançamentos simultâneos e recuperar o estado depois de uma falha abrupta, evitando um indicador de jogo aberto sem processo correspondente.
11. Exibir erros duráveis com a etapa, motivo, código de saída e opção de copiar diagnóstico sanitizado. Evitar notificações breves como único registro da falha.

Saída desta fase: o launcher acompanha o ciclo inteiro sem desaparecer silenciosamente ou informar sucesso antes da abertura real.

## 4. Revisar o caminho comum de Java, versões e loaders

1. Escolher Java a partir do manifesto e das exigências do loader, com validação do executável e da arquitetura. Conferir os runtimes já instalados antes de fazer novos downloads.
2. Revisar argumentos JVM, limites de memória, classpath, bibliotecas nativas e diretório de trabalho. Testar caminhos com espaços e acentos, inclusive o ambiente usado neste PC.
3. Iniciar com argumentos estruturados de processo. Evitar remontar o comando como texto passado por outro shell.
4. Preservar a sanitização de variáveis que injetam argumentos JVM. Falhas de autenticação devem aparecer como erros de autenticação, sem esconder uma falha nativa do launcher.
5. Validar herança de manifestos e pontos de entrada de Vanilla, Fabric, Quilt, Forge e NeoForge. Restringir combinações que o provedor não oferece e explicar o motivo na interface.
6. Exercitar preparação nova, arquivos já instalados, downloads interrompidos, manifesto local e cache parcial. Reparar arquivos identificados como inválidos em vez de baixar tudo indiscriminadamente.
7. Tratar versões legadas e argumentos específicos de sistema operacional. Testar os agentes de skin e LAN junto com a inicialização real, além dos testes isolados existentes.
8. Auditar otimizações, hooks e agentes na preparação do jogo. Para investigar conflitos, usar uma cópia da instância com alterações reversíveis e registrar exatamente o que foi desativado.
9. Manter o Luxmc Shield e a verificação de integridade. Não reduzir proteções para fazer um teste passar.
10. Um pacote incompatível deve produzir diagnóstico de Java, loader, dependência ou mod, conforme a evidência, mantendo o launcher utilizável.

Saída desta fase: caminho compartilhado consistente e problemas específicos de pacotes identificáveis, sem perda de dados.

## 5. Matriz de testes reais

Conferir versões de Java e loaders nos manifestos disponíveis no momento da implementação. Testar somente combinações oferecidas pelos respectivos projetos.

| Grupo | Casos representativos | Resultado necessário |
| --- | --- | --- |
| Caso original | Vanilla 26.3, instalação nova e segunda abertura | Menu real, mundo carregado, launcher vivo e retorno após sair |
| Vanilla moderno | 1.21.1 e 1.20.1 | Java correspondente, renderização, abertura e encerramento |
| Vanilla legado | 1.16.5, 1.12.2 e 1.8.9 | Natives e argumentos legados corretos |
| Fabric | Versão suportada, sem mods extras e com um pacote real | Ponto de entrada e agentes compatíveis |
| Quilt | Uma combinação efetivamente suportada | Inicialização real e diagnóstico útil em falha |
| Forge | 1.20.1 e uma combinação legada suportada | Preparação do loader e abertura do jogo |
| NeoForge | Uma combinação moderna suportada | Instalação do loader e abertura do jogo |
| Modpack leve | Fabulously Optimized compatível | Primeira abertura e abertura com cache |
| Modpack pesado e legado | RLCraft e SkyFactory 4 compatíveis | Progresso durante a espera e chegada ao menu |
| Caso de terror | One Terror Block 1.20.1 com o loader do pacote | Abertura real do pack e mundo carregado |
| Personalização | Pack Luxmc, skin e capa nas combinações suportadas | Recursos mantidos e agentes sem impedir o início |
| Sala LAN | Vanilla e One Terror Block, com dois PCs quando disponíveis | Descoberta do mundo, entrada do convidado e sala ativa com launcher oculto |

Também testar falhas e condições de uso:

- Java ausente, arquitetura incorreta e Java incompatível.
- Falha do Java antes e depois dos dois segundos atuais.
- Processo vivo sem janela, janela de erro e janela que aparece e fecha.
- Mod incompatível, biblioteca ausente, download interrompido e bloqueio de execução.
- Memória insuficiente, argumentos inválidos e pacote com preparação demorada.
- Cancelamento, segundo clique em Jogar e tentativa de duas instâncias ao mesmo tempo.
- Troca de aba durante preparação e durante a abertura do jogo.
- Preferências de manter aberto e ocultar/reabrir, retorno após sair e permanência da sala P2P.
- Instalação nova, atualização da versão anterior e reabertura depois de crash.

Executar primeiro Vanilla 26.3 e a regressão do crash. Só ampliar a matriz depois que esse caso funcionar realmente. Registrar por caso o artefato, versão, Java, loader, evidência de abertura e resultado. A validação em dois PCs depende da disponibilidade dos participantes e deve permanecer explicitamente pendente se não ocorrer.

## 6. Fechar a lacuna da distribuição

1. Executar `pnpm check` após alterações Svelte/TypeScript e `cargo check` após alterações Rust. Rodar testes unitários e de integração relevantes, o conjunto de navegador e a build de produção.
2. Acrescentar testes nativos do comando de lançamento na configuração Windows de distribuição. Exercitar limites de pilha, falha abrupta e propagação dos erros entre backend e interface.
3. Usar mocks para estados e falhas controladas, e processos reais para o ciclo de vida. Mocks não substituem os testes de abertura do Minecraft.
4. Gerar um instalador candidato e verificar o `luxmc.exe` contido nele e o executável realmente instalado. Relacionar os hashes ao commit e à configuração de compilação.
5. Testar o instalador candidato no PC que apresentou o crash e em outro ambiente Windows disponível, usando instâncias de teste e protegendo os dados existentes.
6. Exigir a abertura real de Vanilla 26.3 pelo executável empacotado, além dos casos prioritários da matriz. Um runner sem suporte gráfico adequado deve registrar essa limitação; uma falha de renderização na VM não pode ser convertida em prova de sucesso.
7. Repetir instalação, atualização, reinstalação e desinstalação, conferindo atalhos, preservação de dados e ausência de dependência do Lux Repair.
8. Fazer a varredura do pacote e também dos executáveis efetivamente extraídos ou instalados. Documentar separadamente detecção de malware, integridade dos arquivos e assinatura Authenticode.
9. Preparar notas da correção com causa, comportamento corrigido, testes reais e limitações. Preparar a próxima publicação com um número novo, preservando a release 3.9.2 existente.
10. Na publicação solicitada, verificar manifesto assinado, todos os instaladores, versão no site, API de atualização e bytes do download Windows. O site e o atualizador devem oferecer o mesmo artefato aprovado nos testes reais.

Saída desta fase: instalador corretivo validado em uso real e evidências suficientes para preparar sua distribuição.

## Critérios de conclusão

- A falha nativa relatada foi reproduzida ou localizada por evidência e a causa responsável foi corrigida.
- Vanilla 26.3 abre pelo launcher Windows empacotado, chega ao menu e carrega um mundo.
- O launcher acompanha a preparação e permanece vivo; erros recuperáveis são apresentados com diagnóstico persistente.
- Os casos suportados prioritários da matriz abrem realmente, com resultados registrados por Java e loader.
- Cancelar, trocar de aba e encerrar o jogo deixam a sessão e a interface em estado consistente.
- Mundos, contas, skins, configurações e recursos existentes permanecem preservados.
- As verificações exigidas pelo projeto e os testes de regressão passam.
- Pacote, executável instalado e resultados dos testes podem ser relacionados ao mesmo candidato de release.
- Testes que não foram executados, especialmente a partida LAN em dois PCs, aparecem como pendentes.

Modpacks de terceiros podem conter incompatibilidades ou exigir hardware específico. O compromisso é validar as combinações suportadas e os casos representativos, corrigir a falha compartilhada e apresentar erros úteis. Não declarar garantia universal baseada somente nos testes automatizados.

## Ordem de implementação na retomada

1. Ler este plano e conferir mudanças posteriores.
2. Preservar dados e reproduzir o crash de Vanilla 26.3.
3. Identificar e corrigir o estouro de pilha, com teste de regressão.
4. Corrigir confirmação de abertura, acompanhamento, cancelamento e diagnóstico.
5. Revisar Java e loaders e executar a matriz real, começando pelo caso original.
6. Gerar e instalar um candidato, testar o executável empacotado e preparar a próxima publicação.

Entregáveis: causa documentada, correção de código, regressões automatizadas, relatório dos testes reais, instalador candidato identificado por hash e notas prontas para a próxima release.
