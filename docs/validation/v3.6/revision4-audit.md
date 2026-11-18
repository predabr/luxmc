# Luxmc 3.6 — Jogar Juntos, revisão 4

Revisão de 10 de outubro de 2026. A entrega desta revisão é local. O serviço da comunidade foi atualizado para suportar as funções solicitadas; nenhum novo instalador foi publicado no GitHub ou no site nesta revisão.

## Comunidade, tags e exclusão

Somente a identidade autenticada do dono pode consultar a administração, alterar tags ou remover perfis. Nome, avatar, tags e o campo de cargo enviados pelo cliente não concedem essa permissão. O dono pode adicionar tags ao próprio perfil. As tags aparecem publicamente nos amigos, na busca, na barra lateral e nos participantes da sala, inclusive quando chegam pela sincronização em tempo real. Elas não concedem privilégios administrativos.

A exclusão exige digitar o nome exato do perfil. O servidor remove os dados públicos, amizades, bloqueios, tags, salas e tickets em uma transação. Uma identidade mínima marcada como removida preserva o vínculo de login e impede que o próximo heartbeat recrie o perfil. O perfil removido deixa de acessar o serviço social. Contas de jogo, instâncias e mundos locais permanecem preservados. A exclusão do próprio dono é recusada.

A migração 0012 foi aplicada no banco de produção. O endpoint real aceitou a identidade do dono, retornou os 12 perfis existentes na verificação e recusou a exclusão anônima com HTTP 401. Nenhum perfil real foi excluído e nenhuma tag real foi modificada durante os testes. O serviço de eventos também verifica perfis removidos e encerra suas conexões.

## Atualizações e preservação de dados

As referências oficiais de projeto e versão foram restauradas nos quatro modpacks instalados. Novas instalações passam a registrar essas referências antes de apresentar sucesso, para que a busca de atualização funcione. A versão de cada modpack segue o manifesto de seu autor, evitando atualizar mods isoladamente para uma versão incompatível do Minecraft.

| Instância | Verificação do provedor |
| --- | --- |
| Vanilla | Minecraft 26.3 já instalado; versão 1.8.9 preservada como instância distinta |
| Fabulously Optimized | 15.0.0-alpha.5, Minecraft 26.3; já atualizado |
| Cobblemon Official Modpack Fabric | 1.8.1, Minecraft 1.21.1; já atualizado |
| Zeefski Jujutsu Kaisen Craft | Atualizado de 3.2 para 3.2.1, Minecraft 1.20.1 |
| All the Mods 10 | 8.2, Minecraft 1.21.1; já atualizado |

A atualização real do ZJJK levou 73 segundos neste computador. O arquivo oficial foi conferido pelo provedor; a pasta anterior ficou preservada como backup, incluindo o mundo existente. O marcador de propriedade da nova pasta permite iniciar uma instância atualizada sem tentar migrá-la de volta para o backup. Pastas externas ou com marcador de outro perfil continuam recusadas.

Evidências: [versões consultadas](revision4/provider-versions.json), [instâncias atualizadas](revision4/installed-packs.json). O catálogo Mojang foi consultado diretamente; a versão estável mais recente encontrada foi 26.3 e o snapshot foi 26.4-snapshot-3. A referência oficial do ZJJK utilizado é o [arquivo 8509973 no CurseForge](https://www.curseforge.com/minecraft/modpacks/zjjk/files/8509973).

## Pacote de recursos e desempenho

Luxmc Essentials acrescenta identidade, ícone e mensagens de menu, com metadados adequados à versão do Minecraft. É preparado na criação, importação e inicialização das instâncias Java compatíveis a partir de 1.6. Foi instalado nas seis instâncias Java deste computador, preservando as outras opções e os mundos. A ativação inicial usa prioridade baixa; se o usuário desativar o pacote depois, a escolha é respeitada. Uma pasta com o mesmo nome criada pelo usuário não é sobrescrita.

A variante Bedrock usa um manifesto próprio, UUIDs distintos e fica disponível na pasta de conteúdo compartilhado. Não altera a seleção global de recursos nem a lista de pacotes dos mundos. O manifesto declara compatibilidade a partir de 1.16; a execução real desta variante dentro do Bedrock não foi validada nesta revisão.

O catálogo Bedrock foi consultado novamente pela API nativa: 229 entradas, com versão estável 1.26.52.3 e Preview 1.26.60.30. Nenhuma instância Bedrock estava instalada no PC durante essa consulta; nenhum pacote Bedrock foi instalado ou removido nesta revisão. Evidência: [catálogo](revision4/bedrock-catalog.json).

O cálculo de hashes do cache passa para tarefas de trabalho, evitando bloquear as tarefas de rede. O progresso de importação é agrupado a cada quatro arquivos e também enviado na conclusão. A leitura dos metadados dos JARs usa buffer para reduzir pequenas operações de disco no Windows. As transições incluem entrada dos cartões, seleção de abas, abertura de detalhes e resposta dos botões. Os modos de movimento reduzido e economia continuam respeitados.

## Verificação

| Área | Resultado |
| --- | --- |
| Frontend | `pnpm check`: zero erros e avisos; 150 testes unitários aprovados |
| Backend | `cargo check` aprovado; 205 testes aprovados, seis integrações optativas ignoradas na execução padrão e executadas separadamente quando aplicável |
| Site e contas | 50 testes aprovados, incluindo 15 testes sociais |
| Interface | 17 suítes aprovadas; exclusão com confirmação, tags públicas, controles ocultos para membros e cancelamento tardio conferidos |
| Interface compilada | Navegação da sala, tags públicas, confirmação de exclusão, restrição de membros e fundos dos painéis aprovados; [resultado](revision4/production-ui/browser.json) |
| Manifestos | 12 testes de versão, arquivos e assinatura aprovados |
| Rede real | Relay público, convite publicado e listener multicast padrão do Java 8 com transferência TCP aprovados |
| Dados reais | Referências dos quatro modpacks conferidas, ZJJK atualizado com backup e Essentials presente nas seis instâncias |
| Segurança | Executável final analisou os 730 mods dos quatro pacotes instalados: nenhum indicador conhecido encontrado; nenhum mod executado ou removido |

Os testes de interface usam respostas IPC controladas. Os testes de rede transferem dados de um serviço de teste e não modificam mundos. O usuário informou que ainda não fez o teste entre os dois PCs. Portanto, uma partida Minecraft entre computadores em redes diferentes ainda precisa ser validada; não foi declarada uma garantia universal de P2P.

O Shield continua usando indicadores estáticos documentados, leitura limitada, hashes e cache por conteúdo. Ele complementa o antivírus e não garante detectar toda ameaça. Os testes usam dados inertes, sem executar malware, desativar o antivírus ou restaurar arquivos em quarentena. Permanecem cinco avisos de código não utilizado no Rust e o aviso anteriormente documentado em `braces`, da cadeia de compilação. Esta revisão não declara ausência de avisos ou vulnerabilidades.

## Entrega

O instalador **Lux MC Launcher.exe**, versão 3.6.0, revisão 4, foi compilado e instalado neste PC. A cópia de entrega está também em `C:/Users/preda/Downloads/Lux MC Launcher.exe`. O arquivo tem 21,284,646 bytes e o mesmo hash do bundle NSIS gerado pelo Tauri.

SHA-256: `d4f441bd06f48769edd251e3efb6a107179f00e03660f04957136a4b40bb976b`.

Os manifestos locais `SHA256SUMS` e `local-release.json` foram assinados com a chave Ed25519 existente e conferidos com a chave pública oficial. Essas assinaturas verificam integridade; não constituem certificado Authenticode do Windows. Nenhum manifesto público com URLs não publicadas foi gerado.

A conferência da instalação encontrou revisão 4 no registro, executável idêntico ao compilado após a normalização do marcador NSIS, seis instâncias e uma conta preservadas. Os arquivos `level.dat` dos cinco mundos conservam seus hashes. O projeto em Documentos permanece preservado. O verificador de reparo retornou `healthy:true`; o launcher foi aberto e a janela respondeu.

Evidências: [executável final e Shield](revision4/release-native.json), [instalação](revision4/installed.json), [reparo](revision4/repair.json), [abertura](revision4/opened.json), [entrega](revision4/delivery.json). O [roteiro LAN](revision4/teste-lan.md) pode ser usado nos dois PCs com este mesmo instalador.

A revisão foi entregue localmente. Os serviços necessários ao painel do dono estão publicados; o instalador público anterior continua no GitHub e no site.
