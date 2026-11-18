# Aplicar aparência: reaplicação persistente

O botão Aplicar skin e capa permanece disponível mesmo quando a aparência selecionada já está aplicada. Contas Microsoft mantêm a ação única de aplicar e sincronizar. A ausência de mudanças continua sendo usada para o texto de estado e a persistência do rascunho, sem bloquear a ação.

A ação fica bloqueada enquanto a página hidrata, enquanto salva ou quando não há conta ou textura selecionada. O estado de carregamento bloqueia cliques concorrentes. Selecionar uma skin, inclusive pelo NameMC, continua sem aplicar automaticamente.

## Verificação

- `pnpm check`: 0 erros e 0 avisos.
- `tests/skin-apply-persistent.browser.cjs`: passou para contas local, Luxmc e Microsoft com seleção já aplicada; reaplicação, navegação para Instâncias e volta, reload, bloqueio de cliques duplicados e retry de sincronização Microsoft.
- `tests/appearance-microsoft.browser.cjs`: passou; salvamento local preservado após falha Microsoft, retry, capa restaurada após reload.
- `tests/namemc-picker.browser.cjs`: passou; importação pelo seletor e link, deduplicação, limpeza de listeners e recuperação de erro sem aplicação automática.

Os testes browser usam a interface real do launcher no Edge headless com as respostas dos comandos nativos simuladas. A alteração não modifica os handlers nativos de aplicação ou autenticação.
