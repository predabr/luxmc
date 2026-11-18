# Ajustes de modpacks, perfil e site — 08/10/2026

## Entrega

O código permanece na versão 3.5.0. Estes ajustes geram um instalador local para teste; não substituem a release já publicada no GitHub nem o instalador público do atualizador. O site e seu serviço social foram publicados em produção. As novas capturas serão fornecidas pelo usuário.

## Início e estabilidade

- Removido o acesso “Modpacks para o seu PC” da página inicial.
- A inserção de uma instância com ID já existente agora atualiza o registro, em vez de duplicar um cartão com a mesma chave. A atualização da lista também elimina IDs repetidos.
- Acrescentada uma barreira de recuperação para falhas de renderização, com ações para tentar novamente ou recarregar a interface.
- O perfil público pode ser fechado durante o carregamento. Falhas de rede deixam de prender o usuário nessa janela.

## Instalação e mods duplicados

- O reparo identifica mods pelos IDs declarados nos metadados Fabric, Quilt, Forge e NeoForge.
- A versão indexada pelo modpack tem preferência sobre uma cópia extra do mesmo conjunto de IDs. Cópias byte a byte idênticas também são deduplicadas.
- Os arquivos removidos ficam em `.luxmc/removed-duplicates`, acompanhados de um registro da decisão. Outros mods e cópias que contenham módulos adicionais são preservados.
- Versões diferentes sem uma escolha confiável do manifesto não são apagadas arbitrariamente. Conflitos reais que permanecerem continuam sendo diagnosticados.
- O reparo funciona ao conferir a instalação, preparar o jogo e abrir o diagnóstico da instância; a pasta de um jogo em execução não é alterada.
- A leitura dos metadados de arquivos CurseForge foi paralelizada, com limite de oito leituras. O registro no banco continua em uma única transação.
- A verificação do estado de lançamento e a cópia de dados importados saíram do trabalho síncrono da tarefa assíncrona.
- A instalação informa separadamente o registro e a conferência dos arquivos. O histórico registra o tempo de arquivos e de registro/validação.
- Requisições sociais identificam a versão correta do launcher. Downloads usam limite de inatividade de leitura, permitindo transferências longas que continuem progredindo.

Nenhum resultado deste trabalho comprova uma velocidade específica de instalação do All the Mods 10 no computador do usuário. Não foi feita uma nova instalação completa desse pack nem uma medição comparativa com CurseForge.

## Catálogo

As abas seguem a ordem **Descrição → Galeria → Changelog → Versões**. Descrição, galeria e versões usam cache para exibir dados imediatamente e são revalidadas ao abrir o projeto. O changelog vem do provedor e da versão selecionada. A interface distingue carregamento, ausência de notas e erro de consulta; não cria notas fictícias.

## Perfil e ferramentas de dono

- O nome de exibição e o retrato/GIF salvos no perfil público passam a aparecer no perfil do próprio usuário e nas barras laterais.
- O nome oficial da conta continua sendo usado para autenticação e para buscar a cabeça de Minecraft quando necessário.
- A conta social autenticada de Spect3rBW recebe o selo **Dono** no servidor. O selo não é atribuído apenas por um apelido semelhante nem por um campo enviado pelo cliente.
- O dono ganha uma página para diagnosticar uma instância, consultar o histórico de logs e executar manutenção.
- O diagnóstico reutiliza a remoção de dados sensíveis. A manutenção corrige duplicatas seguras e agenda uma verificação completa antes da próxima abertura.
- O backend confere o papel de dono a cada operação. A manutenção exige que o Minecraft esteja fechado.

O serviço em produção confirmou `role: owner` para Spect3rBW. A consulta encontrou o nome público vazio; o usuário deve salvar o nome e as imagens desejados no editor. Dados públicos existentes não foram sobrescritos automaticamente.

## Migração

Além da detecção dos launchers conhecidos, o usuário pode escolher uma pasta de jogo. O importador reconhece metadados Prism/MultiMC e CurseForge quando disponíveis; permite confirmar nome, versão e loader. Copia mundos, mods, configurações, scripts, resource packs, shaders, screenshots e opções para uma instância separada, preservando a origem. Links simbólicos não são percorridos.

A seleção por pasta não importa contas, autenticação nem componentes proprietários exclusivos de outros clientes. Não é uma certificação de compatibilidade de todas as versões de Minecraft existentes.

## Site

Mantido o estilo atual e acrescentada uma seção de consumo observado, com relatório público das amostras. A referência é da versão instalada **3.2.0**, explicitamente identificada como diferente da 3.5.0:

- 739,9 MiB de memória residente somada entre sete processos, incluindo WebView.
- 1,25% de CPU média, normalizada por 12 processadores lógicos.
- 20 amostras aproximadamente a cada segundo, interface aberta com wallpaper personalizado e Minecraft fechado.

A soma de working sets pode contar memória compartilhada mais de uma vez. A medição não representa todos os computadores, páginas ou personalizações.

Produção: https://luxmc-r92.pages.dev/

Deploy: https://f3c2329d.luxmc-r92.pages.dev/

Foram conferidos HTTP 200 para página inicial, relatório de consumo e sitemap, além do papel de dono autenticado. A verificação pública também confirmou que o instalador Windows original 3.5.0 continua com SHA-256 `870261977b3e4f59bc6308ce892febf09ab8d17993c64e453b48572aff67ab2c` e que o manifesto público corresponde à release.

## Validação

- `pnpm check`: sem erros e sem avisos.
- `pnpm test:website`: 44 testes passaram, incluindo a proteção contra falsificação do papel de dono.
- `pnpm build:website`: concluído.
- Suíte de navegador de conteúdo: passou nos dez cenários de catálogo, instâncias, criação, laboratório, links, skins, Windows, armazenamento, idiomas e wallpaper.
- Regressão de pastas/perfil: passou nos temas claro e escuro, incluindo GIF, identidade pública e proteção contra instância duplicada.
- `cargo check --locked`: concluído, com cinco avisos já existentes no projeto.
- `cargo test --lib --locked`: 156 testes passaram, nenhum falhou e dois ficaram ignorados. Inclui migração de mundos/scripts, reparo NeoForge preservando a versão do manifesto, preservação de versões ambíguas e conservação de módulos adicionais.
- Site em produção conferido em larguras de 390, 1024 e 1440 pixels: sem transbordamento horizontal nem erros JavaScript.
- Build Windows de produção e instalador NSIS concluídos, mantendo a versão 3.5.0.
- Verificador executado com os binários gerados em uma pasta temporária: restauração da cópia 3.5.0 quando não há versão anterior, retorno à 3.2.0 quando há cópia anterior válida, confirmação da integridade após o retorno e recuperação mesmo com o cache atual corrompido. Nenhum desses testes alterou a instalação do usuário.

## Instalador local

Arquivo: `release-windows/Luxmc-3.5.0-ajustes.exe`

Tamanho: 20.887.334 bytes.

SHA-256: `eb3004efe050ea4a6f76ee3dc7856dc6cd8a2c79a569807cd8f8e71d2823ffad`.

O verificador prefere a versão anterior válida quando encontra corrupção. Sem uma versão anterior disponível, restaura a cópia verificada da versão atual. A geração do pacote e esses testes não instalaram nem desinstalaram o launcher no PC do usuário.

Evidências: `docs/validation/current-patch/delivery-2026-10-08.json` e `docs/validation/current-patch/native-recovery-2026-10-08.json`.

Não foram testadas todas as versões de Minecraft nem todas as configurações de hardware. A causa exata do travamento observado pelo usuário ainda depende do log da ocorrência; a duplicação da lista foi um defeito concreto reproduzido e corrigido.
