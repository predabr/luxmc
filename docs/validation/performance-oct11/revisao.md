# Otimização do Luxmc — 11/10/2026

Alterações locais, mantendo a versão 3.8.0.

## Preparação de Minecraft e modpacks

- Auditoria de metadados por leitura com busca no JAR, sem carregar integralmente cada arquivo pequeno. O leitor confere alteração do arquivo e cancelamento. A verificação profunda mantém a leitura do conteúdo e CRC.
- Download de recursos e bibliotecas em paralelo, usando o mesmo semáforo do gerenciador de downloads.
- Preparação de Java e loader em paralelo. A procura e a verificação de Java rodam fora dos executores assíncronos de download.
- Java personalizado é verificado contra a versão requerida. Um binário inválido aciona a preparação automática do runtime compatível. No Windows, a verificação de javaw usa o java.exe correspondente.
- Hashes de arquivos Modrinth e CurseForge são calculados em tarefas de trabalho com concorrência limitada a quatro, ou menos conforme o processador. A espera por verificação e por cache pode ser cancelada sem registrar a instância.
- O cliente Minecraft é conferido por tamanho e hash esperado, com reutilização do cache de arquivos verificados, em vez de aceitar somente o tamanho do arquivo.
- A opção de verificação completa volta a executar a auditoria profunda.
- O Shield mantém hashes e inspeção de bytecode, distribui arquivos entre trabalhadores e verifica a pasta de mods depois da preparação de loader e aparência, antes de iniciar o jogo. Hooks conservam a verificação anterior à execução e o lançamento conserva a verificação final.
- O estado de arquivos é atualizado após essa preparação para evitar repetir a conferência por alterações feitas pelo próprio launcher.
- Removida a limpeza automática da memória residente no Windows durante instalação, após início do jogo e a cada 12 segundos no daemon. A ação manual e a limpeza ao ocultar a janela permanecem disponíveis. A memória residente pode aparecer maior porque os dados usados continuam em RAM; o Windows continua gerenciando a pressão de memória.

O uso de SetProcessWorkingSetSize com os dois parâmetros em -1 remove páginas residentes. Evitar chamadas periódicas durante trabalho ativo é uma decisão de implementação para reduzir a necessidade de recuperar essas páginas. [Documentação da Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-setprocessworkingsetsize).

## Interface

- Navegação com indicador móvel e resposta animada dos ícones.
- Entrada de páginas limitada a dez superfícies visíveis, com filtragem de elementos aninhados antes da medição de layout.
- Cartões do catálogo compartilham um observador. Cartões vistos recentemente reaparecem sem repetir a entrada a cada reconstrução da lista virtual.
- Animações interrompidas ao ocultar a página e respeito ao movimento reduzido.
- Imagens carregadas do cache reconhecidas sem reiniciar o esqueleto de carregamento.
- Instalação com etapas Pacote, Arquivos, Conferência e Pronto, tempo decorrido e barra indeterminada animada.
- O término de um grupo de arquivos ou de um arquivo individual não é mais mostrado como conclusão do download inteiro.

## Verificações concluídas

- `pnpm check`: zero erros e zero avisos.
- `pnpm build`: aprovado, produção em `build`.
- Vitest: três arquivos e nove testes de progresso, compatibilidade e identidade de imagens do catálogo aprovados.
- Testes de navegador com backend simulado: quinze rotas em cada tema, claro e escuro, sem erros de JavaScript nem transbordamento horizontal. Os cenários verificam a classe do tema aplicado; os aliases antigos Boost e Organizador redirecionam para seus destinos existentes.
- Catálogo verificado em janela de 960 px.
- Cancelamento na conferência simulado sem publicação da instância na biblioteca.
- Movimento reduzido sem animação de entrada.
- Ensaio de interface com 800 superfícies aninhadas: onze medições de layout e onze animações, incluindo a do contêiner. Sessenta cartões usam um observador compartilhado.
- Regressão do editor de perfil: persistência, falha e nova tentativa de salvar, imagens, GIF, enquadramento, prévia, ordenação e proteção do rascunho aprovados.
- Nove arquivos Rust alterados passaram pelo parser de rustfmt. Isso confirma sintaxe, sem substituir a compilação.
- Comparação de referência em Python, somente leitura, com 48 JARs instalados, em três rodadas: metadados equivalentes; 85.096.866 bytes lidos pela estratégia de arquivo inteiro e 4.838.794 bytes pela estratégia com busca. Os arquivos conservaram tamanho e data. Não é um benchmark do Rust nem do início do Minecraft.

## Pendências de validação nativa

`cargo check` não concluiu. Houve bloqueio de scripts de compilação pelo Controle de Aplicativos do Windows, com erro 4551, e erros E0463 nas dependências de macros yoke_derive e derive_more_impl. A limpeza específica do cache de yoke-derive não resolveu o problema. O log Code Integrity confirmou 27 bloqueios recentes de componentes Rust relacionados a esse build. Nenhuma política ou proteção foi desativada.

Os testes Rust adicionados para leitura parcial, CRC, alteração de arquivo, cancelamento de verificações, hashes e cliente de mesmo tamanho com conteúdo incorreto ainda precisam executar em um ambiente de compilação autorizado. Os testes existentes de Shield também precisam rodar nesse ambiente.

O novo executável não foi gerado. O launcher instalado e o Minecraft não foram iniciados. A aceleração real de abertura de cada modpack precisa ser medida após compilar e instalar estas alterações; nenhuma promessa de que todos os modpacks abrem foi validada.

## Servidores

A proposta de infraestrutura, integração de planos, isolamento, backups e piloto está em [Luxmc Worlds](C:/Users/preda/OneDrive/Documenti/Luxmc/docs/luxmc-worlds-servidores.md). Nenhuma máquina paga foi contratada nem servidor hospedado criado nesta revisão.
