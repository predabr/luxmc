# Resgate 3.9.2: fluidez, janela SDL e integração LAN

O painel flutuante de abertura foi removido. O cancelamento continua disponível na barra lateral durante a preparação, inclusive após navegar para outra página. Erros reais continuam com diagnóstico e acesso aos logs.

O monitor agora reconhece `SDL_app`, além de `GLFW30` e `LWJGL`, e mantém a confirmação em duas observações consecutivas da janela do processo correto. A implementação oficial do SDL registra esse nome em [SDL_windowsevents.c](https://raw.githubusercontent.com/libsdl-org/SDL/main/src/video/windows/SDL_windowsevents.c). O jogo 26.3 abriu no teste do usuário, mas o monitor anterior não reconhecia sua janela SDL e continuava mostrando a abertura.

Logs são persistidos em lotes de até 128 linhas, com uma transação por lote e intervalo máximo de 250 ms para lotes parciais. Timestamps, ordem e filtragem de credenciais são preservados. Um teste com 300 mensagens verificou três lotes. O fechamento da sessão descarrega o lote pendente antes de encerrar o registro.

O pré-carregamento de páginas espera a inicialização e é suspenso durante abertura do jogo, jogo em execução, rolagem ou janela oculta. Wallpapers animados, efeitos de fundo e blur também deixam de disputar recursos durante a abertura. As preferências visuais são restauradas depois.

Os agentes LAN e P2P passam a procurar os marcadores necessários no pool de constantes das classes Java, sem transformar cada classe inteira em uma string. O teste ASM verifica três mensagens de publicação LAN, os ganchos de preparação e abertura e a integração do login P2P. Classes sem os marcadores continuam intactas.

Quando os contadores ingleses do Windows estão ausentes, o launcher seleciona a API de fallback do OSHI pelo argumento `-Doshi.os.windows.hkeyperfdata=false`, respeitando qualquer opção explícita do usuário. Nenhuma configuração do Windows é alterada. O teste isolado com OSHI da 26.3 confirmou que os dados do processo continuam acessíveis: mediana total de 4,074 s no padrão e 4,000 s com fallback, três amostras por configuração. Essa diferença pequena não demonstra aceleração do início completo do Minecraft.

## Verificação local

- `pnpm check`: zero erros e avisos.
- `pnpm test:run`: 191 testes passaram em 33 arquivos.
- `cargo check --release --locked --lib`: passou; cinco avisos preexistentes de código não usado.
- Testes de navegador para sessão, confirmação de abertura e sala: passaram, incluindo navegação durante abertura, cancelamento restrito à sessão, eventos antigos ignorados e erro real visível.
- `scripts/test-client-hooks.mjs`: passou.
- `scripts/test-client-glfw.mjs`: passou, sem inicialização gráfica antecipada pelo agente.
- `scripts/test-client-agent.py --available`: passou com Authlib 1.5.21, 4.0.43, 6.0.54, 7.0.63 e 10.0.77, usando Java 17, 21 e 25. Foram verificadas skin, capa, modelos slim/default, aparências compartilhadas e salas, preservação da identidade Microsoft e rejeição de sala expirada.

Os testes de aparência abrangem o cenário informado de uma conta Microsoft e outra Luxmc. Eles não substituem uma sessão real entre dois computadores nem garantem compatibilidade com cada combinação de mods. O próximo teste de vocês deve usar a mesma versão e modpack, entrar na mesma sala Luxmc, abrir o mundo para LAN e conferir conexão, skin e capa em ambos os lados.

O candidato mantém a versão 3.9.2. Não foi promovido a uma nova versão pública nesta etapa.
