# Reformulação do site LuxMC — 4 de outubro de 2026

> Registro histórico: a direção visual descrita abaixo foi substituída pela revisão em [website-visual-revision-2026-10-04.md](website-visual-revision-2026-10-04.md).

A reformulação foi implementada no site estático em `website/`, mantendo Cloudflare Pages Functions, as integrações e os caminhos públicos existentes. Prévia: http://127.0.0.1:8790. Para iniciar novamente: `pnpm preview:website`. O comando usa Node 24 e SQLite em memória; as contas de teste não são persistidas nem enviadas ao banco de produção.

## Auditoria e decisões

O site tinha seis camadas de estilos, scripts de efeitos sobrepostos, imagens decorativas de cenários Minecraft e anúncios que desviavam o fluxo de download. A nova implementação usa um único sistema de estilos, fontes locais, capturas reais do launcher e uma composição WebGL procedural. Os arquivos antigos de estilos, efeitos e anúncios foram removidos. Assets que também aparecem em outros testes do projeto foram preservados, sem carregá-los como decoração no novo site.

A identidade usa fundo escuro, azul lapis, metal claro e a geometria do símbolo LuxMC. A hero reúne o título, o CTA detectado por plataforma, a versão consultada no GitHub e um portal entre blocos instanciados. O produto aparece em três etapas, com capturas reais de instâncias, catálogo e personalizador. O catálogo, o diagnóstico demonstrativo, as skins, os amigos e a instalação seguem essa mesma linguagem.

Foram reformuladas Home, Skin Studio, Conta, login, cadastro, recuperação, painel autenticado, Privacidade e Termos. Foi adicionada uma página 404. O header mobile tem controle de foco, fechamento por Escape e conteúdo de fundo inerte enquanto o menu está aberto. Ampliação de screenshots e apoio PIX usam dialogs nativos.

As afirmações técnicas foram confrontadas com o código: o site consulta Modrinth; CurseForge está integrado no launcher. A licença do projeto é própria, não MIT. O túnel multiplayer pode usar conexão direta ou relay. CrashDoctor analisa logs e recomenda ações; o terminal do site é identificado como exemplo. Mod Shield verifica hashes conhecidos e estruturas suspeitas em JARs, sem prometer detectar todos os conflitos ou substituir um antivírus.

## Integrações preservadas

Os arquivos de `website/functions/`, `website/lib/` e `website/migrations/` têm os mesmos hashes da cópia realizada antes desta reformulação. Não houve alteração de dados ou configuração de produção. Downloads, autenticação HttpOnly, recuperação, preferências, amizades, protocolo `luxmc://` e os instaladores permanecem nos caminhos existentes.

A cópia anterior está em `%TEMP%/luxmc-website-before-reform-20261004/website`. O repositório já tinha muitas alterações do usuário antes do trabalho; o diff contra Git também inclui essas alterações anteriores. A lista desta entrega está em `website-reform-delivery.json`.

## Validação executada

| Verificação | Resultado |
| --- | --- |
| `pnpm build:website` | Aprovado |
| `pnpm test:website` | 26 testes aprovados |
| `pnpm check` | Zero erros e zero avisos |
| `pnpm test:run` | 119 testes aprovados em 19 arquivos |
| `pnpm build` | Aprovado |
| Recursos locais das seis páginas | Referências verificadas por teste |
| API e banco do site | Testes de conta, recuperação, CSRF, social, releases e protocolo aprovados |

Não existe script de lint no projeto. Neste ambiente, os comandos pnpm foram executados com `--config.verify-deps-before-run=false` para usar a instalação existente, pois o pnpm disponível difere da versão declarada no projeto. Não foram editados arquivos Rust nesta reformulação.

O navegador foi usado para revisar todas as páginas e os principais fluxos. Foram exercitados pesquisa Sodium, loader Fabric, versão 1.21.1, resultado vazio, paginação real, idiomas PT/EN/ES, menu mobile, ampliação de captura, FAQ, consulta real de Hypixel, copiar comando Linux, dialog PIX, validação de conta, mostrar/ocultar senha, login/logout local e abas do painel. No Studio foram testados Steve/Alex, modelos, rotação, teclado, importação PNG e rejeição de dimensões inválidas. O PNG baixado foi comparado byte a byte com o arquivo importado e coincidiu.

| Largura | Home, Studio, Conta, painel, Privacidade, Termos e 404 |
| --- | --- |
| 320 | Sem overflow horizontal |
| 375 | Sem overflow horizontal |
| 430 | Sem overflow horizontal |
| 768 | Sem overflow horizontal |
| 1024 | Sem overflow horizontal |
| 1440 | Sem overflow horizontal |
| 1920 | Sem overflow horizontal |

Foram registradas 49 combinações de página e largura. Uma segunda medição da Home confirmou também ausência de títulos e controles cortados nas sete larguras. Os registros estão em `docs/visual/2026-10-04-website-reform/responsive.json`.

As rotas Windows, AppImage, Debian, RPM, Arch e macOS retornaram HTTP 302 para assets do release oficial v2.0.2. Todos os seis destinos responderam HTTP 200 a HEAD. Isso verifica a disponibilidade dos arquivos; não representa instalação ou execução desses pacotes.

## Performance e movimento

O bootstrap da Home tem aproximadamente 5,2 KiB sem compressão e 2 KiB com gzip. Three.js é compartilhado entre a hero e as skins, evitando carregar duas versões. O módulo comum tem aproximadamente 469,5 KiB sem compressão e 119,3 KiB com gzip. Os tamanhos completos estão no JSON da entrega.

Hero e skins iniciam quando entram na área de visualização. A cena usa instancing, DPR máximo de 1 em dispositivos menores e 1,5 nos demais, menos blocos em dispositivos com poucos núcleos ou economia de dados e limite de atualização de aproximadamente 30/50 FPS. Esses números são limites definidos no código, não resultados de benchmark. As cenas suspendem renderização contínua quando estão fora da tela, pausadas ou em aba oculta. Recursos gráficos, observers e listeners têm descarte.

As capturas reais foram convertidas para WebP, com dimensões reservadas. Fontes WOFF2 e licenças ficam no próprio site. Skeletons reservam uma página completa do catálogo. Chunks com hash recebem cache imutável; o build remove seus chunks obsoletos.

O navegador desta revisão estava com `prefers-reduced-motion: reduce`. O WebGL e a interação manual foram verificados nesse modo; as linhas do diagnóstico permanecem legíveis e a câmera não depende da animação contínua. Os testes também exercitam pausa, cancelamento de timers e comportamento com movimento normal simulado. Uma falha no carregamento do módulo 3D foi simulada para verificar o fallback e a continuidade do acesso ao download.

Não foram medidos FPS em dispositivos físicos nem pontuações Lighthouse ou métricas de campo LCP, CLS e INP. Não há alegação de pontuação de performance. A revisão visual e os testes não substituem essa medição após publicação.

## Entrega

O site está disponível na prévia local. As alterações não foram publicadas. Capturas de Home desktop/mobile, Studio, diagnóstico e downloads estão em `docs/visual/2026-10-04-website-reform/`.
