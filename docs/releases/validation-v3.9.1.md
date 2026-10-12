# Verificação da 3.9.1 — Do Seu Jeito

Fonte compilada: 8c3e7344ffaaad5be24e30dace3c5206fc5b4bbd.

- Frontend: svelte-check sem erros ou avisos, 161 testes unitários e compilação de produção concluídos.
- Interface: 20 testes de navegador passaram na publicação, incluindo provedores, loaders, versão, categorias, busca, respostas antigas, paginação, teclado, temas, layout responsivo e movimento reduzido. A vitrine foi removida e os filtros ficam à direita.
- Site: 85 testes passaram. O checkout usa QR Pix com valor e referência por pedido, confirmação manual no banco e recibos privados após aprovação.
- O CI completo passou no Windows e no Linux. No Linux, 226 testes unitários Rust passaram, sete foram ignorados pela rotina e os testes de integração habilitados passaram.
- Windows: compilação, auditoria do instalador, recuperação de atualização interrompida e ciclo de instalação, reinstalação e desinstalação passaram no runner. Linux e macOS também concluíram os pacotes.
- Os sete instaladores foram conferidos antes da publicação. Todos os 11 arquivos públicos responderam com HTTP 200; os manifestos Ed25519 foram verificados com a chave pública oficial.
- O site oferece a mesma versão 3.9.1. Seu instalador Windows foi baixado e conferido: SHA-256 516ef1b8a340f4ee4e1971564de9d9370075b360d94ae0f6cc7867ff7b270c39.

[Publicação e verificações](https://github.com/predabr/luxmc/actions/runs/38201476978) · [CI completo](https://github.com/predabr/luxmc/actions/runs/38201428195)

As capturas do catálogo usam dados de demonstração em navegador de teste. O launcher instalado não foi iniciado nesta revisão. A descoberta LAN foi exercitada com os probes Java e Minecraft descritos nas notas da versão; isso não substitui uma partida em dois PCs no One Terror Block 1.20.1. Cartão e confirmação bancária automática continuam dependendo de um provedor aprovado. O instalador Windows ainda não tem assinatura Authenticode; a assinatura dos manifestos verifica integridade de distribuição.
