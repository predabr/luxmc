# Login Microsoft no site

Implementação local em 15/10/2026. GitHub e release 4.0 continuam sem publicação.

O botão em `/conta` usa autorização com código e PKCE, com estado de dez minutos vinculado ao navegador por cookie HttpOnly. O servidor troca o código na Microsoft, autentica Xbox Live/XSTS e consulta o perfil oficial do Minecraft Java. Os tokens Microsoft, Xbox e Minecraft não são persistidos; a sessão do site é independente, HttpOnly e revogável.

## Configuração para ativar

No cadastro Microsoft Entra do aplicativo Luxmc, habilitar contas Microsoft pessoais e cadastrar a plataforma **Web**, com retorno exato:

`https://luxmc-r92.pages.dev/api/microsoft/callback`

No Cloudflare Pages, projeto `luxmc`, configurar em produção:

- `MICROSOFT_CLIENT_ID`: ID do aplicativo cadastrado no Entra.
- `MICROSOFT_CLIENT_SECRET`: segredo desse aplicativo, exclusivamente como secret do Cloudflare. Nunca incluir no repositório, frontend ou conversa.
- `MICROSOFT_REDIRECT_URI`: opcional; o endereço acima é o padrão. Domínios próprios permitidos: `luxmc.top` e `www.luxmc.top`, sempre HTTPS e mesmo caminho.

Aplicar a migração `website/migrations/0017_microsoft_login.sql` antes do deploy. Sem configuração, o servidor retorna 503 com explicação; não há simulação de login.

## Preservação da identidade

O vínculo usa o UUID retornado pelo serviço autenticado do Minecraft. Um nickname igual não concede acesso ao perfil antigo, Pro, amigos ou funções de dono.

Para vincular um perfil Microsoft antigo, abrir primeiro **Meu plano e pedidos** no launcher recuperado e, no mesmo navegador autenticado, abrir `/conta` e usar **Entrar com Microsoft**. O acesso existente ao perfil e a confirmação da Microsoft são necessários. A conta autenticada deve ter o mesmo nickname Minecraft. O vínculo conserva o ID social, as amizades, o perfil e a assinatura existentes.

Após vincular, novas entradas Microsoft no site encontram a mesma identidade. O fluxo de autenticação Microsoft do launcher ainda usa sua sessão local; a persistência dessa identidade agora também faz parte do banco local e de suas próximas cópias de recuperação. Apagar todos os dados e cópias do launcher continua sendo uma exclusão permanente.

## Verificação

`pnpm test:microsoft-login` cobre estado/PKCE, origem, cookie de navegador, expiração, repetição de callback, conta estável, vínculo autenticado sem perder Pro/amigos, bloqueio de tomada por nickname, licença ausente e senha gerenciada pela Microsoft.

O teste real com a Microsoft depende do cadastro de retorno, segredo configurado e entrada pelo próprio usuário. Não declarar o login de produção validado apenas com os testes simulados.

Referência: [Microsoft — Authorization code flow](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow).
