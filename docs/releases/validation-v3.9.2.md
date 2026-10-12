# Verificação do Luxmc 3.9.2

- Svelte e TypeScript: verificação sem erros ou avisos e 185 testes unitários aprovados.
- Interface: 25 cenários de navegador aprovados, cobrindo início e falha do Minecraft, atualização ao trocar de aba, 55 registros de changelog, capas, importação e renderização de skins, configurações responsivas, temas, idiomas, filas, cancelamento, autorização do dono, salas e presença.
- Backend Windows: cargo check e 236 testes Rust aprovados. Sete integrações externas ficam fora da rotina; o teste de download real do Fabulously Optimized foi habilitado separadamente e aprovado com tamanho e hash conferidos.
- Site: 85 testes aprovados, incluindo sessão, vínculo de compras à conta, autorização do dono, comprovantes privados, QR Pix e aprovação manual.
- Distribuição: 15 testes de manifestos, assinaturas e limites de profundidade aprovados. A auditoria continua identificando a versão nominal de braces; a correção local aplicada pelo pnpm foi exercitada pelos testes de profundidade.
- Descoberta LAN: probes com bytecode real de Minecraft 1.8.9, 1.20.1, Forge 1.20.1, 1.21.1 e 26.3 passaram com Java 17 e 25. O agente completo passou com Java 8, 17, 21 e 25 e encerrou corretamente.

A prévia e as capturas da interface usam dados de demonstração em navegador de teste. A validação de descoberta LAN e transporte não substitui uma partida em dois PCs no One Terror Block 1.20.1. Os modpacks dependem de mods compatíveis, do loader, de Java e dos requisitos da máquina; não existe garantia de funcionamento de todos os pacotes de terceiros.

Os executáveis Windows ainda não possuem assinatura Authenticode de editor confiável. As assinaturas Ed25519 confirmam a origem do manifesto e a integridade dos downloads; os bloqueios de reputação ou do Controle Inteligente continuam dependendo da política do Windows. Nenhuma proteção do computador foi desativada nesta revisão.
