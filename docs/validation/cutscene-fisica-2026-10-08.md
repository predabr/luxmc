# Abertura da logo com física — 2026-10-08

A abertura foi refeita com fundo simples e a logo original. A logo aparece, quebra em 48 fragmentos triangulares irregulares, cai com gravidade, colisões, atrito e rotação e retorna por molas amortecidas. A simulação usa Matter.js, com passos fixos de 120 Hz; a recomposição desativa colisões entre fragmentos para permitir o encaixe exato. Não é uma simulação física de fratura do material; os fragmentos são predefinidos e o movimento é simulado com corpos rígidos 2D.

A duração normal é de cerca de 5,7 segundos, com saída suave. Esc, Enter, espaço e o botão Pular encerram a abertura. Preferência de movimento reduzido ou animações desativadas mostram somente a logo por um momento. A simulação pausa enquanto a janela está oculta e libera RAF, observador, eventos e corpos ao terminar.

O temporizador da inicialização não encerra mais a cutscene. Terminar a cutscene também não declara que os dados do aplicativo terminaram de carregar.

## Verificação

Teste de browser com inicialização artificialmente atrasada: temas claro e escuro aprovados, incluindo todas as fases e colisões com o chão. Ao concluir, o erro máximo de posição foi inferior a 0,01 unidade e o de rotação foi de 0,0008 radiano. Movimento reduzido e pular com Esc aprovados. Capturas de cada fase foram inspecionadas visualmente em docs/validation/cutscene-physics.

Versão 3.2.0 mantida. Inclui as correções anteriores de GIF e sessão Microsoft. Nenhuma publicação ou substituição automática da instalação do usuário.

Documentação do motor: https://brm.io/matter-js/docs/classes/Engine.html

pnpm check: zero erros e avisos. A interface compilada de produção também passou no teste completo da animação, nos dois temas, com movimento reduzido e pular. Não houve erros de JavaScript. O motor é carregado somente para a abertura animada.

## Instalador local

Build Windows concluído com sucesso. Cabeçalho e versão conferidos.

- Arquivo: release-windows/Luxmc-3.2.0-cutscene-fisica.exe.
- Versão: 3.2.0.
- Tamanho: 20852535 bytes.
- SHA-256: 69e8f874fe42a1b9737b6a19326f594bed075a20260ccab8099a54bdcdbf5ab2.
- Aplicativo instalado preservado (hash antes/depois igual): True.
- Sem publicação no site/GitHub ou instalação automática.
