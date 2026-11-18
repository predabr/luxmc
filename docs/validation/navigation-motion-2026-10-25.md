# Fluidez de navegação e revisão de loaders — 25/10/2026

O usuário confirmou que o NameMC abriu na revisão 40133561063.1. Pediu corrigir atraso nas transições e revisar os loaders; ao concluir, abrir para ele testar, sem testes de uso pelo agente.

Mudanças desta etapa:
- Entrada de páginas sem fade do contêiner inteiro, escala ou cascata de atrasos. Até quatro superfícies visíveis, 120–180 ms e deslocamento de 2–6 px.
- Removido MutationObserver que examinava adições em toda a subárvore de cada escopo animado; capas e catálogos mantêm suas próprias entradas. Nenhuma espera de animação bloqueia a navegação.
- Retirado fade adicional do grid/lista de instâncias; preservadas animações de interação e entradas das superfícies.
- Amigos agora é a primeira rota pré-carregada. Links principais antecipam código ao apontar, focar ou pressionar. Preload somente do código, com rejeição tratada e nova tentativa permitida em falha.
- Indicador lateral passa a 160 ms.
- Importação manual aceita metadados Fabric em Legacy Fabric e Babric. Manifestos passam a reconhecer identificadores explícitos desses dois loaders e suas versões fixas.

Limites: sem partida de Minecraft, inspeção visual, benchmark ou novas suítes de testes nesta etapa, por solicitação do usuário. Checagens obrigatórias de código e compilação são executadas. Preparação real de bibliotecas Legacy Fabric/Babric já havia passado na etapa anterior; isso não garante compatibilidade de todos os mods nem substitui uma partida completa.

Referências consultadas: https://legacyfabric.net/usage.html e https://babric.github.io/use/installer/.

Estado: pnpm check aprovado (zero erros/avisos), cargo check aprovado (seis avisos preexistentes). Instalador 4.0.0 local 40133561064.1 em compilação. Ainda não aplicado. Sem publicação GitHub nesta etapa.

Backup atual concluído: C:\Users\preda\AppData\Local\Luxmc-build\backups\motion-4.0-20261025-163750; 10 perfis e 1827 arquivos protegidos. A comparação antes/depois será feita antes de reabrir o aplicativo; não restaura backups antigos.

## Compilação concluída

Build Windows 4.0.0 local 40133561064.1 concluída com sucesso; instalador NSIS copiado e hashes de origem/destino idênticos. Arquivo C:/Users/preda/Downloads/Luxmc-4.0-fluidez-20261025.exe, 21.648.631 bytes, SHA256 3fbd4bf7e2f2f9dd7e6992869a5f62f6bc786183209b1270fbd4889eb6c6b8e7.

Pendente aplicação: Luxmc ainda ativo na bandeja (helper single-instance io.github.luxmc.Luxmc-siw). Não chamar CloseMainWindow nesse helper nem matar processos. Pergunta assíncrona pediu encerramento normal; não houve resposta até finalizar a compilação. Rotina privada apply-motion-1025.ps1 está pronta e só prossegue sem luxmc/java/javaw, confere hash, snapshot antes/depois e binário e abre o instalado. Não foi executada. Não apresentar a revisão nova como instalada.

Nenhum teste GUI, partida, benchmark ou nova suíte foi executado nesta etapa; somente pnpm check, cargo check e build obrigatórios. NameMC confirmado pelo usuário na revisão anterior. Sem commit, push ou publicação de binário no GitHub.

## Instalação concluída

Usuário confirmou encerramento. Atualização aplicada com saída zero; dados e binário conferidos. Dez perfis, duas contas, identidades, 1.827 arquivos protegidos, seis wallpapers e 26.305 arquivos de instâncias preservados. Build 40133561064.1 iniciada para o usuário testar. Não houve navegação/teste interativo nem publicação GitHub.
