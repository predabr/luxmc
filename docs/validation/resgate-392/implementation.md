# LUXMC-RESGATE-392 — candidato de correção

## Evidência e mudança

O executável público 3.9.2 foi identificado pelo SHA-256 `b17782ec396c1ab834fc2f3aafc160516052e57fe60a6f9dcc580da4328c54cc`. O clique em Jogar na Vanilla 26.3 reproduziu a exceção `c00000fd` em `luxmc.exe`, offset `1b01957`, em 13/10/2026. A instrução correspondente pertence à sondagem de pilha. O PE reserva 1.048.576 bytes de pilha; foram encontrados prólogos com alocações individuais de até 1.251.248 bytes. A captura adicional da pilha foi interrompida pelo usuário, portanto a atribuição simbólica do offset ao chamador continua pendente.

O comando Tauri agora constrói somente uma tarefa pequena, desloca a preparação para o runtime do backend e usa futures alocados no heap. A seleção de loaders retorna um future boxed por implementação, evitando incluir toda a maior cadeia de preparação no future de cada chamador. O limite de pilha das threads não foi aumentado como correção.

A sessão de lançamento tem UUID próprio, etapa, PID e horário de criação do processo. A confirmação no Windows exige duas observações consecutivas de uma janela de renderização GLFW/LWJGL. Um processo vivo ou uma caixa de erro do Java não autoriza sucesso nem ocultação. A espera mantém o monitor de saída e permite cancelar, sem timeout que mate modpacks demorados.

O progresso e o diagnóstico ficam no layout global, persistem ao navegar e oferecem logs, cancelamento e cópia. Eventos de outra sessão não podem encerrar a sessão atual na interface. O backend mantém registros sanitizados e escritos atomicamente por sessão, além do último diagnóstico. O cancelamento exige o UUID ativo; o encerramento exige o processo pertencente à sessão. No Windows, o handle validado é mantido durante o encerramento, evitando reutilização do PID.

## Verificação

- Backup SQLite consistente e configurações preservados antes da reprodução; integridade do banco: `ok`.
- `pnpm check`: zero erros e avisos nas verificações realizadas.
- `cargo check --release --locked --lib`: aprovado, com avisos preexistentes de código não utilizado.
- Suite frontend: 186 testes aprovados antes dos dois novos testes de API; os 11 testes de lançamento/API atualizados também passaram.
- Build frontend de produção: aprovada.
- Regressão de navegador da sessão: navegação durante abertura, cancelamento por UUID, diagnóstico persistente e descarte de evento antigo aprovados.
- Novos testes Rust cobrem construção/poll inicial em pilha de 1 MB, janela estável, processo sem janela, falha precoce, monitor perdido, cancelamento e registros sanitizados. A execução local foi bloqueada pela política do Windows (`os error 4551`); compilação/CI e resultados devem ser registrados separadamente.

## Limites de entrega

A implementação está em validação. A pedido do usuário, a continuação ocorre sem controle de janelas, mouse ou teclado e sem abrir Minecraft no PC. Menu e mundo reais no executável empacotado, matriz de modpacks/loaders, outro Windows e LAN em dois PCs permanecem pendentes. Testes com fixtures não comprovam esses resultados.

A versão pública 3.9.2 é preservada. Este candidato não recebe um novo número nem é promovido a release sem a validação e a publicação solicitadas. Não há certificado Authenticode confiável disponível; esta correção não garante que o Controle Inteligente de Aplicativos aceitará um binário novo. As configurações de proteção do Windows não foram alteradas.
