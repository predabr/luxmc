# Resgate 3.9.2: fluidez, janela SDL e integração LAN

O painel flutuante de abertura foi removido. O cancelamento continua disponível na barra lateral durante a preparação, inclusive após navegar para outra página. Erros reais continuam com diagnóstico e acesso aos logs.

O monitor agora reconhece `SDL_app`, além de `GLFW30` e `LWJGL`, e mantém a confirmação em duas observações consecutivas da janela do processo correto. A implementação oficial do SDL registra esse nome em [SDL_windowsevents.c](https://raw.githubusercontent.com/libsdl-org/SDL/main/src/video/windows/SDL_windowsevents.c). O jogo 26.3 abriu no teste do usuário, mas o monitor anterior não reconhecia sua janela SDL e continuava mostrando a abertura.

Logs são persistidos em lotes de até 128 linhas, com uma transação por lote e intervalo máximo de 250 ms para lotes parciais. Timestamps, ordem e filtragem de credenciais são preservados. Um teste com 300 mensagens verificou três lotes. O fechamento da sessão descarrega o lote pendente antes de encerrar o registro.

O pré-carregamento de páginas espera a inicialização e é suspenso durante abertura do jogo, jogo em execução, rolagem ou janela oculta. Wallpapers animados, efeitos de fundo e blur também deixam de disputar recursos durante a abertura. As preferências visuais são restauradas depois.

Os agentes LAN e P2P passam a procurar os marcadores necessários no pool de constantes das classes Java, sem transformar cada classe inteira em uma string. O teste ASM verifica três mensagens de publicação LAN, os ganchos de preparação e abertura e a integração do login P2P. Classes sem os marcadores continuam intactas.

Uma tabela de saltos por marcador evita comparar cada posição de cada string. Em 3.000 classes reais da 26.3, repetidas cinco vezes, a alocação medida caiu de 140.611.680 bytes para 160 bytes. A busca levou 0,107 s contra 0,043 s da conversão anterior; é uma redução de alocação com pequeno custo adicional de CPU, não uma medição do tempo completo de abertura do jogo. Cem casos adicionais com prefixos repetidos verificam que os saltos não omitem marcadores.

Quando os contadores ingleses do Windows estão ausentes, o launcher seleciona a API de fallback do OSHI pelo argumento `-Doshi.os.windows.hkeyperfdata=false`, respeitando qualquer opção explícita do usuário. Nenhuma configuração do Windows é alterada. O teste isolado com OSHI da 26.3 confirmou que os dados do processo continuam acessíveis: mediana total de 4,074 s no padrão e 4,000 s com fallback, três amostras por configuração. Essa diferença pequena não demonstra aceleração do início completo do Minecraft.

## Verificação local

- `pnpm check`: zero erros e avisos.
- `pnpm test:run`: 191 testes passaram em 33 arquivos.
- `cargo check --release --locked --lib`: passou; cinco avisos preexistentes de código não usado.
- Testes nativos em modo release no Windows: 244 passaram, zero falhas, sete testes de integração externa ignorados. Incluem transferência criptografada P2P, anúncio LAN na interface física, entrada de convidados e troca de texturas na sala.
- Testes de navegador para sessão, confirmação de abertura e sala: passaram, incluindo navegação durante abertura, cancelamento restrito à sessão, eventos antigos ignorados e erro real visível.
- Os 26 arquivos de regressão de navegador passaram localmente. Duas tentativas atingiram timeout durante compilação concorrente do frontend; os testes afetados e o restante da sequência passaram na repetição, sem alterações no código ou nas asserções. A CI em máquinas limpas continua em execução.
- `scripts/test-client-hooks.mjs`: passou.
- `scripts/test-client-glfw.mjs`: passou, sem inicialização gráfica antecipada pelo agente.
- `scripts/test-client-agent.py --available`: passou com Authlib 1.5.21, 4.0.43, 6.0.54, 7.0.63 e 10.0.77, usando Java 17, 21 e 25. Foram verificadas skin, capa, modelos slim/default, aparências compartilhadas e salas, preservação da identidade Microsoft e rejeição de sala expirada.

Os testes de aparência abrangem o cenário informado de uma conta Microsoft e outra Luxmc. Eles não substituem uma sessão real entre dois computadores nem garantem compatibilidade com cada combinação de mods. O próximo teste de vocês deve usar a mesma versão e modpack, entrar na mesma sala Luxmc, abrir o mundo para LAN e conferir conexão, skin e capa em ambos os lados.

O candidato mantém a versão 3.9.2. Não foi promovido a uma nova versão pública nesta etapa.

O build nativo NSIS terminou e o candidato foi instalado com `/S /UPDATE`, retornando código zero. O executável instalado corresponde ao arquivo incluído no instalador; a diferença de três bytes em relação à entrada do empacotador é apenas o marcador `UNK` → `NSS` do Tauri. O pacote não contém `luxmc-repair.exe`. A verificação após a instalação confirmou a integridade do banco, a manutenção das quantidades de contas/perfis e dos arquivos de configuração da raiz. O launcher atualizado foi aberto e sua janela ficou visível. Os hashes e limites estão em [candidate-fluidez.json](candidate-fluidez.json).
