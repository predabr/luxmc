# P2P: sala do convidado e Minecraft — 04/10/2026

O convidado passa a abrir a tela da sala, com endereço local, participantes e capacidade compartilhados pelo túnel. Consultas de estado e entrada têm tempo máximo e mensagem de recuperação. A navegação de retorno preserva a sala.

O túnel permite consultas de status e conexão de jogo simultâneas, mantendo limites de streams e de participantes. O estado da sala é consultado em canal autenticado separado. O endereço local pode ser copiado ou adicionado a servers.dat da instância selecionada; os servidores existentes e metadados são preservados, com backup antes da escrita. NBT inválido não é sobrescrito.

Validação: pnpm check sem erros ou avisos; pnpm build concluído; cargo check concluído com três avisos existentes. Teste de navegador aprovado: entrada, participantes, cópia, servidor da instância, navegação, saída, tempo máximo de carregamento e recuperação. Teste de relay público aprovado. Sete testes nativos aprovados; o teste de relay ignorado na execução padrão foi executado separadamente e aprovado. Testes nativos cobrem transporte simultâneo, controle da sala e preservação de servidores.

O teste real entre os dois jogadores ainda precisa ser repetido com o novo executável em ambos os computadores. O hospedeiro abre o mundo para LAN e gera um novo convite. O convidado entra no servidor salvo ou usa o endereço exibido em Conexão direta. Minecraft e mods precisam ser compatíveis entre as duas instâncias.

Executável entregue: C:/Users/preda/Downloads/Luxmc-2.0.2-p2p-fix.exe. SHA-256 registrado em p2p-guest-delivery.json e conferido contra a compilação. Subsistema PE gráfico do Windows (2). Launcher iniciado e janela Luxmc respondendo, PID 14780.
