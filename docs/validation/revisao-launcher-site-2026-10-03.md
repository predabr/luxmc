# Revisão do launcher e site — 3 de outubro de 2026

## Escopo e preservação

Revisão funcional do frontend Svelte, integração de catálogo, site existente em `website/`, testes automatizados e leitura dos caminhos nativos de busca e instalação. As alterações locais anteriores foram preservadas. Esta revisão não certifica ausência de todos os bugs do produto.

## Correções e melhorias aplicadas

- A instalação de modpacks respeita a versão escolhida na aba de versões e o filtro de Minecraft. Arquivos sem extensão de pacote não são mais aceitos como fallback.
- O catálogo obtém versões de Minecraft pela API tipada do launcher. Mantém a versão selecionada e as versões das instâncias quando a consulta falha.
- Loaders usam valores consistentes em minúsculas; o filtro Kitchen Sink usa o slug correto.
- A seleção de instância mostra incompatibilidades de Minecraft e loader segundo os metadados do projeto, bloqueia mods em Vanilla e valida novamente antes da instalação.
- A instalação não altera o loader da instância apenas na interface.
- A indicação de conteúdo instalado é associada à instância de destino durante a sessão.
- Filtros ativos podem ser removidos individualmente; a pesquisa pode ser limpa; buscas e paginação retornam ao início da área de resultados.
- O selo genérico “Verificado” foi removido dos detalhes: não havia evidência correspondente para cada projeto.
- O site ganhou superfícies, contraste, cartões e navegação visual consistentes, com inspiração na organização do Modrinth e preservação da marca azul do Luxmc.
- O catálogo do site tem filtros de loader e Minecraft, ordenação, 12 resultados por página, contagem fornecida pela API e paginação.
- Falhas de rede no site exibem erro e permitem tentar novamente, sem apresentar resultados estáticos como resposta da pesquisa.
- Respostas antigas de busca são descartadas. A lista de versões carrega sem bloquear a busca inicial.
- Novos controles do site têm traduções em português, inglês e espanhol.
- O menu mobile oculta seus controles da navegação por teclado quando fechado, mantém o foco ao navegar e fecha com Escape. O botão de download volta a ter texto em telas pequenas.
- O consentimento de cookies continua funcionando quando o armazenamento do navegador está bloqueado. O fallback de imagem evita tentativas infinitas.
- Textos que anunciavam licença MIT foram alinhados ao arquivo LICENSE existente, que contém licença própria e gratuidade para uso pessoal.
- A versão do pnpm foi fixada em 10.30.3 no projeto e na CI. Dependências foram reinstaladas com o lockfile existente, sem atualização do lockfile.
- Seletores antigos dos testes de catálogo, Jukebox e laboratório foram ajustados à interface atual, preservando as verificações funcionais.

## Validação

| Verificação | Resultado |
| --- | --- |
| `pnpm check` | 0 erros e 0 avisos |
| `pnpm test:run` | 79 testes, 12 arquivos, aprovados |
| `pnpm test:website` | 19 testes aprovados |
| `pnpm build` | Build estático concluído |
| Catálogo do launcher em Chromium/Edge | Busca antiga descartada, paginação, rolagem, teclado, cancelamento e escolha de versão aprovados |
| Conteúdo de instância | 1.000 mods, acesso ao último item, filtros, atualizações congeladas, configurações e ferramentas testados em 1440 e 960 pixels |
| Criação de instância | Seleção de NeoForge estável e beta aprovada |
| Laboratório | Diagnóstico reversível, cápsulas e importação de benchmark aprovados |
| Links do launcher | Fluxos de protocolo aprovados |
| Skins | Importação, persistência, renderização, capas e recuperação de erro aprovados |
| Site em Chromium/Edge | Filtros, paginação, recuperação de rede, foco e layouts em 1440, 1024, 768 e 390 pixels aprovados |
| Páginas secundárias | Conta, skins, privacidade e termos abrem sem erro JavaScript e sem overflow horizontal em 390 pixels |
| API Modrinth real | Endpoints de versões e pesquisa filtrada responderam HTTP 200 |
| `git diff --check` | Sem erros de whitespace |
| `cargo check` | Não executou: `cargo` não está instalado/disponível neste ambiente |

Os testes de interface do launcher usam respostas nativas simuladas. O teste do site simula a API para validar erros, paginação e recuperação de forma reproduzível. As capturas do catálogo mostram dados de teste; a prévia local usa a API real. As páginas secundárias foram verificadas com recursos externos indisponíveis, sem certificar o serviço de contas ou o renderizador 3D remoto.

O build e a bateria final de navegador foram executados sequencialmente: gerar `.svelte-kit` durante os testes no servidor de desenvolvimento pode provocar recarregamentos e invalidar uma sessão de teste.

## Pendências nativas encontradas por leitura estática

Estas observações permanecem sem correção nesta entrega e precisam de testes Rust e execução nativa antes de liberação:

1. Em `src-tauri/src/commands/mods.rs`, buscas que falham ou expiram viram listas vazias e podem ser armazenadas no cache. A interface pode interpretar indisponibilidade como ausência de resultados.
2. No modo de fontes combinadas, o mesmo offset é usado em cada provedor e a lista combinada é truncada. Projetos descartados de uma página podem não aparecer na próxima. É necessário definir e testar a paginação conjunta.
3. A compatibilidade geral de um projeto não garante a compatibilidade de cada arquivo. A consulta e seleção nativa de versões de mods precisa validar o loader da instância, especialmente em projetos que publicam Fabric e Forge separados.

Não foram validados compilação Rust, bundles Tauri/Electron, login Microsoft real, lançamento de Minecraft, instalação real de modpacks, multiplayer/P2P ou serviços Cloudflare com dados persistidos. O executável `Luxmc.exe` não foi recompilado nesta revisão. O site não foi publicado.

## Artefatos

- Prévia local: `http://127.0.0.1:4174/`
- Capturas: `docs/visual/2026-10-03-discovery/`
- Teste do site no navegador: `tests/discovery.browser.cjs`
- Novos testes de catálogo: `tests/catalog.test.mjs`
- Testes de compatibilidade: `src/lib/utils/catalogCompatibility.test.ts`
