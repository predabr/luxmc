# Verificação do Luxmc 3.6 — revisão 2

As melhorias foram verificadas em compilações locais antes da publicação. A versão exibida permanece 3.6.0; a revisão do atualizador passa a 2. O pacote público é recompilado pelo GitHub Actions a partir da tag desta revisão.

- Svelte: zero erros e avisos. Frontend e site compilados.
- Rust: 187 testes aprovados na verificação das melhorias, com quatro integrações ignoradas por padrão. Java oficial foi exercitado separadamente.
- Interface: navegação, exclusão com erro e nova tentativa, indicadores de download, 30 telas e conteúdo com 1.000 mods verificados. Seleção Java validada em 56 combinações de tema, cor e filtro, com contraste mínimo de 4,70:1.
- Layout: criação de instâncias e tela de amigos verificadas em larguras de 640 a 1.668 pixels, sem sobreposição. Preferências de movimento do sistema, efeitos desativados e modo econômico verificados.
- Java 8: downloads oficiais Mojang e Temurin validados. Minecraft 1.8.9 abriu e inicializou recursos em uma instância temporária.
- Fabulously Optimized 6.5.0 / Minecraft 1.21.1: 50 arquivos conferidos por hash, duas aberturas com recursos inicializados e nenhum aviso falso. A compilação otimizada importou o pacote em 14,63 segundos e removeu seus arquivos em 1,75 segundo neste PC. Os perfis originais foram preservados.
- Bedrock 1.26.52.3: instalação integrada, importação e remoção de conteúdo e isolamento de pasta verificados. A abertura manteve o processo ativo por 20 segundos; isso não equivale à inspeção visual de uma partida.
- GIFs reais com 25 e 150 quadros foram convertidos preservando duração e repetição. A resolução pode ser reduzida para respeitar o limite de envio.
- Instalação local otimizada: código de saída zero, preferências e pastas existentes preservadas e verificador de reparo íntegro.
- LAN atual: nove testes Rust cobrem convites, autorização, preparação, ping Minecraft pelos proxies, TCP bidirecional e tolerância a consultas perdidas. A interface foi verificada com dados simulados. Uma partida entre redes diferentes continua pendente.
- Site: abertura, revelação de blocos, troca de captura, pausa, movimento reduzido, FAQ pelo teclado e tela de 390 pixels verificados. A imagem nova do personalizador foi conferida na seção e na visualização ampliada, sem barra de tarefas.

As medições descrevem este computador e estas condições de teste. Não garantem tempos equivalentes em outras máquinas, compatibilidade de qualquer conjunto de mods ou funcionamento em todas as modificações do Windows. Os relatórios privados de conta e mundos não fazem parte da publicação.
