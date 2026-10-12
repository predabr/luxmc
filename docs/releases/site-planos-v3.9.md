# Planos — Luxmc 3.9

A página mantém o nome autenticado, o plano e o vencimento. Pedidos e editores de aparência deixaram de ocupar a comparação de planos; a edição continua no launcher. O cartão do plano atual recebe um selo e não inicia uma compra duplicada.

Novos pedidos pausados desabilitam a escolha com uma indicação explícita. Pedidos já criados continuam acessíveis pelo botão “Ver pedido”, em uma janela que pode ser fechada. Se o recebimento for pausado enquanto a página está aberta, a resposta do servidor atualiza os botões e informa o motivo. Nenhuma configuração de recebimento foi ativada por esta alteração.

Verificação local: `pnpm check` sem erros ou avisos, `pnpm build:website`, 78 testes de API e interface, e fluxo de navegador com compra e comprovante simulados, aprovação, plano da própria conta, confirmação de identidade, pausa, fechamento da janela e layout móvel. Os testes não fizeram pagamentos nem alteraram contas reais.

A publicação deste ajuste no site acompanha o instalador oficial 3.9 conferido pelo manifesto; o pipeline do launcher continua independente.
