# Perfis e amizade — correção da publicação, 13/10/2026

O domínio principal e a publicação anterior retornavam HTTP 404 com `Unknown action` para `profile_get`, `profile_save`, `owner_users`, bloqueios e outras ações atuais. `sync` e `search` continuavam reconhecidos. A causa era a descoberta de `functions/` na raiz do repositório ao publicar a pasta de arquivos estáticos: os dois handlers antigos da raiz reconheciam somente as ações básicas. A implementação atual estava em `website/functions/` e os testes locais importavam essa implementação.

Os handlers antigos agora reexportam a implementação gerada atual. A publicação passa pelo script `scripts/deploy-website.mjs`, que executa o Wrangler com diretório de trabalho fixado em `website/`, inclusive quando chamado de outra pasta. O projeto Cloudflare se chama `luxmc` e usa o domínio `luxmc-r92.pages.dev`. Nenhuma publicação no GitHub foi feita.

O build completo do site passou. Foram aprovados 31 testes de perfis, contas, benefícios e publicação, e 37 testes de comércio, sessões, interface, artefatos e publicação. Os dois testes de publicação aparecem em ambos os grupos. O teste de publicação executa uma ferramenta substituta a partir de outra pasta e confere o diretório e os argumentos realmente recebidos; os handlers da raiz e do site mantêm as mesmas ações e a exigência de autenticação.

Publicado em `https://e8205c85.luxmc-r92.pages.dev`. No domínio principal, nove ações sociais atuais agora retornam HTTP 401 para pedidos sem credenciais, em vez de `Unknown action`. A configuração do endpoint de sessão também foi confirmada sem credenciais.

O executável instalado foi usado para consultar o serviço real com a identidade da conta atual: perfil próprio recuperado, lista com três amigos aceitos, três perfis públicos recuperados e diretório do dono recuperado. Uma tentativa de salvar descrição inválida foi rejeitada pela validação de conteúdo atual; nenhum perfil foi alterado. A identidade própria retornada no perfil corresponde à retornada pela lista social. A publicação não exigiu reinstalar o launcher.

Os formulários já abertos com erro precisam ser fechados e abertos novamente para refazer a consulta. O processo original do launcher foi mantido aberto; somente o processo daemon criado para a validação foi encerrado.
