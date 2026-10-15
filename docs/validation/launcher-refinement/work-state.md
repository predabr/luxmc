# Luxmc — refinamento da 3.9.5

## Escopo
Contagem e limpeza de armazenamento, conflitos de mods, transparência com wallpaper, animações configuráveis, ícones progressivos, avisos de atualização de modpacks, remoção acessível, GIFs Plus/Pro, atualizador por compilação e propostas de logo sem aplicação.

## Validação local
- pnpm check: zero erros e avisos.
- Vitest: 215 testes passaram.
- Site: 90 testes passaram; saída gerada de membership atualizada.
- Manifestos e assinaturas: 13 testes passaram.
- Navegador: mil mods em telas 1440/960; histórico de 55 versões; progresso de atualização entre páginas; GIFs Free/Plus/Pro; limpeza com execução suspensa até conclusão e recontagem forçada; mesma versão com compilação diferente, sem aviso repetido após instalação; painéis respeitam opacidade do wallpaper.
- cargo check debug falhou ao carregar dependências existentes; tentativa com alvo separado foi bloqueada por Controle de Aplicativo (erro 4551) no build-script do crossbeam-utils. cargo check --release confirmou o mesmo bloqueio em zerofrom_derive. Não desativamos a proteção. Rustfmt validou a sintaxe. Ainda falta compilação e testes nativos em ambiente de CI.

## Entrega
Nenhuma nova versão publicada ou instalada neste refinamento. Versão pública continua a 3.9.5 original, commit 772931fed5c7a7f5ae9d7e893620a66f29464dbb. Propostas de logo em docs/design/logo-proposals, fora dos assets do produto.

## Próximas verificações
Concluir suíte de navegador e CI Windows/Linux; analisar divergências e corrigir antes de preparar um instalador. Testar os JARs reais do Terrorcraft em cópia isolada, sem modificar as instâncias do usuário. Não prometer início instantâneo nem compatibilidade universal sem medir cada pacote.
