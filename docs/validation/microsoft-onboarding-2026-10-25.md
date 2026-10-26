# Microsoft: site e launcher

Revisão local do launcher `4.0.0`, build `40133561070.1`.

O primeiro acesso Microsoft no site apresenta a identidade confirmada e as instruções para baixar ou abrir o launcher. O formulário de senha Luxmc fica oculto durante essa etapa. Quando o serviço Minecraft responde ao site, a prévia usa o UUID oficial; quando essa consulta é bloqueada pelo provedor, a confirmação no launcher continua disponível. O site não inventa uma associação entre e-mail Microsoft e nickname Minecraft.

O launcher solicita um ID token Microsoft com nonce derivado do verificador PKCE. Essa prova fica somente na memória nativa, não é serializada para a interface e não é gravada no banco. A autenticação normal do Minecraft mantém a skin retornada pelo serviço oficial. A sincronização do portal roda separadamente e não bloqueia a entrada no launcher.

O servidor valida assinatura, emissor, audiência, tenant, nonce e validade do ID token. Para associar o UUID Minecraft, emite um desafio aleatório de uso único com validade de 120 segundos. O launcher confirma o desafio no serviço de sessões do Minecraft usando sua sessão oficial; o servidor verifica o resultado no endpoint `hasJoined`. O token de acesso Minecraft não é enviado ao site.

A associação preserva o ID social existente, amigos, plano e preferências. Um dispositivo novo sem dados protegidos pode reutilizar o perfil previamente associado ao Microsoft. Um perfil local que já tenha amigos, plano ativo, conta ou personalizações exige resolução explícita do conflito, em vez de descartar esses dados. Os testes verificam a reutilização da mesma conta em um segundo dispositivo.

A página aguarda a confirmação e consulta o estado a cada cinco segundos enquanto está visível. Ao confirmar, abre uma sessão do site e mostra o avatar pelo UUID Minecraft. O cookie de espera dura até sete dias para permitir instalar o launcher; o desafio de posse Minecraft continua durando apenas dois minutos. Versões anteriores têm instruções para abrir Meu plano e pedidos e confirmar o vínculo existente.

## Verificação

- `pnpm check`: zero erros e zero avisos.
- `cargo check`: aprovado, com seis avisos preexistentes.
- Dois testes nativos: nonce ligado ao PKCE e prova do portal excluída da serialização.
- `pnpm test:website`: 138 testes aprovados, incluindo assinaturas inválidas, expiração, replay, vínculo incorreto, preservação de amigos/Pro, segundo dispositivo, traduções e conclusão visual do fluxo com avatar oficial.
- Após separar o limite de consultas de estado do limite de tentativas de autorização, os 20 testes do fluxo Microsoft passaram novamente. Consultas periódicas da página não esgotam as tentativas de login.
- `pnpm build:website`: aprovado.
- Migração `0020_microsoft_launcher_sync.sql` aplicada ao D1 após exportação privada do banco. As novas tabelas guardam apenas hashes e associações; não guardam tokens Microsoft ou Minecraft.

As verificações usaram fixtures e inspeção de código. O login Microsoft real e o tempo de abertura do Minecraft ficam para o usuário testar, sem o agente entrar em sua conta ou iniciar o jogo.

A publicação inicial em `https://dd5ba23d.luxmc-r92.pages.dev` foi conferida no domínio de produção: página de conta e script responderam com a nova sequência, a configuração Microsoft estava habilitada e o início da autorização confirmou o mesmo aplicativo usado pelo launcher, nonce e PKCE S256. Esse controle não concluiu um login real.

A revisão final do site foi publicada em `https://72b73a92.luxmc-r92.pages.dev`, incluindo o limite separado das consultas de estado. O instalador local `40133561070.1` terminou com código zero; o executável instalado corresponde à compilação. A comparação preservou os dez perfis, as contas, identidades sociais, arquivos protegidos e wallpapers; a contagem de arquivos das instâncias permaneceu em 26.276. O launcher foi aberto para o usuário, sem iniciar Minecraft nem executar um login em sua conta.
