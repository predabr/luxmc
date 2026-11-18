# NameMC, vitrine, salas e Rich Presence — 25/10/2026

## Entrega local

Versão mantida em 4.0.0. Alterações locais, sem commit, push, release, deploy, aplicação de migração remota ou instalação no computador. Preservadas as correções de modpacks/configurações da retomada anterior e as alterações preexistentes do usuário.

## Implementação

- NameMC: estilo injetado apenas em páginas HTTPS do NameMC, ocultando espaços publicitários reconhecidos, inclusive os inseridos posteriormente. Sem observer, timer permanente, remoção de login ou alterações em desafios de verificação. A navegação continua limitada ao NameMC e a seleção de skins conserva a validação existente.
- Windows: a janela inicia em about:blank, registra o filtro WebView2 e então navega ao catálogo. Requisições para domínios publicitários conhecidos recebem uma resposta vazia 204. Comparação exata de domínio/subdomínio; imagens s.namemc.com e challenges.cloudflare.com não são bloqueadas. Nenhuma extensão ou configuração global do navegador foi instalada. O navegador externo conserva seu próprio comportamento. Linux/macOS recebem o filtro visual; interceptação de rede nova é específica do Windows.
- Vitrine: favoritos passam a ter capa, título canônico e tipo mod/modpack. Capas de CDN reconhecido são sincronizadas ao perfil público, sem depender da instalação local do visitante. Entradas antigas continuam válidas; nomes sem arte identificada usam indicação alternativa. Aliases equivalentes como pixelmon/The Pixelmon Modpack não aparecem duplicados na vitrine.
- Editor: busca de mods e modpacks com 500 ms de espera e seis resultados, seleção explícita do projeto e descarte de consultas antigas. Falha do catálogo mantém a opção de nome manual. Capas locais não são enviadas como caminhos privados ao servidor.
- Pro: formatos de vitrine panorâmica com o primeiro favorito em destaque e pôsteres, além da grade disponível para todos. Servidor valida o plano; expiração volta à grade e conserva a escolha para renovação. Prévias diferentes e legendas PT/EN/ES.
- Salas: uma resposta de polling iniciada antes de criar/alterar a sala não pode sobrescrever o estado posterior. A capacidade anunciada considera a validade do Pro. A capacidade real continua autorizada no backend.
- Updates: instalar/aplicar um pacote invalida a consulta anterior. O término de uma consulta antiga não remove a nova consulta pendente. Mesmo ID instalado/remoto não produz aviso de atualização, mesmo com um sinalizador inconsistente do serviço.
- Discord: fila que descarta presenças antigas e ordena a limpeza após o envio em andamento. Capa do modpack ativo quando há URL pública reconhecida; modo privado conserva a imagem genérica. Textos limitados sem cortar Unicode, botões HTTPS validados e retorno real de aceitação no daemon.
- Downloads e configurações: mantidas as correções anteriores de duplicatas, concorrência por destino, nativas/manifestos, Forge e categorias de configuração. Nenhuma proteção de integridade foi removida para esconder falhas de terceiros.

## Migração necessária antes do deploy

`website/migrations/0018_profile_showcase.sql` adiciona pack_artwork com padrão [] a social_public_profiles. Não modifica os nomes favoritos existentes. Aplicar a migração antes de publicar o serviço que lê a coluna. Clients antigos preservam as capas dos nomes retidos ao salvar; removê-los também retira a arte correspondente. Migração e compatibilidade foram verificadas em SQLite temporário, não no banco de produção.

Incluir os arquivos novos na futura revisão, especialmente namemc-clean.js, ProfileFavourites.svelte, profileFavourites.ts, a migração e os testes novos. Não usar git add geral: há muitos arquivos antigos não rastreados.

## Evidências

- pnpm check: zero erros/avisos; check:website e pnpm build aprovados.
- cargo check Windows: aprovado, seis avisos preexistentes.
- Vitest: 229 testes, 39 arquivos aprovados.
- Site: 126 testes aprovados, incluindo capas persistidas, salvamento por client antigo, URLs rejeitadas, formatos Pro e expiração/renovação.
- Native NameMC: três testes aprovados, um teste externo ignorado. Validação de domínio mantém skins e verificações fora do bloqueio.
- Native Discord: um teste aprovado para Unicode, limites e botões.
- Native updates: sete testes aprovados para versão atual, prevenção de downgrade, edição/loader, preservação de mundos, mods pessoais e rollback de reserva.
- Native downloads: sete testes aprovados para concorrência, retomada, falha preservando arquivo, hash/cache, nativas e manifesto.
- P2P: 16 testes aprovados, dois externos ignorados. Incluem proxy criptografado, descoberta na interface física, reconexão e anúncio/retirada de mundos de convidados.
- Navegador: showcase-room-clean.browser.cjs passou para filtro visual e ads dinâmicos em fixture, capa/título único, busca de mod com tipo correto, persistência do perfil Pro e polling antigo. instance-settings-repair.browser.cjs, profile-recovery-gif.browser.cjs e appearance-studio.browser.cjs passaram; o último cobre PT/EN/ES, temas e largura compacta.
- git diff --check aprovado.

Logs privados: C:/Users/preda/AppData/Local/Luxmc-build/requests/refinement-20261024/showcase-*.log.

A captura showcase-room-2026-10-25/showcase-pro.png usa imagens e identidades de fixture; não representa a capa original do Pixelmon.

## Limites da validação

A consulta HTTP direta do NameMC respondeu 403; não houve tentativa de contornar o desafio. O filtro visual foi validado em HTML controlado e a integração WebView2 compilou, mas ainda não houve teste da janela nativa com o site real nesta build. O bloqueio cobre domínios/espaços reconhecidos; não é garantia de remover anúncios futuros ou do navegador externo.

A presença real não foi enviada à conta Discord do usuário. Os testes de RPC usam mocks e validação nativa do payload. Não houve partida completa entre dois PCs, carga real de vinte jogadores nem abertura de todos os modpacks. Não foi produzida uma nova release neste pedido.

Referências primárias consultadas: https://namemc.com/minecraft-skins e https://docs.discord.com/developers/platform/rich-presence. As APIs WebView2/Tauri foram conferidas no código das dependências locais 0.38.2/2.11.5.


## Integração autorizada em 25/10/2026

Após o usuário autorizar integrar tudo necessário, exportada cópia privada da tabela social_public_profiles e aplicada a migração 0018 no D1 luxmc-social. Deploy Pages concluído d49ec0ff. Produção /planos e /i18n.js respondem 200 e contêm os novos benefícios. Plus informa GIFs também nas pastas; Pro informa Vitrine/Pôster e capacidade LAN de até 20 incluindo host, com ressalva operacional de hardware/conexão. Wallpapers gratuitos nos três planos; PT/EN/ES, preços e limites preservados. 39 testes de site e teste ampliado de idiomas aprovados. Navegador em produção aprovou idiomas, desktop/mobile e ausência de overflow. Compilação do instalador Windows 4.0.0 local em andamento, sem publicação de novo binário GitHub até o registro desta nota.


Instalador Windows de teste concluído e instalado com /S /UPDATE, exit 0, em 25/10. Versão 4.0.0, build local 40133561062.1. Cópia em Downloads/Luxmc-4.0-teste-20261025.exe, SHA256 b8634010e132c70464f29ffdcb99bd3a77ba49e0879291b36708b4de7f1df1a1. Arquivo instalado corresponde ao compilado com diferença somente nos três bytes do marcador Tauri UNK/NSS, conferidos em memória. Comparação após instalação preservou 9 perfis, 2 contas, identidades, 398 arquivos protegidos, 6 wallpapers e 24.054 arquivos de instâncias. App iniciado para teste do usuário. Novo binário não foi publicado no GitHub.
