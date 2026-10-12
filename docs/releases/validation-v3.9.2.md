# Verificação do Luxmc 3.9.2

- Svelte e TypeScript: verificação sem erros ou avisos e 185 testes unitários aprovados.
- Interface: 25 cenários de navegador aprovados, cobrindo início e falha do Minecraft, atualização ao trocar de aba, 55 registros de changelog, capas, importação e renderização de skins, configurações responsivas, temas, idiomas, filas, cancelamento, autorização do dono, salas e presença.
- Backend Windows: cargo check e 236 testes Rust aprovados. Sete integrações externas ficam fora da rotina; três foram habilitadas separadamente e aprovadas: download real do Fabulously Optimized com tamanho e hash conferidos, transferência por relay sem transporte direto por IP e conexão à sala por convite curto publicado.
- Site: 85 testes aprovados, incluindo sessão, vínculo de compras à conta, autorização do dono, comprovantes privados, QR Pix e aprovação manual.
- Distribuição: 15 testes de manifestos, assinaturas e limites de profundidade aprovados. A auditoria continua identificando a versão nominal de braces; a correção local aplicada pelo pnpm foi exercitada pelos testes de profundidade.
- Descoberta LAN: probes com bytecode real de Minecraft 1.8.9, 1.20.1, Forge 1.20.1, 1.21.1 e 26.3 passaram com Java 17 e 25. O agente completo passou com Java 8, 17, 21 e 25 e encerrou corretamente.

A [verificação do commit da release no GitHub](https://github.com/predabr/luxmc/actions/runs/38318523610) terminou com sucesso nas rotinas de Windows e Linux. Os logs locais desta rodada foram preservados junto aos resultados de validação.

O instalador Windows e o executável principal passaram na varredura do Microsoft Defender, com assinaturas de detecção 1.459.690.0. A instalação, a reinstalação da mesma versão e a desinstalação retornaram código zero na VM; os dados protegidos foram preservados e os atalhos e o protocolo apontam diretamente para o launcher. A remoção completa dos dados do usuário não foi exercitada nessa rotina. Os [resultados do Defender](../validation/v3.9.2/windows-release-audit.json) e do [ciclo de instalação](../validation/v3.9.2/installer-lifecycle.json) identificam o mesmo instalador pelo SHA-256.

A [rotina de publicação](https://github.com/predabr/luxmc/actions/runs/38318528690) terminou com sucesso e publicou a [release 3.9.2](https://github.com/predabr/luxmc/releases/tag/v3.9.2) em 12/10/2026. Os sete instaladores e os quatro arquivos de metadados retornaram HTTP 200; as assinaturas Ed25519 foram verificadas. O site anuncia 3.9.2 e os links Linux e macOS apontam para os novos arquivos. O download Windows servido pelo site foi baixado e comparado com o manifesto oficial: 21.289.581 bytes, SHA-256 `e651194377fee9c1c97c3c4814979d16a6e8261ebf8ca71cc4049ef1c8ed69ee`. O [relatório público](../validation/v3.9.2/public-release.json) registra os arquivos e as rotas conferidas.

A prévia e as capturas da interface usam dados de demonstração em navegador de teste. A validação de descoberta LAN e transporte não substitui uma partida em dois PCs no One Terror Block 1.20.1. Os modpacks dependem de mods compatíveis, do loader, de Java e dos requisitos da máquina; não existe garantia de funcionamento de todos os pacotes de terceiros.

Os executáveis Windows ainda não possuem assinatura Authenticode de editor confiável. As assinaturas Ed25519 confirmam a origem do manifesto e a integridade dos downloads; os bloqueios de reputação ou do Controle Inteligente continuam dependendo da política do Windows. Nenhuma proteção do computador foi desativada nesta revisão.
