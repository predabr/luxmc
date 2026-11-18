# Recursos, capas e movimento — 13/10/2026

Alterações locais sobre `f78a19f`. A publicação no GitHub está cancelada e o PR 6 foi fechado sem merge.

O catálogo exclui imagens de patrocinadores pelos metadados, URL e link da descrição antes de buscar uma imagem horizontal. A descrição real do SkyFactory 4 contém somente um anúncio da Nodecraft: ele é descartado e a imagem original do projeto permanece como alternativa. A consulta mantém a identidade do provedor/projeto. Imagens são carregadas sob demanda, cards fora da tela usam `content-visibility`, sondagens de proporção são agrupadas em três e deduplicadas com cache limitado.

O agendamento de animação usa prazos para compensar a frequência do monitor e manter a taxa escolhida sem acumular frames após atrasos. O personalizador principal continua andando, correndo e voando; usa menos frames durante o jogo e modo de desempenho. O wallpaper mantém sua pausa por foco, visibilidade e jogo. Efeitos de clique são finitos, removidos após terminar e suspensos no modo de desempenho ou movimento reduzido.

O perfil JFR do Zombie Invade apontou enumeração repetida em `FilePackResources` e `ZipFile.getZipEntry`. Foi implementado um experimento para reaproveitar somente os metadados de entradas de instâncias exatas de `ZipFile`, com chaves fracas e orçamento estimado de 16 MiB. Preserva dados, ordem, CRC, fechamento e isolamento de mutação; subclasses como `JarFile` seguem a enumeração original. Arquivos com mais de 50 mil entradas usam o caminho original. A leitura dos recursos e inspeção de segurança continuam completas. O experimento permanece desativado por padrão e exige a opção JVM `-Dluxmc.resourceIndex=true`: o teste do Zombie Invade não comprovou ganho relevante no tempo total.

Verificações concluídas antes do teste nativo:

- `pnpm check`: zero erros e avisos; frontend de produção compilado.
- `cargo check`: concluído, com cinco avisos existentes de código não utilizado.
- 31 testes de identidade/capas/galeria e recuperação do cache aprovados; mais dez testes de agendamento em monitores de 60, 120, 144, 165 e 240 Hz.
- Browser: descrição real dos cinco SkyFactory verificada; anúncio da Nodecraft rejeitado.
- Browser: wallpaper em vídeo retoma após reinício, alteração de porta e cache vazio; efeitos finitos e interrupção dos loops aprovados.
- Java 17 e 25: transformações existentes de LAN/login preservadas; leitura ZIP, metadados, isolamento de mutação, fechamento e execução do bytecode transformado aprovados. A enumeração isolada de 12 mil entradas repetida 80 vezes caiu de 136 para 39 ms em Java 17 e de 119 para 42 ms em Java 25. Em Java 8, o mesmo mecanismo passou e a enumeração caiu de 714 para 135 ms. Isso não é o tempo de abertura de um modpack.
- Site: build e 14 testes de UI/artefatos aprovados; três cards aparecem, efeitos terminam e movimento reduzido é respeitado. Publicado em `https://0b3a236c.luxmc-r92.pages.dev` e confirmado no domínio principal.

Os tempos de referência do Zombie Invade na mesma máquina foram 187,93 s e 189,15 s até os recursos, com janela em 54,21 s e 57,39 s. O usuário não tem CurseForge instalado; não há comparação medida com esse launcher.

O primeiro candidato, extraído do NSIS, abriu Zombie Invade com o experimento ativado: janela em 58,81 s, recursos em 191,30 s, inicialização de mods confirmada e estabilidade por mais 60 s. O hook em `FilePackResources` foi confirmado no log do processo. Como os tempos de referência foram 187,93–189,15 s, isso não comprova melhora no carregamento real. Vanilla 26.3 abriu a janela em 21,23 s e os recursos em 23,53 s, mantendo-se aberto por 60 s.

A recuperação de capas também remove anúncios antigos do cache automático, preservando as imagens selecionadas pelo usuário. Os cards passam a buscar galeria somente quando se aproximam da área visível.

Teste com 36 projetos: 14 cards mantidos pela virtualização existente, somente quatro galerias solicitadas na posição inicial. Rolar até o final carrega a galeria do último card. O canvas real de partículas segue a alteração de FPS, deixa de desenhar durante o jogo e retoma depois.

RLCraft/Forge 1.12.2 com o experimento ativado: janela em 18,98 s, recursos em 323,11 s e estabilidade por mais 60 s. A medição anterior era 317,47 s. Também não comprovou aceleração relevante; a proteção de ativação explícita foi mantida no candidato final.

ATM10/NeoForge 1.21.1: janela em 31,96 s e recursos/inicialização de mods em 210,60 s, estável por 60 s. A medição antiga era 113,41 s de janela e 292,45 s de recursos: o intervalo da janela aos recursos ficou praticamente igual, cerca de 179 s. Não foi atribuída ao experimento a diferença do tempo anterior à janela. Todos os quatro perfis originais foram preservados e os processos de jogos iniciados pela validação foram encerrados.

O candidato final foi compilado como NSIS Windows x64 às 20:04 de 13/10/2026, com o experimento de ZIP desativado por padrão. O executável extraído foi comparado byte a byte ao binário de entrada: somente o marcador de tipo de bundle foi alterado pelo empacotamento. Nenhum executável de reparo foi incluído.

No WebView2 do pacote final, Andar, Correr e Voar produziram frames diferentes no canvas, mantendo o launcher aberto. A busca real do CurseForge por SkyFactory 4 exibiu a arte original do projeto, sem o anúncio da Nodecraft. Vanilla 26.3 abriu a janela em 18,91 s, inicializou recursos em 21,03 s e permaneceu estável por mais 60 s; o experimento de ZIP não foi ativado. Os quatro perfis originais foram preservados.

A instalação silenciosa local terminou com código zero. O programa instalado corresponde ao executável extraído, e os hashes da base e dos arquivos JSON de configuração permaneceram iguais durante a instalação. O backup consistente foi salvo em `C:/Users/preda/AppData/Local/Luxmc-build/backups/resource-index-final-20261013-install`. O instalador local está em `C:/Users/preda/Downloads/Lux MC Launcher - Desempenho 3.9.2.exe`.

SHA256 do programa instalado: `ddd56299a03ed627ef5ed7d853725fa0d329e218d201289dc043757e08c98521`.

SHA256 do instalador: `cb52f0f97b0daa8cd287cb3810bfdcde19279f002f1d65d2f92d861eb130b7f6`.

A redução grande da abertura de todos os modpacks continua sem solução comprovada. O custo após a janela apareceu praticamente igual no ATM10, e os testes de RLCraft e Zombie Invade não mostraram aceleração relevante. Não há comprovação de abertura rápida de todos os modpacks nem de redução total de RAM apenas pelos testes de agendamento. A enumeração de arquivos isolada não será apresentada como ganho de carregamento real.
