# Personalizador, introdução e site — revisão de recuperação

## Personalizador

A seleção não aplicada era substituída pela aparência da conta ao remontar a página. O rascunho agora é preservado em sessionStorage por conta, sem aplicar alterações automaticamente. Seleções assíncronas canceladas não podem sobrescrever a escolha mais recente.

A prévia usa renderização direta no framebuffer principal, sem o pós-processamento com alvo de profundidade flutuante da biblioteca. O buffer não é preservado entre quadros; o desenho tem limite de 30 FPS e DPR de 1,25. O descarte dos recursos ocorre uma vez por geometria/material, sem forçar perda de contexto ao navegar. Os controles internos da biblioteca são desconectados para não disputar os eventos do mouse.

Perda de contexto mostra uma prévia alternativa da seleção. Após restauração, o desenho é retomado; há um botão para reiniciar a prévia. Falhas deixam de ser silenciadas no loop de animação.

## Evidência e limites

Foi usado o PNG local indicado pelo nome no print do usuário. A falha espontânea original não foi reproduzida no primeiro teste isolado; portanto não há evidência para atribuir todos os quadros pretos a uma única causa de driver. As mudanças eliminam problemas identificados no ciclo de vida e tornam a recuperação explícita.

O WebKitGTK foi executado com as variáveis de composição usadas pelo launcher. Passaram 36 verificações: importação com capa, arraste, perda e restauração forçada de WebGL, rascunho após navegação e oito ciclos de seleção/saída/retorno. Cada ciclo leu os pixels do framebuffer e verificou transparência, presença do modelo e ausência de erro GL. Os comandos de conta/importação são simulados nesse teste; a renderização é real. Não foi alterada a aparência Microsoft da conta do usuário.

Executável final de produção compilado em 7 min 47 s com `pnpm tauri build --no-bundle`.

Svelte/TypeScript: zero erros e avisos. Frontend: 59 testes aprovados. Contas/site: 15 aprovados. Uma execução foi interrompida pela atualização do servidor de desenvolvimento durante a compilação; a execução final foi feita após a compilação do frontend e passou.

## Introdução

Nova composição com paisagem existente do Minecraft, marca em camadas, título, transição curta de 2,4 segundos e saída suave. Mantidos o botão de pular, atalhos e preferência por movimento reduzido. Composição conferida em prévia isolada no navegador.

## Site

Página inicial reorganizada em torno do produto, com título central, imagem maior, botões e cartões consistentes, tipografia e espaçamento revisados. Layout conferido em desktop e em 390×844. Portal, downloads, catálogo e demais seções preservados. Textos excessivos ou sem evidência foram simplificados.

Publicado em https://afd5f29b.luxmc-r92.pages.dev, também servido pelo endereço https://luxmc-r92.pages.dev. Nenhum push para GitHub.

## Referências consultadas

- https://modrinth.com/news/article/skins-now-in-modrinth-app/
- https://github.com/modrinth/code/blob/main/packages/ui/src/composables/skin-rendering/use-skin-preview-scene.ts
- https://github.com/modrinth/code/blob/main/packages/ui/src/components/skin/SkinPreviewRendererImpl.vue
- https://skmedix.pl/

Foi interpretado “ModRich” como Modrinth a partir da descrição. O código foi consultado como referência de arquitetura, sem copiar implementação ou identidade visual.

Logs: `/home/pedro/.cache/luxmc-validation/2026-09-26-personalizer/`.
