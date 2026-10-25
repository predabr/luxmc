# Identidade Microsoft e apresentação — 25/10/2026

## Referências

- [Microsoft OpenID Connect](https://learn.microsoft.com/en-us/entra/identity-platform/v2-protocols-oidc): autenticação da conta do site por ID token, separada das APIs do jogo.
- [Validação de ID tokens](https://learn.microsoft.com/en-us/entra/identity-platform/id-tokens): assinatura, audiência, emissor, nonce e validade.
- [jose](https://github.com/panva/jose): validação JWT e JWKS compatível com Cloudflare Workers.
- [GitHub — README](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes): acesso rápido a uso, instalação, recursos e suporte.
- [GitHub — imagens e links relativos](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax): banner no próprio repositório e texto alternativo.
- [web.dev — animações eficientes](https://web.dev/articles/animations-guide): priorizar transformações e opacidade, preservar movimento reduzido e interromper efeitos fora de uso.

## Alterações

- OIDC com validação criptográfica por jose 6.2.13. Troca de código usa o segredo somente no servidor. O site deixa de depender do endpoint Minecraft bloqueado; a autenticação do jogo permanece no launcher.
- Primeiro vínculo exige perfil Luxmc autenticado e confirmação explícita das duas contas. Confirmação expira em dez minutos, usa cookie HttpOnly/Secure e rejeita mudanças de conta em outra aba. Não junta contas por nome ou e-mail e não altera o Minecraft UUID.
- Migração adicional 0019 conserva as tabelas e vínculos Minecraft anteriores. Nenhuma conta, amizade ou assinatura é removida pela migração.
- Confirmação de identidade e orientações em português, inglês e espanhol. Código de amizade para atribuição manual de plano recebe explicação e exemplo no painel do dono.
- Catálogo do site mostra todos os loaders declarados e a versão do Minecraft em ordem numérica. Logos não são pequenos pôsteres duplicados no banner.
- Apresentação do produto com grade equilibrada de quatro recursos, foco visível e contador legível no estado selecionado.
- README com banner vetorial local, navegação, tabela de recursos e comparação dos benefícios existentes dos planos. Licença vigente GPL-3.0-only e proposta sem vigência continuam distintas.

## Verificação

17 verificações de autenticação assinada passaram: nonce, audiência, emissor, tenant, validade, assinatura forjada, state, replay, vínculo, troca de contas, expiração e preservação de amigos/plano. Conjunto relacionado de contas, sessão e interface do site: 43 verificações passaram. Compilação TypeScript e geração do site aprovadas. Nenhuma entrada na conta Microsoft real foi feita pelo agente; essa validação continua com o usuário.

O usuário forneceu três capturas atuais em 25/10/2026: biblioteca, catálogo e amigos. Os PNGs originais foram copiados sem alterar seu conteúdo para `website/assets/captures-4.0`. Elas substituem as capturas anteriores na apresentação da página inicial, na página de planos, nos metadados de compartilhamento e no README. O destaque e o tour mantêm a proporção completa das imagens, com expansão disponível no tour.
