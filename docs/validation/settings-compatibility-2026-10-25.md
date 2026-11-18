# Configurações e compatibilidade — 25/10/2026

Continuação local da revisão de capas e navegação. Publicação do launcher e do site continua cancelada.

## Alterações

- Ações da instância em um rodapé estático após o corpo do editor. O editor usa fluxo vertical com espaçamento próprio; a barra não usa posição sticky/fixed, offsets nem transformações para acompanhar a rolagem. Cancelar também fica bloqueado durante o salvamento.
- Preservadas as categorias das configurações, espaçamento ampliado e animações curtas de seleção/entrada. O rodapé tem fundo legível com wallpaper.
- Arquivos de versões incluem Minecraft, loaders, canal de lançamento e data quando declarados pelo provedor. A versão específica de um modpack mantém os loaders e a versão do Minecraft ao entrar na fila de instalação.
- Detalhes unem metadados do projeto e dos arquivos publicados, sem perder loaders ao substituir os dados da busca. Uma relação por loader mostra as versões efetivamente presentes nos arquivos carregados.
- Histórico do CurseForge com paginação, recuperação de falha e estados de carregamento. As entradas mostram Minecraft, loader e canal do arquivo. Layout responsivo e nomes de arquivos sem transbordamento.
- Cache de detalhes válido reutilizado por cinco minutos, evitando nova consulta ao voltar ao mesmo projeto. A consulta de versões Modrinth dispensa changelogs; o histórico de mudanças continua em sua própria aba.
- Cartões do site mostram todos os loaders declarados nos resultados e calculam a versão de Minecraft por ordenação numérica, sem presumir a ordem da API.
- Textos novos em português, inglês e espanhol. Ports mantidos em projetos distintos não recebem compatibilidade presumida.
- Ideia de outros jogos registrada em docs/plans/multigame-futuro.md, sem implementação.

## Validação e entrega

Verificações de compilação e tipos apenas; sem navegar no launcher ou iniciar Minecraft, conforme preferência do usuário. O comportamento visual e a instalação de modpacks serão experimentados pelo usuário.

Build local pretendida: 4.0.0 / 40133561066.1. Somente registrar como instalada depois de gerar e conferir o instalador, comparar os dados preservados e abrir o executável atualizado.
