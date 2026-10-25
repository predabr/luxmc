# Revisão em 25/10/2026

O caminho do site foi separado da autenticação do jogo. A revisão usa o fluxo de código OpenID Connect da Microsoft (`openid profile`) com PKCE, state por aba, nonce, assinatura RS256, audiência, emissor, tenant e validade conferidos por `jose`.

O vínculo persistente identifica o sujeito assinado pela Microsoft. Um nome ou e-mail exibido não dá acesso a um perfil Luxmc. No primeiro vínculo, o usuário precisa autenticar o perfil Luxmc e confirmar as duas contas; uma sessão de “Meu plano e pedidos” do launcher pode autenticar o perfil existente. Amigos, plano e ID social são mantidos. Não há criação automática de um segundo perfil nem transferência de assinatura por nickname.

A primeira autenticação pode terminar na tela de confirmação de vínculo; isso é uma etapa explícita do cadastro, não uma falha do Minecraft. Após confirmar, o botão Microsoft entra diretamente. Sem um perfil anterior, é possível criar uma conta Luxmc pelo formulário e então confirmar o vínculo pendente.

Essa revisão não afirma a posse do Minecraft com base no ID token Microsoft. A verificação do jogo permanece no launcher e o fluxo anterior de comprovação Minecraft não é substituído por uma identidade inventada.

Migração necessária: `website/migrations/0019_microsoft_identity.sql`. Validar a entrada real no Entra depois da publicação. Os registros abaixo descrevem o diagnóstico histórico do fluxo anterior.

## Histórico do fluxo anterior

# Login Microsoft no site

Implementação publicada no site em 15/10/2026. A publicação dos instaladores 4.0 é acompanhada separadamente em `docs/validation/release-4.0-polish.md`.

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


## Configuração aplicada em 15/10/2026

A plataforma Web e o retorno de produção foram salvos no Entra. As duas variáveis Microsoft foram gravadas como segredos de produção do Cloudflare Pages; a credencial tem duração de seis meses. A migração 0017 foi aplicada e o botão público abriu o seletor oficial da Microsoft.

O usuário identificou a conta Microsoft do Spect3rBW. A tentativa real passou pela Microsoft, Xbox Live e XSTS. A chamada do servidor Cloudflare para `api.minecraftservices.com/authentication/login_with_xbox` recebeu HTTP 403 com uma resposta classificada como bloqueio do provedor. A entrada completa e o vínculo de produção continuam pendentes. Não selecionar outra conta nem criar uma identidade por nickname como alternativa.


## Diagnóstico do retorno de produção

O primeiro erro ocorria no modo de redirecionamento da requisição. O cliente agora usa `manual` e rejeita respostas de redirecionamento sem enviar credenciais a outro destino. Com esse ajuste, a troca do código Microsoft e as etapas Xbox funcionaram. O diagnóstico registra apenas serviço, status e códigos classificados; não registra corpos de resposta, tokens, códigos de autorização ou segredos.

A identificação explícita das requisições como Luxmc não removeu o HTTP 403 do endpoint Minecraft. O problema restante não é uma senha solicitada ao usuário, callback ausente nem segredo inválido. Foi solicitada informação sobre um backend já existente fora do Cloudflare. Não contratar hospedagem, trocar IPs para contornar o bloqueio ou enfraquecer a confirmação da identidade Minecraft.

O usuário confirmou que não tem outro servidor disponível. Em produção, a tela agora explica o bloqueio e orienta o uso do launcher. A publicação do instalador 4.0 foi concluída independentemente dessa pendência de autenticação web.
