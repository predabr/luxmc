# Abertura pulada e fundo branco — 2026-10-08

No PC do usuário, MinAnimate do Windows está em 0. A cutscene anterior detectava movimento reduzido e encerrava a abertura após 600 ms (a versão antiga usava um intervalo ainda menor). Também foi identificado que o processo em execução é AppData/Local/Luxmc/luxmc.exe, versão 3.0.2.

A abertura solicitada agora executa a simulação completa independentemente da preferência geral de animações do sistema. Não altera configurações do Windows. Esc e o botão Pular continuam disponíveis. As demais animações do launcher mantêm suas preferências existentes.

O documento inicial define um fundo escuro com a cor de sistema Canvas e a janela/webview Tauri usa a cor RGB do token de fundo inicial. A logo é pré-carregada. Isso cobre o período anterior à hidratação da interface.

Verificação: pnpm check e cargo check aprovados. Browser nos temas claro/escuro: quebra, 48 fragmentos, colisões e reconstrução aprovados. Emulação de movimento reduzido: abertura completa aprovada, com opção de pular. Documento inicial com JavaScript desativado: fundo rgb(18, 18, 18), sem tela branca. Sem alterações de versão, publicação ou instalação automática. Versão 3.2.0 mantida.

A interface compilada de produção passou novamente: temas claro/escuro, sequência completa com movimento reduzido ativo, botão de pular e documento inicial escuro sem JavaScript. A janela nativa recebe explicitamente RGB 10/11/15, alinhado ao token inicial de fundo.

## Instalador local

Build Windows concluído. Cabeçalho e versão do instalador conferidos.

- Arquivo: release-windows/Luxmc-3.2.0-abertura-corrigida.exe.
- Tamanho: 20853901 bytes.
- SHA-256: 53e3c00c726ae1507d0fa24f401bc1dcfc33f1f7b2c651f2c5bd33ebd0f6a909.
- Aplicativo instalado preservado (hash antes/depois igual): True.
- Sem publicação no site/GitHub ou instalação automática.
