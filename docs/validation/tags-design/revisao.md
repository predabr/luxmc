# Tags do launcher · 11/10/2026

As tags usam um componente compartilhado com ícones, cores dos tokens do tema, contornos suaves, entrada animada e respeito às preferências de movimento. Dono, Pro, Plus, Testador e Amigo têm identidades distintas. Tags personalizadas mantêm seu texto e um símbolo neutro.

O visual foi aplicado aos perfis públicos, perfil da conta, amigos, painel lateral, participantes da sala e editor do dono. Os cartões de amigos reservam espaço para as tags e separam as ações, evitando que os nomes sejam cortados.

## Verificação

- `pnpm check`: 0 erros e 0 avisos.
- `tests/room-owner-cancellation.browser.cjs`: aprovado, incluindo edição pelo dono, exibição pública e ocultação dos controles para outras contas.
- Prévia com dados simulados em temas claro e escuro, a 1600 e 960 pixels: sem erros de JavaScript, sem transbordamento e sem corte nos nomes das tags.
- `git diff --check`: aprovado.
- `tests/friends-redesign.browser.cjs`: interrompido pela exigência antiga de altura do cabeçalho abaixo de 110 pixels. O cabeçalho não foi alterado nesta tarefa.

As imagens nesta pasta usam dados de demonstração. A alteração está no código local; não foi gerado instalador nem publicada uma versão nesta tarefa.
