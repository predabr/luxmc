# Conexão, console e exclusão de servidores — revisão local .909

## Alterações

- O início exige resposta do protocolo Minecraft na porta IPv4 do servidor. Uma conexão TCP isolada não é suficiente para mostrar “pronto”. Três verificações sem resposta após o início exibem indisponibilidade, mantendo acesso à parada e ao console.
- A cópia do endereço consulta novamente o servidor e usa `127.0.0.1`, coerente com a interface de rede em que o Java está escutando. Esse endereço serve ao computador anfitrião; não é apresentado como domínio público.
- Comandos usam controle RCON com senha aleatória e endereço de loopback. Quando essa conexão não abre, o launcher usa a entrada padrão com flush. Uma falha após submissão ao RCON não reenvia o comando pela entrada padrão.
- O console permite seleção, cópia de até 2 MiB recentes do log, histórico com setas e atalhos para jogadores e salvamento. Selecionar texto pausa a rolagem. O campo permanece disponível se o processo estiver ativo mas sem resposta de status.
- A parada exibe seu andamento e continua consultando o processo. Foram preservados os limites de encerramento e o isolamento do processo filho.
- A confirmação de salvamento aceita os formatos Vanilla e Forge/NeoForge e a resposta autenticada do comando de salvar, usada pelas versões recentes que não emitem a antiga linha de confirmação.
- A confirmação de exclusão informa arquivos e tamanho e inclui backups pertencentes ao servidor. A remoção continua recusando links, pastas externas e servidor em execução. Outros servidores e dados compartilhados não são removidos.

## Verificação

O usuário autorizou expressamente iniciar/parar um servidor temporário separado de seus mundos, reutilizando sua aceitação da EULA. Os testes reais usam pastas temporárias, Java e artefatos oficiais; verificam resposta Minecraft, `list` por RCON, salvamento, configurações, parada com código zero e exclusão da pasta temporária. Não representam teste de login Microsoft ou conexão pública de outro computador.

Os resultados finais, hashes das 545 fontes, artefatos e instalação ficam em `%LOCALAPPDATA%/Luxmc-build/requests/local-server-20261026/connection-*.json` e logs correspondentes. A liberação depende de `connection-validation.json` com `finalVerification=passed`; enquanto essa verificação estiver pendente, a revisão não deve ser tratada como entregue.

Verificação das fontes concluída: Svelte/TypeScript sem erros ou avisos, 232 testes do frontend, `cargo check` e 355 testes nativos aprovados, com 18 testes optativos ignorados na suíte padrão. O ciclo real passou em Vanilla 1.21.1, NeoForge 21.1.248 para 1.21.1 e Vanilla 26.3. A interface de produção passou em 1440×900, 1366×768 e 960×600 no tema escuro e 1440×900 no claro; cópia real para a área de transferência também conferida.

Uma execução concorrente da suíte excedeu o prazo do teste sintético de parada. Ele passou isoladamente em 5,24 segundos e a suíte final completa passou com uma thread em 109,95 segundos. Os prazos de produção não foram alterados. Os testes reais identificaram diferenças na confirmação de salvamento em Vanilla, NeoForge e 26.3; as verificações finais acima incluem essas correções.

## Entrada pública

O usuário confirmou que possui somente o site atual. Nenhuma VPS, conta, túnel público ou domínio foi contratado/configurado nesta revisão. O gateway próprio do Luxmc continua pendente de infraestrutura e implementação. A opção de experimentar com Oracle Always Free e um subdomínio Duck DNS está documentada em [infraestrutura-publica-gratuita.md](../infraestrutura-publica-gratuita.md), com limites e fontes oficiais.

## Entrega local em 29/10/2026

Revisão `40255548682.909` instalada e reaberta. Backup `backup-20261029-073737`; payload NSIS conferido e preservação verificada de duas contas, quatro perfis, 17 tabelas e 60 arquivos protegidos. O serviço separado respondeu na revisão .909, e a consulta nativa retornou 100 versões do catálogo. Nenhum mundo existente foi iniciado na entrega. Nenhuma publicação no GitHub ou site foi feita nesta revisão.

Instalador local: `C:/Users/preda/Downloads/Luxmc 4.0 - conexao e console.exe`. SHA-256 do instalador: `286f966ef9c2e5f8352f42b579f6924b50804f05f4829cd844fbfdf662d884f8`. SHA-256 do executável instalado: `99a765cac1d6d1a6cc9b062256e81bf3a96402a1a79949b96171d52da147ac73`. Registro completo em `connection-delivery.json` na pasta privada de validação.
