# Luxmc Plus, Pro e Mundos 24 horas

Plano de produto e implementação — 11 de outubro de 2026.

Este documento é uma proposta. Preços, limites, metas e políticas abaixo precisam ser validados antes da venda. A elaboração deste plano não contrata infraestrutura, habilita cobranças, publica alterações ou implementa os serviços.

## 1. Modelo recomendado

Manter o launcher gratuito e financiar sua evolução com serviços opcionais: sincronização, armazenamento de backups, atendimento e hospedagem de servidores. Não reduzir a segurança, a velocidade dos downloads ou a funcionalidade já disponível para criar uma vantagem artificial nos planos pagos.

Usar três produtos com nomes claros:

- **Luxmc Livre:** o launcher e suas funções locais.
- **Luxmc Plus e Luxmc Pro:** serviços da conta e benefícios no site e no launcher.
- **Luxmc Mundos:** hospedagem de Minecraft que continua funcionando com o computador do cliente desligado.

A hospedagem tem preço próprio. Para evitar cobrar duas mensalidades pelo mesmo cliente, um pacote de hospedagem pode incluir Plus ou Pro durante sua vigência. A compra migra a cobrança existente, com crédito proporcional devidamente calculado; não cria uma assinatura adicional escondida.

O proprietário contrata a máquina e a administração. Seus convidados não precisam comprar Plus ou Pro para entrar no jogo. Acesso ao Minecraft continua sujeito à licença e à autenticação do jogo.

## 2. Assinaturas propostas

| Benefício | Luxmc Livre | Luxmc Plus | Luxmc Pro |
| --- | --- | --- | --- |
| Preço mensal proposto | Grátis | R$ 9,90 | R$ 19,90 |
| Preço anual proposto | — | R$ 99 | R$ 199 |
| Instâncias, mods, modpacks e funções atuais | Sim | Sim | Sim |
| LAN/P2P atual, Shield e velocidade dos downloads | Mesma base | Mesma base | Mesma base |
| Backups locais e exportação dos próprios arquivos | Sim | Sim | Sim |
| Badge da assinatura no site e no launcher | — | Plus | Pro |
| Coleções novas de temas e molduras originais do Luxmc | Recursos atuais | Coleção Plus | Coleções Plus e Pro |
| Sincronização de preferências entre dispositivos | Base atual | Ampliada | Ampliada |
| Armazenamento de backups em nuvem | — | 5 GB | 20 GB |
| Backups de mundos locais em nuvem | — | Manual | Manual e automático ao fechar o jogo |
| Histórico pretendido | — | Até 3 versões por mundo | Até 10 versões por mundo |
| Atendimento | Comunidade | Padrão | Fila prioritária em horário divulgado |

As versões de backup contam dentro da cota total. Cinco GB de mundo e três cópias não cabem em cinco GB de armazenamento. Mostrar o espaço efetivamente utilizado, a política de rotação e quais cópias serão removidas. Não apagar o mundo local se a nuvem estiver cheia.

Preferências sincronizadas devem excluir caminhos absolutos do PC, senhas, tokens Microsoft, credenciais de servidor e configurações incompatíveis entre sistemas. Conflitos devem gerar uma escolha entre cópias, nunca sobrescrever silenciosamente o mundo mais recente.

Temas e molduras pagos são criações originais de interface. Não vender capas do Minecraft nem vantagens dentro do jogo. Preservar as opções atuais de GIF, wallpaper e personalização.

## 3. LAN com até 30 pessoas

Há três capacidades diferentes: membros conectados à sala do launcher, conexões do túnel e jogadores efetivamente aceitos pelo Minecraft. Aumentar um número na interface não resolve o limite do jogo nem a carga no PC anfitrião.

Antes de oferecer salas maiores:

1. Medir o comportamento atual com dois PCs e duas redes reais.
2. Retestar o caso reportado: One Terror Block, Minecraft 1.20.1, mesma versão do pack nos dois lados.
3. Testar descoberta LAN, porta correta, anúncio local do mundo, conexão ao servidor e reconexão separadamente.
4. Cobrir mudanças de porta ao reabrir para LAN, firewall, Wi-Fi, perda de conexão e NAT/CGNAT.
5. Aumentar a carga gradualmente para 5, 10, 20 e 30 jogadores, observando CPU, memória, upload, latência e tempo dos ticks.
6. Confirmar que o limite real de jogadores do mundo permite a capacidade anunciada; não instalar uma modificação incompatível para aumentar esse limite.

O caminho direto deve ser preferido quando funciona. Um retransmissor pode servir como alternativa para redes incompatíveis, com tráfego e custo medidos. Não substituir o transporte atual sem um teste comparativo que preserve os casos já funcionando.

Permanecer na sala ao trocar de aba e recuperar seu estado ao retornar. Mostrar os perfis, os jogadores conectados e um diagnóstico simples que diferencie “amigo conectado”, “mundo encontrado” e “Minecraft pronto para entrar”.

**Decisão recomendada:** tratar 30 jogadores como meta técnica de validação. Não anunciar “30 pessoas garantidas em qualquer modpack” e não vender aumento de vagas por assinatura antes de esclarecer o enquadramento das regras da Mojang.

As diretrizes limitam acesso a servidores condicionado a produtos externos e modificações que verificam esses produtos para afetar funções do jogo. A aplicação exata a uma LAN paga precisa de revisão; este plano não presume aprovação. [Diretrizes de uso do Minecraft](https://www.minecraft.net/en-us/usage-guidelines).

## 4. Pacotes de hospedagem

| Pacote proposto | Memória do contêiner | Disco ativo | Assinatura incluída | Uso inicial pretendido | Preço mensal para simulação |
| --- | --- | --- | --- | --- | --- |
| **Luxmc Mundos — Origem** | 4 GB | 30 GB | Plus | Vanilla e packs leves previamente testados | R$ 49,90 |
| **Luxmc Mundos — Horizonte** | 8 GB | 60 GB | Pro | Grupos maiores e packs aprovados para essa configuração | R$ 89,90 |
| **Luxmc Mundos — Expansão** | 12–16 GB | Conforme cotação | Pro | Packs pesados e necessidade de CPU maior | Definido após testes e cotação |

Esses preços são hipóteses comerciais, não cotações de fornecedor. A memória inclui consumo fora do heap Java; reservar folga entre o limite do contêiner e `-Xmx`. O disco ativo é separado da cota de backups em nuvem.

Definir a quota de CPU, se é compartilhada ou reservada, e sua capacidade real antes da oferta. RAM não determina sozinha o número de jogadores. Evitar “CPU ilimitada”, “mods ilimitados” e promessas de FPS para servidores.

Origem teria backup diário e Horizonte a cada seis horas, limitados à cota contratada. Se os backups de um servidor e os mundos locais compartilharem a cota Plus/Pro, a interface precisa mostrar isso explicitamente. Antes do lançamento, simular mundos crescendo e definir se será necessário vender armazenamento adicional.

Começar com Java. Oferecer Bedrock em etapa posterior usando o servidor dedicado oficial, catálogo e rede próprios. Não prometer compatibilidade entre um modpack Java e clientes Bedrock.

## 5. Jornada do cliente

1. Entrar no site com uma conta vinculada ao perfil usado no launcher.
2. Escolher Plus, Pro ou um pacote Luxmc Mundos, vendo o total, a renovação e os limites antes de pagar.
3. Confirmar o pagamento no provedor.
4. Receber o benefício no site e no launcher depois da confirmação do backend.
5. Para hospedagem: dar nome ao mundo, escolher região, versão e um pack de servidor compatível, ou importar um mundo.
6. Acompanhar preparação com etapas reais, percentual quando disponível e cancelamento efetivo.
7. Clicar em **Jogar**, convidar amigos e administrar o servidor pela mesma conta.

A aba **Mundos** mostraria capa, nome, versão, jogadores presentes, estado do servidor, espaço e último backup. Detalhes técnicos ficam na administração, não no fluxo principal de jogar.

O site permitiria trocar plano, consultar pagamentos, cancelar, exportar dados e administrar o mesmo mundo. O launcher mostraria estado atualizado e acesso ao suporte, sem pedir uma segunda conta para o painel de hospedagem.

## 6. Base existente e componentes novos

O projeto já contém contas, sessões, perfis, tags e presença em Cloudflare Pages/Functions, D1 e Durable Objects. Isso é uma base reaproveitável; cobrança, quotas de nuvem e servidores administrados ainda precisam ser implementados.

Usar:

- **Site e launcher:** interfaces Luxmc para assinatura e administração.
- **Backend existente ampliado:** autenticação, planos, pagamentos, permissões e catálogo.
- **D1:** metadados comerciais, vínculos entre contas, estados dos trabalhos e referências dos servidores.
- **Fila de trabalhos:** criação, backup, restauração, atualização e cancelamento com execução recuperável.
- **Cloudflare R2 ou armazenamento compatível:** arquivos privados de backup, separados das máquinas dos jogos.
- **Pterodactyl estável e Wings em Linux:** controle de processos e servidores de jogo isolados.
- **Monitoramento externo:** disponibilidade, consumo, alertas e restauração após falhas.

O Pterodactyl é aberto e usa contêineres por servidor, com controles de CPU, memória e I/O. Ele exige administração e máquinas próprias; não fornece hospedagem gratuita. Usar a linha estável, não uma prévia, para o piloto comercial. [Pterodactyl](https://pterodactyl.io/), [instalação estável](https://docs.pterodactyl.io/v1/panel/getting-started).

Cloudflare cuidaria do site, autorização e metadados. O processo persistente do Minecraft e tarefas demoradas de instalação e restauração rodariam nas máquinas Linux. Não tentar executar o Minecraft em Pages ou Workers.

O cliente fala com a API Luxmc. Só o backend possui a chave administrativa do Pterodactyl. Credenciais de console ou arquivos, quando necessárias, devem ser temporárias, limitadas ao servidor autorizado e revogáveis.

## 7. Contas, tags e permissões

A compra pertence a um identificador estável da conta, nunca ao apelido, avatar ou e-mail digitado na tela. Contas Microsoft e contas Luxmc devem conseguir se vincular ao mesmo cadastro mediante comprovação de controle. O login Microsoft no launcher não substitui automaticamente essa vinculação comercial no site.

Separar três informações:

- **Papel administrativo:** dono do Luxmc, operador ou cliente.
- **Tags públicas manuais:** testador, amigo e outras atribuídas exclusivamente pelo dono.
- **Benefícios pagos verificados:** Plus, Pro e capacidades vigentes, derivados do pagamento confirmado.

Uma tag “Pro” escrita manualmente não pode liberar armazenamento nem servidores. O badge da assinatura verificada precisa ser distinguível de uma tag decorativa. O próprio dono pode usar o serviço, mas continua sujeito às quotas técnicas e aos registros de auditoria.

Ter usuários convidados com permissões específicas: jogar, ver console, reiniciar, gerenciar backups ou administrar arquivos. Convite para jogar não concede acesso à cobrança ou aos arquivos. Ao remover um convidado, revogar também as permissões e sessões correspondentes.

Registrar alterações de tags, concessões, bloqueios, restaurações e exclusões. Remoção de perfil social e encerramento de conta com assinatura são operações distintas; não apagar um servidor pago por uma mudança no perfil.

## 8. Pagamentos e estados comerciais

Mercado Pago é o candidato inicial para uma operação brasileira. A API oferece criação de assinaturas e cobranças recorrentes; métodos disponíveis e comportamento de recorrência precisam ser confirmados na modalidade e na conta de vendedor escolhidas. [Documentação de assinaturas](https://www.mercadopago.com.br/developers/pt/docs/subscriptions/overview).

Primeira entrega: cartão com recorrência validada em sandbox. Pix pode começar como compra de período pré-pago; só oferecer renovação automática por Pix após testar uma integração que realmente suporte esse fluxo. Não chamar pagamento manual de assinatura automática.

Fluxo obrigatório:

1. Backend cria uma intenção de compra para a conta autenticada e o preço cadastrado no servidor.
2. Checkout do provedor processa o pagamento; o Luxmc não armazena número de cartão.
3. Webhook tem autenticidade verificada conforme o produto escolhido.
4. Backend consulta o provedor e confere vendedor, valor, moeda, produto, referência e estado efetivo do pagamento.
5. O evento é registrado uma única vez; pagamentos repetidos não duplicam créditos ou servidores.
6. A assinatura e seus benefícios são atualizados com vigência e revisão.
7. Uma reconciliação periódica recupera notificações perdidas e evita benefícios indevidos após estorno.

Nunca liberar serviço apenas pelo retorno do navegador ou porque a autorização de recorrência foi criada. É preciso confirmação do pagamento do período correspondente. [Validação de notificações do Mercado Pago](https://www.mercadopago.com.br/developers/pt/docs/checkout-pro-preferences/payment-notifications).

Estados internos: aguardando pagamento, ativo, pagamento atrasado, cancelamento agendado, suspenso e encerrado. Reembolso e contestação têm registros próprios. Cancelamento interrompe a renovação e preserva o acesso ao período já pago, salvo situações justificadas nos termos.

Política proposta para atraso: aviso imediato, três dias de tolerância, suspensão do processo depois desse prazo e conservação dos dados por 30 dias, com exportação pelo painel. Após o prazo, avisar e aplicar a exclusão prevista. Essa política é uma proposta operacional que precisa de orçamento e revisão dos termos antes da venda.

Downgrade não apaga arquivos para encaixar a nova cota. A interface indica o excedente e oferece exportação ou ampliação. Excluir definitivamente exige confirmar o servidor correto e mostrar o que será removido, incluindo o destino dos backups.

## 9. Backups e proteção dos mundos

Backups locais só devem ler o mundo após ele ser salvo e fechado. Para servidor ativo, usar um mecanismo de salvamento consistente ou uma parada curta; copiar arquivos durante gravações pode produzir uma cópia inútil.

Enviar arquivos diretamente para armazenamento privado por autorização temporária. Suportar upload multipartes ou por blocos, retomada e validação de integridade, para não recomeçar um mundo grande após perder conexão. Links temporários expiram e só autorizam o objeto da conta correta. [URLs assinadas do R2](https://developers.cloudflare.com/r2/api/s3/presigned-urls/).

No começo, implementar snapshots consistentes completos e medir custo e duração. Acrescentar backups incrementais se os dados do piloto justificarem, com restauração completa testada. Nunca anunciar “backup feito” antes de upload, validação e registro final concluídos.

Restauração: criar cópia do estado atual, parar o servidor, restaurar em diretório temporário, verificar arquivos e iniciar. Se falhar, reverter sem sobrescrever a única cópia válida. Testar restauração em outra máquina, não apenas no mesmo disco.

Criptografar transporte e armazenamento; limitar acesso operacional aos backups. Não chamar isso de criptografia ponta a ponta sem implementar e validar gestão de chaves compatível com os backups automáticos.

## 10. Modpacks, versões e segurança

Começar com um catálogo pequeno aprovado em testes. Para modpack, buscar o pacote de servidor correspondente, versões exatas de loader, Java e dependências. O conjunto de mods do cliente não é automaticamente um servidor válido: alguns mods só funcionam no cliente.

Respeitar as permissões de distribuição e as regras dos provedores. Preferir download autorizado de cada origem e cache apenas quando permitido. Não redistribuir um ZIP com conteúdo sem direito de hospedá-lo.

Fixar a versão de cada mundo. Uma atualização nova do Minecraft ou do pack gera uma opção de atualizar com backup e teste; nunca atualizar automaticamente um mundo para uma versão potencialmente incompatível. Dar preferência à correspondência exata entre cliente e servidor.

Arquivos importados passam por limites de tamanho, quantidade e expansão, proteção contra caminhos fora da pasta e as verificações disponíveis do Shield. Contêineres devem ter imagens aprovadas, usuário sem privilégios, limites e nenhuma montagem do sistema anfitrião ou acesso ao socket Docker. Contêiner não elimina a necessidade de atualizar o kernel e o gerenciador.

Não oferecer scripts de inicialização arbitrários no primeiro lançamento. Separar segredos comerciais e chaves de assinatura da infraestrutura dos jogos. Exigir autenticação reforçada no painel do dono, limitar requisições, proteger operações de conta e testar autorização entre clientes.

A candidatura SignPath continua dependendo das condições do provedor. Manter o projeto e os componentes elegíveis sob licenças abertas; não transformar o launcher em uma edição proprietária com licença comercial dupla. A compatibilidade do novo desenho deve ser confirmada antes de alterar a política de assinatura. [Condições da SignPath](https://signpath.org/terms.html).

## 11. Custos e cálculo do preço

O custo completo inclui máquina de jogo, painel, disco, backups, tráfego ou retransmissão, monitoramento, domínio, suporte, taxas de pagamento, tributos, estornos e capacidade ociosa. Fazer a conta com a ocupação esperada do piloto, não supondo todos os lugares vendidos.

Uma fórmula inicial é:

`Preço mínimo = custo operacional por cliente / (1 − taxas proporcionais − margem desejada)`

Exemplo puramente ilustrativo: custo total de R$ 32 por cliente, taxas e tributos somados de 10% e margem de 25% resultam em R$ 49,23. Arredondar para R$ 49,90 só faz sentido se a cotação e o uso real confirmarem essas premissas. Taxas fixas precisam entrar no custo; 10% não é uma taxa real verificada do Mercado Pago nem uma alíquota tributária recomendada.

O R2 Standard publica US$ 0,015 por GB/mês, com cobranças adicionais por operações e sem tarifa de saída de dados. Assim, 5 GB completamente usados equivalem a US$ 0,075 de armazenamento bruto e 20 GB a US$ 0,30, antes de operações, arredondamento, câmbio, impostos e demais custos. A franquia gratuita é compartilhada pelo serviço, não por cliente. [Preços do R2](https://developers.cloudflare.com/r2/pricing/).

Como referência de mercado, o Modrinth anuncia 4 GB por US$ 10/mês com cobrança trimestral. Isso não é uma cotação para o Luxmc e disponibilidade e localização precisam ser verificadas na contratação. [Modrinth Servers](https://modrinth.com/servers).

Para o piloto, definir um teto interno inicial de gasto, por exemplo R$ 500/mês. Esse número é um limite de decisão proposto, não uma previsão de preço de fornecedor. Se o hardware adequado não couber nesse teto, reduzir o número de clientes do piloto ou rever o orçamento antes de vender.

Escolher provedor com CPU apropriada, armazenamento rápido, suporte a Docker, proteção de rede e localização testada a partir dos usuários brasileiros. Comparar custo total e latência; não contratar apenas a oferta com mais RAM. Capacidade liberada para venda deve respeitar o menor limite medido de CPU, RAM, disco, I/O e rede, com reserva operacional.

## 12. Dados e trabalhos novos

Estruturas propostas, ainda inexistentes:

- `account_links`: vínculos comprovados de login e perfil.
- `billing_customers`, `subscriptions`, `payments`, `billing_events`: cobrança e eventos idempotentes.
- `entitlements`: benefícios verificados, origem e prazo.
- `storage_usage`, `backups`: reservas de cota, objetos e restaurações.
- `hosted_servers`, `server_members`: servidor e permissões por conta.
- `provisioning_jobs`: execução, cancelamento, repetição e compensação de recursos.
- `audit_events`: ações administrativas importantes com retenção definida.

Reservar capacidade e cota de maneira atômica antes de criar recursos. Dois pedidos simultâneos não podem consumir a mesma vaga. Usar identificador idempotente por operação: uma repetição após timeout não cria outro servidor.

Cada trabalho tem etapas e cancelamento verdadeiro. O cancelamento interrompe downloads, instalação e verificações; remove arquivos temporários e libera a capacidade reservada. Não registra sucesso depois de cancelado. Se um servidor foi criado antes da falha, compensar a criação ou informar com precisão seu estado pendente.

Implementar alterações de backend no código-fonte tipado em `site-server/` e gerar `website/functions/` pelo processo existente. Na interface do launcher, usar as APIs tipadas e comandos Tauri, respeitando as convenções do projeto.

## 13. Operação 24 horas

Manter os processos ativos quando o servidor estiver vazio. Configurar início após reinicialização da máquina, recuperação após falha com limite de tentativas e alertas quando o problema exigir intervenção. Um pack incompatível não deve entrar em um ciclo infinito de reinício.

Ter cópias fora do nó de jogo, exportação acessível ao cliente e procedimento escrito para perda completa do nó. Uma segunda máquina pronta para restauração reduz a indisponibilidade; migração de Minecraft geralmente implica parar e reiniciar o processo, não transferência instantânea da partida.

Um único nó é aceitável em piloto identificado como beta, mas não oferece alta disponibilidade. “24 horas” descreve o serviço permanecer ligado; não significa ausência de manutenção ou falhas. Publicar uma meta de disponibilidade e regras de crédito somente após medir e orçar redundância.

Monitorar jogadores reais no servidor, tempo dos ticks, memória, disco, backups, filas, pagamentos e erros. Não usar a quantidade de amigos online no launcher como contagem de jogadores. Mostrar incidentes em uma página de status separada da máquina principal.

Antes da venda: preparar suporte, política de dados, cancelamento, reembolso, nota fiscal e enquadramento tributário, com revisão adequada à operação. Divulgar apenas horários e compromissos de atendimento que possamos cumprir.

## 14. Sequência de execução e critérios de conclusão

| Etapa | Entrega | Critério para avançar |
| --- | --- | --- |
| 1. Produto e custos | Benefícios finais, catálogo inicial, cotação e orçamento | Preço cobre custo no cenário de ocupação conservador; modelo revisado frente às regras aplicáveis |
| 2. Identidade e permissões | Conta comercial vinculada ao launcher; benefícios separados de tags | Não há troca de benefícios por apelido; testes de acesso entre clientes negados |
| 3. Pagamentos em teste | Checkout, webhook, renovação, cancelamento e reconciliação | Sem dupla cobrança/liberação, inclusive com eventos duplicados ou fora de ordem |
| 4. Plus e backups | Sincronização, 5 GB, upload e restauração | Mundo restaurado em outro dispositivo; quota e interrupção verificadas |
| 5. Hospedagem piloto | Pterodactyl estável, nó Linux, servidores e painel Luxmc | Servidor continua acessível com o PC do cliente desligado; reinício e restauração externa funcionam |
| 6. Compatibilidade e carga | Packs aprovados e medições de 2 a 30 jogadores | Capacidade publicada corresponde ao hardware e às cargas testadas |
| 7. Beta fechado | 5–10 clientes, capacidade limitada e monitorada | Pelo menos sete dias de observação, com testes de falha e custos por cliente registrados |
| 8. Lançamento gradual | Venda mensal com vagas reais e suporte preparado | Sem falhas críticas de cobrança, autorização, perda de mundos ou cancelamento |
| 9. Expansão | Pro completo, mais nós, catálogo e Bedrock | Demanda e margem sustentam expansão; restauração e capacidade novamente verificadas |

Os sete dias são um período mínimo de observação proposto, não prova definitiva de confiabilidade. Nenhuma data de lançamento está prometida neste plano; fornecedores, revisão e resultados do piloto podem alterar a sequência.

## 15. Verificação antes de qualquer lançamento

Cobrir compras aprovadas, recusadas, pendentes e estornadas; evento falsificado, repetido, atrasado e perdido; conta excluída; renovação e mudança de plano; compra simultânea e falta de capacidade.

Cobrir tentativa de acesso a arquivos e console de outra conta, sessão expirada, convite revogado, tag decorativa imitando plano, credencial vazada e excesso de cota. Fazer testes de carga e restaurar um mundo após perda simulada do nó.

Cobrir download interrompido, cancelamento em cada etapa, disco cheio, arquivo importado inválido, backup inconsistente, mod só de cliente, Java incorreto e atualização incompatível. Usar mundos de teste e cópias autorizadas, preservando os originais.

Executar `pnpm check`, testes de frontend e site, compilação e testes adequados do Rust após as alterações correspondentes. Repetir testes reais em Windows e Linux. Nenhum teste automatizado de interface substitui a validação entre duas redes ou comprova que todos os modpacks funcionam.

O primeiro objetivo comercial seria **Plus funcionando com backups restauráveis**, acompanhado de um **piloto pequeno de Luxmc Mundos**. Pro, 30 jogadores e Bedrock entram na oferta somente quando seus benefícios e a capacidade anunciada forem demonstráveis.
