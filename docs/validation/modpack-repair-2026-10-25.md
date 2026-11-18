# Modpacks, reparo e configurações — 25/10/2026

Código de retomada: `LUXMC-40-MODPACKS-REPARO-UI`.

## Estado

Correções implementadas apenas no código local, mantendo 4.0.0. Sem commit, push, release, instalação ou alteração de contas/mundos nesta retomada. As alterações antigas da árvore de trabalho foram preservadas. O banco atual contém nove perfis, incluindo DawnCraft; não restaurar backups antigos sobre ele.

## Evidências e mudanças

- BetterAdvancements declara versões de quatro componentes: 0.2.2.142 e 0.3.0.148. A seleção anterior usava SemVer estrito e ignorava ambas. Agora versões estáveis numéricas de até oito componentes são comparadas numericamente; pré-lançamentos ambíguos continuam sem seleção automática. Mantidos os critérios de mesmo mod, loader e arquivos fixados pelo pacote. Duplicatas removidas continuam recuperáveis em backup.
- Os logs de DawnCraft registraram duas transferências simultâneas da mesma biblioteca LWJGL, erros de arquivo inexistente e SHA-1 vazio, seguidos de tentativas repetidas. Downloads para o mesmo destino agora compartilham uma trava; a segunda tarefa reutiliza o arquivo somente após conferir tamanho/hash.
- A verificação de prontidão procurava nomes contendo a versão do Minecraft na raiz de libraries e exigia uma pasta natives extraída antes da abertura. Agora consulta o manifesto local e as bibliotecas nativas da plataforma efetivamente usadas pelo launcher, verificando tamanho/hash e recusando arquivos vazios ou ausentes. A extração continua na preparação normal.
- Downloads/reparos concluídos persistem atomicamente o manifesto da versão. Manifestos idênticos não são regravados; isso permite a conferência seguinte e a leitura offline. O futuro de download é alocado em heap para evitar estouro de pilha no Windows; uma primeira execução de regressão encontrou esse limite e a versão corrigida passou.
- Forge deixa de sobrescrever bibliotecas Maven idênticas a cada preparação. A extração roda fora da thread assíncrona e publica substituições por arquivo temporário; arquivos modificados continuam sendo recuperados.
- No histórico real de DawnCraft, Java 17 já havia sido encontrado imediatamente; Java e loader juntos consumiram 47,52 s. A interface agora descreve a etapa como preparação de Java e carregador, com traduções PT/EN/ES, em vez de atribuir toda a espera ao Java.
- Configurações divididas em Identidade, Desempenho, Java e loader, Janela do jogo, Comandos avançados e Manutenção. Memória em Desempenho, banner opcional recolhido, ações destrutivas separadas e barra de salvamento acessível. Rascunhos são preservados entre categorias; Cancelar restaura os valores do perfil.
- Corrigido o salvamento de perfis com capa padrão local: uma capa inalterada não precisa passar pela validação de nova URL personalizada. URLs novas continuam exigindo HTTPS.

## Verificação

- `pnpm check`: zero erros e avisos.
- `pnpm build`: aprovado; sem gerar instalador ou publicar.
- `cargo check`: aprovado, com seis avisos preexistentes de código não utilizado.
- Validação de mods: 14 testes aprovados; três casos externos ignorados na execução normal.
- Fixture real de Horror OneBlock executada separadamente: oito JARs copiados da instalação e dos backups existentes para área isolada. Quatro versões atuais preservadas, quatro antigas em backup, teste aprovado. Originais não modificados pelos testes. A tentativa inicial de executar todos os casos externos sem suas variáveis de ambiente não foi considerada validação; a fixture explícita foi preparada e executada depois.
- Downloader: sete testes aprovados, cobrindo concorrência, retomada, falhas com preservação de arquivo, cache inválido, nativas antes da extração e manifesto persistido.
- Forge: dois testes aprovados, incluindo preservação de mtime para arquivos idênticos e recuperação de biblioteca modificada.
- Windows: dois testes de limite de pilha de inicialização e reparo aprovados em thread de 1 MiB.
- Navegador: `instance-content.browser.cjs` aprovado em 1440/960 com mil mods, salvamento com falha simulada e recuperação; `instance-settings-repair.browser.cjs` aprovado em 1440/1000 para categorias, rascunho, cancelamento, salvamento e limites da tela; `java-runtime25.browser.cjs` aprovado para seleção de Java 16/25. Os fluxos existentes foram adaptados às categorias e o teste novo entrou na suíte de conteúdo.

Evidências locais de execução: `C:/Users/preda/AppData/Local/Luxmc-build/requests/refinement-20261024/resume-1025-*.log`.

![Configurações em desktop](modpack-repair-2026-10-25/settings-1440.png)

## Limites

Não foi iniciada uma partida completa de Minecraft com este código novo e não foi medido um novo tempo até o menu. O teste dos JARs não substitui a validação de dependências pelo loader durante uma partida. O launcher instalado permanece na build pública anterior até autorização para integrar/instalar. Nenhuma garantia universal de abertura de todos os modpacks é apresentada.
