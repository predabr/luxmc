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

## Continuação da validação
- Primeiro commit: 8e89c102a26abe94ae3be96cc6c1eb6213728137; branch codex/launcher-refinement-builds.
- Primeira CI: https://github.com/predabr/luxmc/actions/runs/38665330636. Linux passou nos 35 testes anteriores de navegador; o novo teste encontrou seleção de plataforma pelo navegador em vez do backend. Corrigido no código seguinte.
- A suíte local passou até catalog-navigation. social-presence falhou com o servidor Vite que havia recebido HMR durante a edição; repetir com servidor novo para evitar módulos duplicados por timestamp. Não marcar a suíte completa como aprovada antes disso.
- Limpeza de armazenamento agora usa estado global: preserva progresso ao sair da página e voltar, testado no navegador.
- Shield evita cópia temporária de JARs já conhecidos, mantendo hash do conteúdo e bloqueio de alteração com mesmo tamanho/mtime. Acrescentados tempos internos de Java/loader, nativos e aparência/Shield.
- Novos testes nativos ignorados por padrão têm execução explícita na CI com os JARs reais GeckoLib 3.1.40 e Schizo Cave Dweller 1.6.4 do provedor, verificados por tamanho e SHA-1 antes de gravar. Nunca executam os JARs.
- Novos arquivos deste bloco: src/lib/stores/storageActivity.svelte.ts e scripts/prepare-real-mod-fixtures.mjs.
- Os seis screenshots preexistentes foram copiados em Local/Luxmc-build/backups/refinement-existing-screenshots antes dos casos de catálogo e restaurados depois; capturas novas em requests/refinement-catalog-captures. Não incluir screenshots antigos nem mudanças vendor no commit.
