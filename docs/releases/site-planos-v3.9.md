# Planos — Luxmc 3.9

A página mantém o nome autenticado, o plano e o vencimento. Pedidos e editores de aparência deixaram de ocupar a comparação de planos; a edição continua no launcher. O cartão do plano atual recebe um selo e não inicia uma compra duplicada.

Novos pedidos pausados desabilitam a escolha com uma indicação explícita. Pedidos já criados continuam acessíveis pelo botão “Ver pedido”, em uma janela que pode ser fechada. Se o recebimento for pausado enquanto a página está aberta, a resposta do servidor atualiza os botões e informa o motivo. Nenhuma configuração de recebimento foi ativada por esta alteração.

O cabeçalho mostra a foto pública sincronizada do launcher ou a cabeça recortada da skin publicada na mesma conta. O menu permite abrir o plano e sair do site. A sessão usa cookies HttpOnly e a confirmação de saída exige CSRF; IDs enviados pelo cliente não escolhem o destinatário de uma compra. Entrar com uma conta no site substitui o vínculo anterior do launcher e abrir um link autenticado do launcher substitui o vínculo anterior do navegador. As fotos usam uma rota autenticada, com cache privado, evitando enviar a imagem completa a cada consulta de sessão.

Verificação local: `pnpm check` sem erros ou avisos, `pnpm build:website`, 83 testes de API e interface, e fluxos de navegador com compra e comprovante simulados, aprovação, plano da própria conta, confirmação de identidade, pausa, fechamento da janela, cabeça da skin, foto, saída e layouts móvel e desktop. Os testes não fizeram pagamentos nem alteraram contas reais.

A publicação deste ajuste no site acompanha o instalador oficial 3.9 conferido pelo manifesto; o pipeline do launcher continua independente.
