# Luxmc 4.0 — revisão de conteúdo, atualizações e aparência

## Alterações

- Listas de mods ignoram diretórios internos dos loaders. A inspeção local identificou `memory_repo` como diretório de cache, não como mod sem capa. A aba Arquivos mantém acesso aos arquivos originais.
- Shaders e pacotes filtram arquivos de configuração `.txt`; diretórios extraídos recebem identificação própria em vez de tamanho zero. Ícones de shaders extraídos podem ser recuperados por correspondência do código com o arquivo ZIP instalado, seguida da identificação por hash no provedor.
- Uma única central distingue versão instalada e disponível. A consulta automática usa metadados; baixar e comparar o pacote exige abrir a revisão. Aplicar exige confirmação e usa o fluxo existente de backup e substituição.
- Novas áreas de ajustes: Interface e animações, Wallpapers, Catálogo e downloads. Os controles afetam configurações reais e a consulta manual do catálogo oficial. A consulta de versões já possuía cache de um minuto e fallback local.
- Migração de wallpaper inclui o arquivo ativo, normaliza URLs `file:` locais e não confirma uma importação se a persistência falhar. O armazenamento de mídia usa cópia independente do original.
- Entradas de conteúdo carregado depois da navegação também animam. A quantidade de animações é limitada; os efeitos terminam e são cancelados ao ocultar/desmontar a tela.
- Pro ganha fundo nos cartões de amigos usando a capa do perfil e acabamento Prisma no avatar. GIFs decorativos deixam de ser montados fora da área visível, em segundo plano, durante o jogo e em modo econômico. Imagens estáticas continuam visíveis.
- O servidor valida os benefícios por plano. Preferências premium sobrevivem ao vencimento e a salvamentos de clientes anteriores. Os novos detalhes usam campos opcionais, preservando os valores de moldura aceitos pelos launchers anteriores.
- O download do instalador pelo site tem limite para obter a resposta hospedada e fallback para o arquivo publicado no GitHub.

## Verificação

- `pnpm check`: zero erros e avisos.
- `pnpm test:run`: 224 testes aprovados. Os testes alterados de atualização e preparação de imagens também foram executados separadamente.
- `pnpm test:website`: 121 testes aprovados, incluindo limite do download, bloqueio Pro e preservação das escolhas em clientes anteriores.
- `cargo check`: aprovado; avisos de código não utilizado já existentes.
- Rust: três testes de miniaturas, três testes de wallpaper com arquivos reais e sete testes de atualização aprovados. Cobrem remoção do original, limpeza de cache, backup/preservação de mundos, mods desativados e seleção de versões.
- `pnpm build`: compilação de produção aprovada.
- Prévia visual com dados descartáveis: primeira linha de mod, shaders sem `.txt`, mundos, revisão e confirmação da atualização, remoção do aviso depois de aplicar, controles do catálogo e cartões Pro. As contas e os mundos pessoais não foram usados nesses testes de escrita.

## Site e Google

Sitemap XML/TXT revisado, incluindo `/planos`. HTTP 200, tipos de conteúdo corretos e ausência de bloqueio de indexação conferidos no domínio público. Publicação final do site: `b315171b`.

O Search Console confirmou o envio do sitemap em 24/10/2026. O teste ao vivo da página inicial confirmou que o URL está disponível para o Google e pode ser indexado. A solicitação manual de indexação foi recusada por cota diária excedida. A página constava como rastreada, mas ainda não indexada; o envio não confirma indexação.

## Limites

Os testes não demonstram tempo de abertura idêntico para todos os modpacks ou qualquer computador. Projetos sem imagem própria ou sem correspondência verificável continuam com indicação alternativa. O relato da janela vazia foi esclarecido como interação com o Gerenciador de Tarefas; não foi reproduzido como falha ao abrir o launcher pelo atalho.
