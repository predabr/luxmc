# Pedidos, comprovantes e conferência Pix — 25/10/2026

O recebimento atual usa uma chave Pix do Nubank, sem API bancária configurada. O envio de um comprovante apenas encaminha o pedido para conferência manual. O plano continua bloqueado até a confirmação do dono, com valor recebido, identificador Pix e crédito conferido. A imagem enviada pelo comprador não é prova de crédito.

## Correções

- Consulta autenticada de revisão a cada cinco segundos, pausada com a página oculta e com intervalo maior após falha. A revisão do comprador considera somente seus pedidos; a revisão geral exige sessão do dono. Ambas respeitam autenticação, origem, CSRF e limites de requisições.
- Atualização da lista somente quando a revisão muda. Cartões inalterados conservam campos preenchidos, foco, arquivo selecionado e comprovante aberto. Formulários de atribuição de plano e segurança não são reconstruídos pela atualização em segundo plano.
- Comprovantes carregam separadamente da fila, com limite de tempo, mensagem de falha e nova tentativa. URLs de arquivos em memória são revogadas ao fechar, trocar ou sair. O acesso continua privado.
- Rejeição exige motivo e mostra falhas no formulário da decisão. Uma decisão aplicada atualiza a fila e os contadores. Versão do pedido e hash do comprovante continuam impedindo a revisão de um arquivo substituído.
- O envio continua sem ativar assinatura; a rejeição não concede plano nem entra no total aprovado. A mesma transação Pix não pode aprovar dois pedidos.

## Verificação

`pnpm check`: zero erros e avisos. `pnpm test:website`: 135 verificações aprovadas. Os novos casos cobrem privacidade das revisões, mudanças de status, envio sem concessão de plano, atualização com comprovante aberto, preservação de campos e foco, falha e nova tentativa do comprovante e rejeição com motivo.

Os testes usam bancos e contas de exemplo. Nenhum pedido real foi aprovado ou rejeitado e nenhum pagamento foi efetuado. Confirmar automaticamente o crédito recebido exige uma integração bancária ou de provedor de pagamentos ainda não configurada.
