# Capas e personalizador — 12/10/2026

Alterações locais sobre 3.9.1, sem mudança de versão.

O catálogo consulta a galeria do projeto quando a capa falha ou é uma imagem quadrada. As respostas precisam corresponder ao ID e ao provedor solicitados. As consultas são deduplicadas, ficam em cache e usam no máximo três conexões simultâneas. Imagens quadradas são ignoradas quando há outra imagem de paisagem; na ausência de galeria, a logo original permanece proporcional sobre um fundo derivado da própria imagem. Eventos de imagens anteriores não alteram o cartão atual. As versões exibidas excluem nomes de loaders, como Forge.

A paisagem do Fabulously Optimized é recuperada pelo ID do projeto ou pelo endereço oficial de sua arte, preservando banners personalizados. Os IDs foram conferidos no [Modrinth](https://api.modrinth.com/v2/project/fabulously-optimized) e no [CurseForge](https://www.curseforge.com/minecraft/modpacks/fabulously-optimized). Instâncias existentes podem recuperar a galeria a partir de sua referência de instalação. As capas automáticas são salvas separadamente das capas escolhidas pelo usuário. Os presets locais de Vanilla Perfected, Better MC e Cobblemon também usam sua paisagem correspondente.

O personalizador reúne a prévia 3D e seus controles em um painel, reorganiza a coleção com nomes legíveis, apresenta as opções de capa e mantém o botão de aplicar visível ao rolar. A explicação sobre visibilidade de skins e capas fica em uma seção expansível. Os manipuladores de importação, NameMC, modelo, câmera, rotação, aplicação e sincronização foram preservados.

## Verificação

- `pnpm check`: zero erros e zero avisos.
- `pnpm build`: concluído.
- `pnpm test:run`: 170 testes passaram em 29 arquivos.
- Catálogo: navegação, filtros à direita, paginação, respostas antigas, instalação por teclado, fallback de imagens e identidade da galeria.
- Instâncias: recuperação persistida da paisagem do FO e preservação de capa personalizada durante uma consulta pendente; logo e paisagem do Vanilla Perfected na home, barra lateral e detalhes.
- Skins: PNG original, fidelidade da textura 3D, importação e remoção de capa, recuperação de erro de capa, animação, rolagem, aplicação e persistência após recarregar.
- NameMC: seleção na janela, link colado, deduplicação, limpeza dos eventos e recuperação de falha de importação.
- Aplicação: contas offline, Luxmc e Microsoft; bloqueio de salvamentos concorrentes, aplicação novamente após navegar e recarregar e nova tentativa de sincronização Microsoft.
- Layout: temas claro/escuro, wallpaper, larguras 1600, 1080, 900 e 640; controles sem transbordamento horizontal.
- `git diff --check`: passou.

Os testes de navegador usam dados simulados e Edge sem janela. As capturas estão em `docs/validation/artwork-personalizer/`. O teste de arte e personalizador foi incluído na rotina de testes de navegador. O launcher instalado não foi iniciado e nenhuma conta, skin ou instância real foi modificada durante a verificação.

## Atualização local

A compilação Windows foi concluída e o pacote NSIS foi gerado. O Windows bloqueou a execução de uma ferramenta de compilação do verificador local; o verificador correspondente foi compilado e testado no [GitHub Actions](https://github.com/predabr/luxmc/actions/runs/38254636622), sem alteração da política de segurança. O artefato recebido teve seu digest, commit, hash do verificador e hash normalizado do programa conferidos.

O Defender não encontrou ameaças no instalador nem nos dois executáveis. A análise dos executáveis diretamente na pasta de build retornou `0x80508023`; a análise de cópias idênticas na pasta de validação terminou com código zero. Os hashes das cópias analisadas foram comparados aos arquivos usados na atualização. Esses resultados não representam assinatura Authenticode nem aprovação dos demais fornecedores.

O Controle de Aplicativo bloqueou o instalador local. Os dois arquivos da aplicação foram atualizados diretamente na instalação existente, após backup. A versão continua 3.9.1. Os hashes instalados correspondem à nova compilação e o verificador corresponde ao programa. Banco de dados, contas e configurações mantiveram seus hashes anteriores. O launcher não foi iniciado; a execução dessa compilação sob a política do Windows ainda não foi testada. O relatório está em `artwork-personalizer/installed-local.json`.

O instalador público 3.9.1 foi preservado. Nenhuma nova versão pública ou alteração no site foi publicada nessa atualização local.
