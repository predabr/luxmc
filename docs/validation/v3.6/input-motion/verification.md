# Correção local de cliques e transições

Solicitação vigente: corrigir a perda de interação depois de excluir uma instância e melhorar as animações ao trocar de tela/aba. Não publicar no GitHub nem no site.

O usuário relatou que a biblioteca continuava visível após excluir, mas nenhum controle recebia cliques. O teste de navegador confirmou que o menu de ações escrevia `pointer-events: none` no corpo. O cartão também iniciava pré-carga de navegação ao pressionar seus controles. Esses caminhos foram eliminados: menus não bloqueiam mais o corpo, usam posicionamento fixo e fecham antes de executar a ação. A armadilha de foco ignora elementos desconectados e devolve o foco à navegação quando o cartão foi removido.

As transições aguardam a atualização do DOM, combinam fade da página e entrada escalonada das superfícies visíveis e são canceladas ao terminar ou navegar. O contêiner da rota não recebe transformação. As abas de instâncias, detalhes de mods, amigos e configurações recebem a mesma animação. O modo de desempenho usa apenas fade curto. Preferências de animação e movimento reduzido continuam respeitadas. O personalizador não teve seu design alterado.

Uma primeira versão usava escala nas superfícies; o teste da lista com mil mods mostrou alteração na largura medida dos controles durante a animação. A escala foi retirada, e os testes de dimensões passaram.

Verificações concluídas:

- `pnpm check`: zero erros e zero avisos.
- Vitest: 147 testes em 26 arquivos.
- Frontend de produção: compilado.
- Testes de exclusão: cancelar repetidamente, pesquisar, simular arquivo bloqueado, tentar novamente, excluir, esperar 20 segundos e navegar de verdade pelo menu lateral; modos normal e desempenho passaram.
- Spinners girando durante operações pendentes, incluindo modo de desempenho e preferência de movimento reduzido explicitamente sobreposta pelo usuário.
- Lista de mil mods em 1440 e 960 pixels: navegação, filtros, medidas, carregamento limitado e abas isoladas passaram.
- Catálogo de 36 packs: pesquisa, paginação, rolagem e instalação por teclado passaram.
- Revisão de 30 telas, 13 rotas animadas e trocas rápidas por cliques: sem erros de página, sem transformação persistente na raiz e sem animações retidas.

A atualização local foi instalada com `/S /UPDATE`. Os cinco arquivos protegidos de dados/autenticação/configurações mantiveram seus hashes durante a instalação, assim como as pastas de instâncias. O binário instalado corresponde à compilação local e o verificador de recuperação confirmou integridade sem reparo ou rollback. Instalador: `Luxmc_3.6.0_local-cliques-animacoes_x64-setup.exe`, 21.127.407 bytes, SHA-256 `228b1258d7728c91b579b9d6a197196015136ffb1e761fdaca9a624917cd4fa4`.

No aplicativo instalado, uma instância descartável foi criada, a confirmação foi aberta/cancelada/reaberta e a exclusão foi executada pelo botão. A pasta e o registro foram removidos de verdade. As três instâncias originais permaneceram intactas. Depois de 30 segundos na biblioteca, os cliques abriram Ajustes, trocaram para a aba Java e voltaram a Instâncias sem reiniciar o processo. A biblioteca terminou sem diálogo remanescente. Resultados em `installed.json`, `native-deletion.json` e `native-ui.json`.

A automação por índice de acessibilidade do WebView2 selecionava a borda da janela, em vez do centro do link. As verificações finais usaram coordenadas da captura completa mais recente e reobservação do resultado, eliminando esse problema de medição. Isso limita o valor diagnóstico da tentativa de clique na versão anterior.

O launcher corrigido ficou aberto, com o tema e fundo selecionados preservados e animações ativadas. Esta correção não foi enviada ao GitHub nem ao site.
