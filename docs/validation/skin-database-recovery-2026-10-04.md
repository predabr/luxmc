# Recuperação do banco local — 4 de outubro de 2026

Erro reproduzido: SQLite código 11, database disk image is malformed. A leitura de sqlite_master e PRAGMA integrity_check falhavam também fora do launcher. O arquivo apresentava tamanho menor do que a quantidade de páginas indicada no cabeçalho. A causa da truncagem não foi determinada.

Backup integral dos arquivos disponíveis do banco e seus arquivos auxiliares: C:\Users\preda\AppData\Local\Luxmc-build\recovery\20261004-160430

Recuperação feita numa cópia com o comando oficial SQLite .recover --ignore-freelist. Resultado com integridade ok e sem violações de chave estrangeira. Conta recuperada e aparência reconciliada com o armazenamento de autenticação existente. Nenhum token foi impresso ou enviado a serviço externo durante a recuperação.

Os registros de perfis não puderam ser recuperados diretamente. Duas entradas foram reconstruídas a partir das pastas existentes: Vanilla 1.8.9, versão identificada nos logs; Vanilla Perfected 26.2, Fabric 0.19.3, 81 arquivos de mods, identificado no modrinth.index.json. Os diretórios de instâncias, mundos, mods e armazenamento de skins do WebView não foram modificados. Preferências individuais de perfil ausentes receberam padrões conservadores, como 4096 MB de RAM.

Verificação com o executável instalado em modo daemon: inicialização do banco e profiles_list retornaram as duas instâncias; auth_save_appearance salvou a aparência já existente da conta com sucesso. A verificação de integridade após a gravação retornou ok. Essa verificação não publicou uma skin na Microsoft nem substituiu a seleção pendente do usuário. O launcher foi reaberto para reaplicar a skin desejada pela interface.

Nenhuma alteração do executável ou instalador foi necessária para corrigir este erro local. Arquivos originais permanecem no backup. Metadados e registros não recuperáveis podem ter sido perdidos antes da recuperação; não há confirmação de recuperação integral do histórico.

Referência: https://www.sqlite.org/recovery.html
