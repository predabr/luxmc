# Luxmc 4.0 — revisão de conteúdo, atualizações e aparência

## Alterações

- Listas de mods ignoram diretórios internos dos loaders. A inspeção local identificou `memory_repo` como diretório de cache, não como mod sem capa. A aba Arquivos mantém acesso aos arquivos originais.
- Shaders e pacotes filtram arquivos de configuração `.txt`; diretórios extraídos recebem identificação própria em vez de tamanho zero. Ícones de shaders extraídos podem ser recuperados pela documentação do projeto ou por correspondência do código com o ZIP instalado. As nove variantes locais do Better MC declaram Euphoria Patches, cujo [site oficial](https://www.euphoriapatches.com/download/) indica seu projeto no Modrinth. A consulta identificou o projeto e sua imagem respondeu HTTP 200.
- Uma única central distingue versão instalada e disponível. A consulta automática usa metadados; baixar e comparar o pacote exige abrir a revisão. Aplicar exige confirmação e usa o fluxo existente de backup e substituição.
- A preparação dos ícones consulta os provedores enquanto a leitura local continua. Duas filas limitam as consultas; abrir o jogo ou ocultar a janela suspende novas solicitações.
- Novas áreas de ajustes: Interface e animações, Wallpapers, Catálogo e downloads. Os controles afetam configurações reais e a consulta manual do catálogo oficial. A consulta de versões já possuía cache de um minuto e fallback local.
- Migração de wallpaper inclui o arquivo ativo, normaliza URLs `file:` locais e não confirma uma importação se a persistência falhar. O armazenamento de mídia usa cópia independente do original.
- Entradas de conteúdo carregado depois da navegação também animam. A quantidade de animações é limitada; os efeitos terminam e são cancelados ao ocultar/desmontar a tela.
- Pro ganha fundo nos cartões de amigos usando a capa do perfil e acabamento Prisma no avatar. GIFs decorativos deixam de ser montados fora da área visível, em segundo plano, durante o jogo e em modo econômico. Imagens estáticas continuam visíveis.
- O servidor valida os benefícios por plano. Preferências premium sobrevivem ao vencimento e a salvamentos de clientes anteriores. Os novos detalhes usam campos opcionais, preservando os valores de moldura aceitos pelos launchers anteriores.
- A regra de imagens animadas cobre também GIFs otimizados como WebP: o Free não pode ativá-los, o vencimento oculta a animação e a renovação a recupera. Esta correção de servidor atende também os instaladores anteriores.
- O download do instalador pelo site tem limite para obter a resposta hospedada e fallback para o arquivo publicado no GitHub.

## Verificação

- `pnpm check`: zero erros e avisos.
- `pnpm test:run`: 224 testes aprovados. Os testes alterados de atualização e preparação de imagens também foram executados separadamente.
- `pnpm test:website`: 122 testes aprovados, incluindo limite do download, bloqueio Pro, WebP animado e preservação das escolhas em clientes anteriores.
- `cargo check`: aprovado; avisos de código não utilizado já existentes.
- Rust: quatro testes de miniaturas, três testes de wallpaper com arquivos reais e sete testes de atualização aprovados. Cobrem remoção do original, limpeza de cache, backup/preservação de mundos, mods desativados e seleção de versões. Uma verificação adicional foi executada diretamente nas nove pastas de shaders da instância, sem alterar seus arquivos.
- `pnpm build`: compilação de produção aprovada.
- Prévia visual com dados descartáveis: primeira linha de mod, shaders sem `.txt`, mundos, revisão e confirmação da atualização, remoção do aviso depois de aplicar, controles do catálogo e cartões Pro. As contas e os mundos pessoais não foram usados nesses testes de escrita.

## Site e Google

Sitemap XML/TXT revisado, incluindo `/planos` e datas de alteração conferidas no histórico do projeto. HTTP 200, tipos de conteúdo corretos e ausência de bloqueio de indexação conferidos no domínio público. Publicação dos recursos do site: `b315171b`.

O Search Console confirmou o envio do sitemap em 24/10/2026. O teste ao vivo da página inicial confirmou que o URL está disponível para o Google e pode ser indexado. A solicitação manual de indexação foi recusada por cota diária excedida. A página constava como rastreada, mas ainda não indexada; o envio não confirma indexação.

## Limites

Os testes não demonstram tempo de abertura idêntico para todos os modpacks ou qualquer computador. Projetos sem imagem própria ou sem correspondência verificável continuam com indicação alternativa. O relato da janela vazia foi esclarecido como interação com o Gerenciador de Tarefas; não foi reproduzido como falha ao abrir o launcher pelo atalho.

## Correções adicionais de instalação, sala e idiomas

- Horror OneBlock: conferidos os seis JARs reais de Cuckoo Clock, Eyes In The Dark e Shy Guy. Todos declaram internamente a versão 1.0.0. A seleção agora inclui overrides com hash conferido e compara sufixos de versão somente dentro do mesmo mod, loader e prefixo de arquivo. Versões antigas ficam em backup; seus registros de override são retirados para o reparo não reinstalá-las.
- O teste nativo executado em uma cópia isolada dos seis arquivos manteve as três versões novas. A pasta original do usuário não foi alterada pelo teste.
- Salas Pro: capacidade nativa de 20 incluindo anfitrião, obtida por consulta autenticada do plano. Free e Plus conservam 10; indisponibilidade do serviço não bloqueia a sala básica. Os limites de streams e mensagens da sala foram ajustados. O agente adapta o limite integrado nas classes reconhecidas enquanto a sessão está ativa e conserva o limite normal fora dela.
- O teste Java validou os limites 20/10 e a expiração em Java 17/25, além dos ganchos de descoberta no bytecode disponível de Minecraft/Forge. Não é um teste de carga com vinte PCs.
- A sala mostra endereço direto copiável por mundo recebido. O anúncio da rede virtual também usa as interfaces locais e conserva a verificação de cliente local nos proxies.
- Rust: 34 testes de rede aprovados, incluindo descoberta pela interface física, encaminhamento de mundo de convidado, retirada do mundo e preservação de conexão. Duas verificações externas continuaram ignoradas.
- Site: 124 testes aprovados. Tentativas Microsoft independentes mantêm PKCE, vínculo HttpOnly, expiração e uso único. Conta e mensagens seguem PT/EN/ES, sem limpar credenciais digitadas ao trocar o idioma. A área de plano do launcher também foi traduzida.
- Teste real no domínio público: o retorno da Microsoft passou pela validação de estado, mas a etapa Minecraft foi bloqueada pelo provedor (HTTP 403/Cloudflare). Login completo do site não foi considerado aprovado.
- Cutscene: limite de desenho de 60 quadros por segundo, suspensão de frames quando oculta e indicador discreto das etapas. Sem alteração da marca.

A partida do modpack entre os dois computadores do usuário e o acesso Microsoft completo no site continuam pendentes de validação/liberação externa. Esses limites não são apresentados como resultados concluídos.

## Publicação oficial conferida

Build pública: [v4.0.0-build.40133561061.1](https://github.com/predabr/luxmc/releases/tag/v4.0.0-build.40133561061.1), com código `3b2aa79`. Os seis jobs de validação/compilação e a verificação final do workflow concluíram com sucesso. O README e as notas receberam uma atualização posterior somente de documentação.

Foram conferidos os onze assets, as assinaturas Ed25519 oficiais do manifesto e de SHA256SUMS, a revisão numérica, os tamanhos e os destinos de download. O instalador completo baixado pelo site corresponde ao SHA-256 `6bde5abebabdeae9cbeb38c810b349b14ec72a338672ddd5fea2905e91436c1a`, com 21.659.682 bytes.

No runner Windows, instalação, reinstalação da mesma versão e desinstalação retornaram zero, preservando os dados protegidos do teste. Os testes de recuperação de download interrompido passaram. O Defender do runner não encontrou ameaças no instalador e no executável; isso não equivale a assinatura Authenticode nem aprovação universal dos antivírus.

A leitura adicional dos metadados dos 180 JARs do Horror OneBlock confirmou somente os três grupos de duplicatas relatados. A proposta de reparo elimina esses três grupos, sem modificar a pasta original durante a auditoria.
