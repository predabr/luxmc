# Modpacks, histórico e progresso — 12/10/2026

Correções locais sobre 3.9.1, sem publicação de uma nova versão.

As capas preenchem o cartão. A recuperação verifica o ID e o provedor, consulta a galeria e procura uma imagem ampla na descrição do próprio autor quando a galeria está vazia. Pequenas imagens de descrição não são usadas como capas. No Modrinth, os detalhes usam os arquivos originais disponibilizados em `raw_icon_url` e `raw_url`, preservando as miniaturas leves da busca. As consultas têm cache, deduplicação e limite de três chamadas simultâneas.

Foram consultados os dados reais de RLCraft, SkyFactory 5, Zombie Invade 100 Days, BattleArmory TACZ e FPS Modpack. A capa do RLCraft veio da descrição do autor. BattleArmory e FPS Modpack usam os arquivos originais com 1024 e 1080 pixels de largura. Zombie Invade e SkyFactory usam a arte original disponível, com 256 e 400 pixels; a resolução é limitada pelos arquivos publicados pelos autores. Os resultados e a captura estão em `modpack-followup/live-cover-results.json` e `modpack-followup/covers-providers.png`.

O changelog abre o histórico de várias versões, permite filtrar uma versão e carregar as anteriores. O CurseForge usa a [paginação documentada](https://docs.curseforge.com/rest-api/#get-mod-files), mantendo acesso aos arquivos posteriores à primeira página. O conteúdo é sanitizado e carregado em grupos, com cache e até três pedidos simultâneos. Uma falha individual permite nova tentativa sem apagar o restante do histórico.

A atualização de modpack passou a usar estado compartilhado por instância. O painel de progresso acompanha a navegação, evita atualizações concorrentes e deixa as instalações novas na fila até terminar. A conclusão limpa o aviso antigo e recarrega os dados da instância aberta. Respostas de verificações iniciadas antes da atualização não restauram um aviso ultrapassado. A substituição por uma versão escolhida manualmente usa o mesmo fluxo.

O fundo radial vermelho e a elipse sob o personagem foram removidos. As funções de skin, capa, importação, NameMC, câmera e aplicação permanecem no mesmo fluxo.

## Verificação

- `pnpm check`: zero erros e zero avisos.
- `pnpm test:run`: 177 testes passaram em 30 arquivos.
- `cargo check --release --features tauri/custom-protocol`: concluído; cinco avisos anteriores de código sem uso. Tentativas de check com outros perfis foram bloqueadas pelo Controle de Aplicativo; nenhuma política de segurança foi alterada.
- Navegador: histórico com 55 versões, incluindo a segunda página do CurseForge; conteúdo sanitizado; troca para Amigos e retorno durante uma atualização; remoção do aviso após concluir; fundo neutro no personalizador.
- Regressão: navegação do catálogo, importação de skins, fidelidade da textura, capa e recuperação de erro, aplicação do PNG original, recuperação de arte e layouts do personalizador.
- Imagens reais: identificação e resolução das artes dos cinco projetos citados, sem alterar contas ou instâncias reais.

Os testes de navegador usam dados simulados para as operações de instalação. A verificação das capas usa metadados públicos e imagens reais dos provedores, mantendo as operações de conta e instância simuladas. Nenhum Minecraft foi iniciado durante esses testes.

## Instalação local concluída

A compilação Windows final e o pacote NSIS foram concluídos. O verificador correspondente foi compilado e testado no [GitHub Actions](https://github.com/predabr/luxmc/actions/runs/38264425892). Foram conferidos o commit, o digest do artefato, o hash do verificador e sua correspondência com o programa.

O Defender não encontrou ameaças no instalador, no launcher e no verificador. Esse resultado não representa assinatura Authenticode nem aprovação dos outros fornecedores.

Os arquivos da instalação em `AppData/Local/Luxmc` foram atualizados após fechar o launcher e fazer backup. Os hashes instalados correspondem ao pacote verificado. Banco de dados, contas e configurações mantiveram seus hashes. A versão continua 3.9.1; o instalador local se chama **Lux MC Launcher — Modpacks em Dia** e está em Downloads.

O launcher foi deixado fechado. A abertura da nova compilação no Windows não foi testada. Nenhuma política de segurança foi alterada. O instalador público 3.9.1, os arquivos do site e a versão pública não foram alterados. O relatório da instalação está em `modpack-followup/installed-local.json`.
