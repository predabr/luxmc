# Luxmc Plus e Pro — Pix com aprovação manual

Implementação da fila de compras e do painel do dono, sem contratação de infraestrutura paga.

## Acesso

No launcher, abra seu perfil e use **Meu plano e pedidos**. O dono recebe também **Central do dono · aprovar planos**. O link usa um ingresso de acesso de uso único, válido por 60 segundos. O site troca esse ingresso por um cookie seguro, inacessível ao JavaScript. A sessão do dono dura 15 minutos e expira após 5 minutos sem atividade. A entrada pelo launcher exige confirmar a senha antes de acessar a fila e os comprovantes. A identidade permitida é `OWNER_SOCIAL_ID`, verificada no servidor a cada pedido.

Quem tem uma conta Luxmc também pode entrar em `/conta` e abrir `/planos`. Contas Microsoft e locais usam o acesso pelo launcher, evitando criar uma identidade diferente no site. Saber o endereço `/painel-luxmc` não permite acessar dados nem aprovar pedidos.

O dono também pode entrar diretamente com a senha própria da central, independente da Microsoft. Ela é armazenada no servidor somente como um hash PBKDF2 protegido por um segredo do serviço. Tentativas são limitadas por endereço e por serviço. Alterar a senha encerra as outras sessões do painel. A configuração dessa senha exige uma sessão já autenticada do dono, ou a identidade nativa do dono.

## Recebimento

O banco inicial é Nubank. A chave inicial foi reaproveitada do Pix público existente no site. Novos pedidos começam pausados: o dono revisa a configuração e pode abrir o recebimento no painel. Cada pedido conserva o banco, a chave, o valor e o prazo que estavam configurados na criação. Alterar a configuração não troca o destinatário de um pedido existente.

1. O comprador escolhe Plus ou Pro e recebe um pedido ligado à sua identidade.
2. Faz o Pix no seu banco e envia PNG, JPEG, WebP ou PDF de até 1 MB.
3. O dono abre o comprovante e confere o crédito e o valor no extrato do Nubank.
4. Informa o identificador da transação Pix, confirma a conferência e aprova, ou rejeita com um motivo.
5. O servidor atualiza o plano; o launcher conectado consulta alterações a cada 30 segundos.

O comprovante não confirma o pagamento automaticamente. A operação não usa API bancária, não debita valores e não faz renovação automática. O mesmo identificador Pix não pode aprovar dois pedidos. Comprovantes alterados e pedidos cancelados invalidam uma revisão anterior. Aprovações, atribuições manuais e mudanças de configuração ficam no histórico.

## Benefícios disponíveis

| Plano | Preço por 30 dias | Coleções públicas | Perfis de aparência sincronizados |
|---|---:|---:|---:|
| Free | Grátis | 6 | 0 |
| Plus | R$ 9,90 | 12 | 6 |
| Pro | R$ 19,90 | 24 | 20 |

Plus e Pro recebem um selo verificado no perfil e na lista de amigos. Os perfis de aparência guardam espaçamento, estilo e velocidade de animações, FPS de wallpaper, qualidade de GIFs, opacidade e preferências de desempenho visual. Arquivos locais, mundos, caminhos e credenciais não são enviados. As funções gratuitas existentes continuam disponíveis.

O prazo inicia na aprovação. Uma renovação do mesmo plano acrescenta 30 dias ao vencimento existente ou à data atual, se já venceu. Mudar de plano inicia o novo prazo na aprovação. Uma atribuição manual substitui o vencimento; o motivo é opcional e recebe uma descrição padrão quando vazio. Ela não registra um pagamento.

A central mostra o plano da conta autenticada e oferece **Alterar meu próprio plano**, que seleciona exclusivamente o ID autenticado. Para outra pessoa, busque pelo código copiado na aba Amigos (`nickname#código`) ou pelo ID completo. O nome sozinho apenas retorna opções. O formulário mostra o código e o ID e exige colar o código do jogador antes de atribuir. O servidor recusa códigos incompatíveis e alterações feitas a partir de uma revisão antiga. Pedidos aprovados usam o ID do comprador automaticamente, sem digitar o destinatário.

Ao vencer, o plano efetivo volta ao Free. Coleções e perfis existentes são preservados. O servidor limita novas inclusões conforme o plano atual. Hospedagem de servidores, mundos em nuvem e aumento pago da capacidade P2P não fazem parte da oferta implementada.

## Armazenamento e privacidade

Comprovantes são guardados no D1 com autorização por comprador ou dono e respostas `no-store`. PDFs são entregues para download, com isolamento de conteúdo. Não há URLs públicas de comprovantes. A versão inicial limita o conjunto dos comprovantes a 100 MB e bloqueia novos envios quando atingir essa capacidade. Arquivos de pedidos concluídos são removidos após 90 dias; o histórico permanece. A limpeza é acionada pelas requisições do serviço, portanto ocorre no acesso seguinte ao prazo.

As compras são revisadas manualmente. Não foi feita transferência real nem simulado crédito bancário em produção. Os testes usam contas e arquivos de exemplo em banco local isolado.

## Implantação

Aplique também `website/migrations/0015_commerce_security.sql` antes de publicar esta revisão. Execute `pnpm test:website`, `pnpm check`, `pnpm test:run`, `pnpm build`, `cargo check` e `pnpm build:website`. Aplique `website/migrations/0013_memberships.sql` e `website/migrations/0014_commerce_owner_login.sql` ao D1 antes de publicar o site e o worker social. O launcher e o daemon precisam ser recompilados para expor o comando `membership_request`. A versão atual permanece 3.8.0; este trabalho não cria uma release no GitHub.

O painel inicia com vendas pausadas até a revisão do dono. O launcher anterior continua funcionando, mas não mostra os novos controles de plano. Certificação de assinatura do executável e hospedagem 24 horas permanecem etapas separadas.

Em 11/10/2026, `cargo check` passou, mas a geração do instalador foi bloqueada pelo Controle de Aplicativo do Windows (erro 4551 ao carregar uma biblioteca do compilador). Nenhuma proteção do Windows foi desativada. A integração do launcher está implementada no código; esta execução não gerou nem instalou um executável novo.

## Central simplificada e proteção do acesso

A central possui quatro áreas: Pedidos, Planos, Histórico e Segurança. Recebimento e alteração de senha ficam recolhidos por padrão. A interface adapta a navegação ao celular, preserva formulários em edição e mostra erros junto às ações.

O servidor exige sessão do dono confirmada por senha para a fila, a busca e os comprovantes privados. Ações sensíveis exigem confirmação nos últimos 5 minutos. Tokens antigos do launcher continuam válidos para o estado do próprio plano e os links de acesso, mas não concedem acesso administrativo por si sós. Reautenticar gira o token de sessão. O limite absoluto é 15 minutos; a inatividade de 5 minutos também é conferida no servidor. Atualizações automáticas da fila não renovam a atividade.

Requisições autenticadas por cookie exigem um token CSRF ligado à sessão. A origem do navegador precisa corresponder ao site. Os cookies usam `HttpOnly`, `Secure` e `SameSite=Strict`. Respostas privadas usam `no-store`, e a central restringe scripts, conexões, estilos, imagens e enquadramento por CSP.

Em Segurança, configure autenticação em duas etapas com o aplicativo autenticador do seu celular. A chave é entregue uma vez durante o pareamento, e fica criptografada com AES-GCM no banco. Só confirme a ativação após adicionar o QR ao aplicativo e verificar seu primeiro código. O TOTP segue RFC 6238, com janela limitada e bloqueio de reutilização de códigos. Os oito códigos de recuperação aparecem uma única vez e ficam armazenados somente como hashes. Usar um deles exige a senha e revoga as outras sessões e links. A desativação exige novamente senha e código válido. A ativação real depende do pareamento pelo dono; a implantação não configura um autenticador em nome dele.

O botão Encerrar outras sessões revoga sessões da central e ingressos pendentes. Nenhum pagamento, preço, vencimento do Pro ou chave Pix é alterado por esta revisão. Nenhuma medida oferece garantia absoluta contra invasões; mantenha o autenticador em dispositivo sob seu controle.

Referências: https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html, https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html, https://www.rfc-editor.org/rfc/rfc6238.