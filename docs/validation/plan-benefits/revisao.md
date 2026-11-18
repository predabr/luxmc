# Benefícios de perfil — validação local

## Resultado

Free mantém avatar, capa, GIF, nome, status, bio e oito favoritos. Plus acrescenta seis paletas, Halo, Contorno duplo e dezesseis favoritos. Pro acrescenta Órbita, Brilho da capa, Pulso do avatar, composição centralizada e vinte e quatro favoritos. Coleções e perfis de aparência sincronizados mantêm os limites de 6/12/24 e 0/6/20.

Uma definição tipada compartilhada controla as opções no launcher e no site. As API consultam o plano da identidade autenticada, recusam valores desconhecidos e não aceitam atribuição de plano, role ou tags pelo editor. A gravação no site altera somente a assinatura visual da própria conta. O comprovante e a aprovação de assinaturas seguem o fluxo anterior; novos pedidos continuam pausados.

Estilos pagos vencidos são normalizados para o plano ativo na leitura e conservados no banco para renovação. Favoritos existentes permanecem salvos; novos favoritos acima do limite ativo são recusados. Clientes anteriores recebem até oito favoritos e mantêm o estilo e os favoritos adicionais ao salvar um perfil. O cliente atualizado solicita até vinte e quatro.

## Conferências

- `pnpm check`: zero erros e zero avisos.
- `pnpm build`: produção gerada com sucesso.
- `pnpm test:website`: 75 testes aprovados, incluindo autorização dos planos, site/launcher, expiração, renovação, compatibilidade e valores maliciosos de estilo.
- `tests/profile-benefits.browser.cjs`: Free, Plus e Pro em temas claro e escuro; prévia e gravação; contadores de favoritos; layout compacto; página de planos em desktop/móvel; movimento reduzido; Órbita animada com movimento habilitado; controles de contratação desativados enquanto pedidos estão pausados. Nove cenários, sem erros de JavaScript ou transbordamento horizontal, também conferidos usando a compilação de produção. Os favoritos de Plus e Pro ultrapassam oito nos cenários para verificar a negociação do limite de leitura.
- `tests/profile-editor.browser.cjs`: regressão de GIF, imagem grande, enquadramento, persistência, ordenação, prévia, erro de gravação e proteção do rascunho aprovada.
- Rust: o arquivo `social.rs` foi analisado sintaticamente com rustfmt, edição 2021. `cargo check` encerrou com código 101 por E0463 em `yoke_derive` e `derive_more_impl`, mantendo o impedimento de compilação já diagnosticado nas dependências pelo Windows.

## Limites e publicação

Os testes de navegador usam o frontend real com API nativa simulada. Eles não comprovam a instalação de um novo executável. O launcher instalado não foi aberto, a versão não foi alterada e nenhum código foi enviado ao GitHub ou à Cloudflare nesta tarefa.

A publicação deverá aplicar a migração `0016_profile_style.sql` antes das novas funções de API e entregar um launcher recompilado, pois a solicitação nativa de perfil agora transporta `profileStyle`. O plano de servidores hospedados permanece adiado.
