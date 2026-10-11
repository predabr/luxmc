# Verificação do Luxmc 3.8 — Novos Universos

A 3.8 publica as melhorias locais do catálogo, do site, da preparação de instâncias, das aparências e da descoberta LAN. A identidade do aplicativo e o nome do instalador **Lux MC Launcher.exe** são mantidos. Metadados do launcher, reparo, idiomas, site e pacotes usam a versão 3.8.0, revisão 0.

## Verificações locais da implementação

- Frontend: 153 testes em 26 arquivos; verificação Svelte sem erros ou avisos e build de produção concluído.
- Site: 55 testes de contas, catálogo, protocolo, downloads, mídia, interface e escrita dos arquivos gerados. Build e revisão no navegador em desktop, mobile e movimento reduzido concluídos.
- Catálogo: seleção de destaques, abertura de detalhes, filtros reais, busca, rolagem, virtualização, paginação, instalação por teclado, cancelamento e capas de fallback verificados. Temas claro/escuro e larguras de 1600, 1080, 800 e 640 px sem corte nos botões de instalação.
- Navegação: retorno à sala, acesso à biblioteca e contador de jogadores do Discord cobertos por regressões no navegador.
- Regressões completas da interface: os 20 cenários passaram no navegador, incluindo criação Java/Bedrock, biblioteca, skins, armazenamento, idiomas, exclusão, cancelamento, temas, movimento reduzido, catálogo e presença. Os dados simulados da conta salva e o seletor de cartões foram atualizados para a interface 3.8.
- Presença: dois testes Rust do heartbeat nativo passaram no Windows, incluindo continuidade sem temporizadores da interface e prevenção de consulta duplicada após sincronização recente. A regressão no navegador cobriu falha temporária, preservação da lista, reconexão, conta local online, troca de conta, evento de sessão antiga, avatar grande, atividade do jogo e encerramento da sessão.
- Agente Java: compartilhamento de aparências validado com diferentes versões de Authlib e Java 17, 21 e 25. Descoberta de porta testada com publicação bem-sucedida, falha de bind, listeners não relacionados, IPv4/IPv6 e classes reais do Minecraft 1.8.9, 1.20.1, 1.21.1, 26.3 e Forge 47.4.10.
- Conexão local: as tentativas IPv4 e IPv6 começam em paralelo. A regressão verifica a descoberta dentro do limite de 300 ms e a troca de bytes nas duas famílias, evitando a espera pela recusa IPv4 no Windows.
- Rust: compilação local e compilação dos testes concluídas. Os dois testes de presença foram executados normalmente e passaram. Tentativas anteriores da suíte completa foram bloqueadas pela política AppControl e não são registradas como aprovadas; a publicação exige a suíte completa nos runners do GitHub.
- Shield: os caminhos obrigatórios de análise antes de iniciar o jogo e depois de comandos de preparação permanecem ativos. Cache, hashes e cancelamento não removem a verificação de segurança.
- Reparo do Windows: oito testes de recuperação passaram. As pastas temporárias dos testes paralelos usam identificadores exclusivos, evitando colisões quando o relógio do Windows retorna o mesmo instante para duas tarefas.

As capturas do novo catálogo usam dados de demonstração e estão identificadas como prévias. Nenhuma conta, mundo ou instância pessoal foi alterada para os testes de interface.

## Limites da validação

Os participantes confirmaram uma partida P2P vanilla 26.3. O relato de falha no One Terror Block 1.20.1 motivou os ajustes da descoberta de porta; a nova implementação ainda precisa ser repetida nesse modpack nos dois PCs. Testes locais não demonstram compatibilidade universal de modpacks ou redes.

As mudanças reduzem trabalho duplicado, escrita sem buffer e picos de memória. Não foi atribuída uma proporção fixa de ganho de velocidade. O tempo total depende do disco, da rede, do provedor e do modpack.

## Publicação

O fluxo de release verifica a fonte, gera os instaladores Windows, Linux e macOS, confere os arquivos e assina `latest.json` e `SHA256SUMS` com a chave Ed25519 oficial antes de tornar a release pública. O site só recebe o instalador depois de verificar versão, assinatura e hash. A assinatura dos manifestos não equivale a certificado Authenticode nem a notarização do macOS.
