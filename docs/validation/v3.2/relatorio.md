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
