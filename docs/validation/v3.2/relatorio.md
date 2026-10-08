# Luxmc 3.2 — relatório de implementação e validação

## Recursos entregues

- **Preparação para salas:** receita autenticada de qualquer mundo LAN anunciado na sala, criação de uma instância separada, Minecraft e loader fixados, versões exatas dos provedores e comparação da impressão digital dos mods. O progresso continua visível ao navegar. Arquivos sem referência de provedor geram aviso; não são inventados downloads para mods locais.
- **Turma pronta:** planejamento, download, conferência, conclusão e falha aparecem para os participantes. A conclusão perde validade quando o mundo fecha ou sua receita muda.
- **Planejador de atualização:** comparação do manifesto Modrinth/CurseForge, arquivos adicionados/removidos/alterados, configurações alteradas, dependências locais e avisos sobre troca de Minecraft/loader ou remoção de conteúdo de mundos. Aplicação preserva a pasta anterior e os backups.
- **Recomendações para este PC:** compara as instâncias que possuem sessões medidas; mostra memória disponível, pico observado, quantidade de amostras e sugestão de RAM com margem. Não promete compatibilidade de packs sem medições.
- **Coleções compartilhadas:** editor no perfil, até seis coleções de oito packs, IDs de projeto e versão exatos e instalação em uma instância própria pela fila persistente. Clientes antigos preservam os novos campos ao salvar o perfil.
- **Dependências:** leitura de Fabric, Quilt, Forge e NeoForge, bibliotecas incorporadas e aliases; navegação pelos requisitos e cálculo transitivo dos arquivos afetados pela remoção. Regras dinâmicas e restrições de versão continuam sendo validadas pelo loader.
- **Velocidade e consumo:** bibliotecas Forge/NeoForge com até seis downloads em paralelo e Java preparado em paralelo; importações evitam reparar novamente downloads já conferidos. Tempos por etapa e duração de instalação são registrados. Consultas de presença ficam menos frequentes em segundo plano, animações ocultas são pausadas e a memória pode ser liberada ao ir para a bandeja.
- **Interface e site:** perfis com GIF e identidade, coleções, capas de packs sem corte, grupos com arrastar e soltar, seleção visual, foco visível, redução de animações e adaptação em telas menores. O site apresenta os recursos 3.2 e mantém a verificação do Google.
- **Linguagens:** mantidos Rust e Svelte/TypeScript; fontes próprias do site migradas para TypeScript; auxiliar C++ substituído por Rust. Java e Python mantidos em suas funções.

## Verificações concluídas localmente

- Svelte/TypeScript: zero erros e zero avisos; build de produção concluído.
- Rust: compilação concluída; 146 testes de biblioteca aprovados e dois ignorados por exigirem ambiente específico.
- Frontend: 136 testes aprovados.
- Site: 43 testes aprovados; 23 testes adicionais de manifesto, perfis e mídia aprovados após as últimas alterações.
- Sete testes de integração reais de Fabric, Forge, NeoForge e Quilt aprovados, incluindo arquivos baixados e conferidos, cache corrompido e preparação sem rede.
- Quilt: preparação inicial medida em 3,20 segundos; com cache e sem rede, 0,32 segundo. Estes números pertencem ao cenário testado e não representam todos os packs.
- Minecraft Vanilla 1.8.9: janela aberta, respondendo e renderizador inicializado. Preparação com arquivos disponíveis: 9,72 segundos.
- Minecraft Vanilla 1.21.1: o teste encontrou chamada prematura ao GLFW no auxiliar visual. A chamada foi removida, um teste de regressão foi adicionado e o jogo abriu após a correção, com janela respondendo e sem falha fatal no log. Preparação: 4,08 segundos.
- Novos fluxos nos temas claro e escuro: recomendações, dependências, coleções com versão fixada, pastas separadas e progresso persistente ao navegar.
- Auditoria axe da central de manutenção: sem violações detectadas em claro/escuro nas larguras 1440, 960 e 720; sem transbordamento horizontal; navegação por teclado verificada.
- Biblioteca e perfil: grupos persistentes, pastas preservadas e GIF/identidade aprovados nos dois temas.
- Fila: histórico, pausa, fechamento/reabertura da interface, retomada, validação, diagnóstico, confirmação de restauração e temas aprovados.
- Site: layout e navegação aprovados em desktop e largura 390.

## Limites dos testes

As aberturas de Minecraft usaram instâncias novas e temporárias; a instalação atual do usuário foi preservada. Os processos iniciados pelo teste foram encerrados e as instâncias de teste removidas.

P2P foi testado com endpoints locais, interrupção/reentrada, mundos de convidados e troca de mundo. Ainda é necessário testar uma sessão entre dois PCs em redes diferentes e suspensão física do Windows. O PC em uso não foi suspenso. Não há garantia universal sobre NAT, firewall, conexão ou autenticação Microsoft.

As recomendações usam dados locais e observados. O planejamento não executa previamente todos os mods da versão nova: mostra o que é conhecido, e a validação do loader continua necessária. A preparação da sala não transfere mundos, credenciais ou configurações locais particulares do anfitrião.

O GitHub executa separadamente a compilação para três sistemas e, na VM Windows, a recuperação de downloads interrompidos e o ciclo de instalação/reinstalação/desinstalação. A publicação só é concluída depois da conferência dos instaladores.

Os dez cenários de regressão do navegador foram aprovados, incluindo catálogo/scroll, mil mods na instância, criação de instância, laboratório, links, skins, diagnósticos Windows, hospedagem/armazenamento, idiomas e restauração do wallpaper. Os testes foram ajustados para distinguir o participante dos painéis de preparação, a notificação do erro inline e a capa principal de seu fundo decorativo.

Também foram concluídas aberturas reais de Fabric 0.16.9 com FerriteCore no Minecraft 1.21.1, Quilt 0.28.1 no 1.20.1, Forge 47.4.20 no 1.20.1 e NeoForge 21.1.248 no 1.21.1. As janelas responderam, os logs registraram a inicialização do renderizador e não houve falha fatal detectada. O verificador 3.2 passou em oito testes de recuperação e numa prova nativa: uma cópia isolada foi corrompida, a corrupção foi detectada e os bytes originais restaurados. O instalador local foi publicado no site e seu download público conferido por SHA-256; a publicação completa do GitHub foi concluída e aprovada.

## Divulgação e Google

Não é possível garantir indexação imediata. O Google informa que o rastreamento pode levar dias ou semanas, e repetir uma solicitação para a mesma URL não acelera o processo. Com o sitemap respondendo corretamente, acompanhe o Search Console e solicite a indexação da página inicial quando a cota estiver disponível. Referência: [Solicitar novo rastreamento — Google Search Central](https://developers.google.com/search/docs/crawling-indexing/ask-google-to-recrawl).

Meu plano para divulgar o Luxmc é publicar demonstrações reais curtas, uma página útil para cada recurso principal e guias originais sobre instalação de packs, recuperação e salas. Mostre versões testadas e limites, publique o link oficial nas descrições e procure criadores pequenos de Minecraft que queiram testar o produto. Uma comunidade de suporte e relatos verificáveis ajudam a transformar downloads em recomendações. Evite páginas repetidas criadas apenas para palavras-chave. Essas sugestões seguem a prioridade do Google para conteúdo original e útil, com evidência de experiência real: [Conteúdo útil — Google Search Central](https://developers.google.com/search/docs/fundamentals/creating-helpful-content), [Guia de SEO](https://developers.google.com/search/docs/fundamentals/seo-starter-guide).

## Linguagens

Não recomendo outra reescrita completa. Rust e Svelte/TypeScript foram mantidos; o site agora tem fontes TypeScript e o auxiliar C++ foi convertido para Rust. Java continua necessário para os agentes e a integração com Minecraft; Python continua nos utilitários. O desempenho deve ser guiado pelas medições de download, verificação, disco e inicialização do Java, porque trocar a linguagem do site não acelera o bootstrap de um modpack.

Foi concluída também a abertura real de Vanilla 26.3: janela respondendo, renderizador inicializado e nenhuma falha fatal no log. A preparação com arquivos disponíveis levou 4,79 segundos.


## Publicação final

- [Release Luxmc 3.2.0](https://github.com/predabr/luxmc/releases/tag/v3.2.0): Windows, AppImage, DEB, RPM, Arch e DMG universal, além de SHA256SUMS e latest.json.
- [Site oficial](https://luxmc-r92.pages.dev/): arquivos, interface e backend publicados; páginas e sitemap respondendo.
- [Instalador Windows](https://luxmc-r92.pages.dev/download/windows): 20.811.859 bytes; SHA-256 `8a79edb3ba08f4b34e7f5d65622876eeb6a8559606d99ed7cc56670c0e9cbf07`. É o mesmo arquivo do GitHub.
- [CI Windows e Linux](https://github.com/predabr/luxmc/actions/runs/37711434238): concluído com sucesso.
- [Pipeline de release](https://github.com/predabr/luxmc/actions/runs/37711440899): todos os jobs aprovados, incluindo conferência dos sete instaladores e publicação do manifesto.
- VM Windows: instalação, reinstalação e desinstalação retornaram código zero; aplicativo e registro foram removidos no teste; os quatro arquivos protegidos permaneceram íntegros. A recuperação de downloads interrompidos também foi aprovada.
- Conferência pública: nove assets disponíveis, URLs de todas as plataformas corretas, manifesto 3.2.0 verificado e instalador do site com hash idêntico ao GitHub.
- Sua instalação existente continua em **3.0.2**; o executável instalado mantém data de alteração de 6 de outubro e não foi substituído.
- Atualizador público: a interface simulando a versão instalada 3.0.2 consultou os metadados reais do GitHub, detectou 3.2.0 e apontou para o instalador correto. O teste não instalou a atualização; os processos temporários de verificação foram encerrados.
