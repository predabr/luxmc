# Correção da expiração e atualização do Luxmc

O erro `invalid datetime: 1791129970000` veio da recuperação anterior: o campo expiresAt do armazenamento de autenticação estava em milissegundos e foi transferido diretamente para accounts.expires_at, que o SQLx decodifica como uma data. A recuperação foi corrigida para converter o valor para UTC e normalizar o modelo da skin.

O banco local recebeu backup antes da correção em `C:\Users\preda\AppData\Local\Luxmc-build\recovery\20261004-161755-expiry`. Uma data foi corrigida. O teste com o executável instalado executou auth_accounts, app_init, profiles_list e auth_save_appearance: uma conta decodificada, duas instâncias retornadas, inicialização e gravação da aparência existentes aprovadas. A integridade SQLite após a gravação retornou ok. Não houve envio de nova skin à Microsoft nesse teste.

A inicialização do banco agora converte expirações numéricas em segundos ou milissegundos antes da leitura tipada de contas, mantendo datas ISO existentes e valores nulos. O teste verifica ambos os formatos numéricos, datas existentes, nulos e uma segunda execução idempotente.

O atualizador respeita a preferência de verificação automática, evita notificações repetidas da mesma versão durante a sessão, compara versões beta e releases, permite nova tentativa após falhas e limita o progresso à faixa de 0 a 100. A verificação continua indicando indisponibilidade em falhas de consulta, sem declarar a instalação atualizada.

O comando nativo usa a mesma trava do lançamento do jogo, bloqueia atualização com Minecraft ativo, limita conexão a 15 segundos, o download completo a 15 minutos e inatividade entre blocos a 30 segundos. Downloads vazios ou incompletos não são instalados. Arquivos temporários são fechados antes da limpeza em falhas no Windows. O nome do arquivo deriva da URL oficial inicial, preservando a extensão após redirecionamentos do GitHub.

Validação: pnpm check sem erros ou avisos; cargo check aprovado com quatro avisos preexistentes; 104 testes frontend aprovados; 102 testes Rust aprovados e um teste de relay público ignorado; teste de interface confirmou preferência desativada, falha de consulta, seleção do instalador oficial e nova tentativa após erro de jogo aberto.

O teste da interface usa backend simulado; não instala uma release do GitHub nem comprova assinatura digital do pacote. A entrega e instalação locais são registradas separadamente em updater-repair-delivery.json. A publicação de uma nova release não faz parte desta correção.

Entrega concluída: `C:\Users\preda\Downloads\Lux MC Launcher.exe`, 19.222.254 bytes, SHA-256 `275CD0E562500B77FBDC05713283C38323FE6B9D8614C78364F5EE416324D040`. Compilação release e bundle NSIS aprovados; instalação silenciosa real retornou código 0. Banco, auth.json e settings.json mantiveram seus hashes durante a instalação. A inicialização completa e gravação da aparência foram novamente aprovadas com o executável recém-instalado, e o launcher foi reaberto.
