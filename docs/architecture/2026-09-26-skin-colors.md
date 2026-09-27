# Correção da skin preta

Foi reproduzido o defeito com o PNG do usuário: havia pixels opacos do modelo, mas o teste não encontrava detalhes coloridos ao renderizar de frente. Esse novo teste falhou antes da correção.

A prévia mantém a geometria, UVs e transparência, mas usa a própria textura como emissão do material, com a contribuição difusa zerada. Assim, as cores não dependem da iluminação do material padrão que produzia a silhueta preta. O arquivo PNG não é modificado.

Após a alteração, passaram 44 verificações WebKitGTK, incluindo oito ciclos que verificam pixels coloridos da textura, seleção, navegação, perda/restauração de contexto e arraste. O teste utiliza o PNG local do usuário e comandos de conta simulados.

O teste do adaptador Java passou em 14 combinações de modelo/capa nas versões Authlib 1.5.25, 3.11.49, 4.0.43, 6.0.54, 7.0.63, 9.0.75 e 10.0.77, com Java 8, 17, 21 e 25. Ele requisita a textura pelo endereço entregue à Authlib e compara todos os bytes com o PNG original do usuário. Isso valida o adaptador, não uma partida completa nem a aparência vista por outros jogadores em qualquer servidor.

Para usar a seleção no próximo lançamento é necessário clicar em Aplicar skin e capa. A sincronização oficial Microsoft continua sendo uma ação separada. Nenhuma aparência de conta foi alterada pelos testes.

Logs: `/home/pedro/.cache/luxmc-validation/2026-09-26-skin-colors/`.

Svelte/TypeScript: zero erros e avisos. Executável de produção compilado em 7 min 29 s.
