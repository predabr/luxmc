# Verificação da revisão 1 da 3.6

A correção de produção está no commit `c4050f2`, tag `v3.6.1-revision.1`. A versão exibida continua 3.6.0. Os testes posteriores atualizam os seletores dos controles Bedrock e conferem o giro do indicador pelo avanço da animação, sem depender de uma espera fixa de 130 ms.

## Interface

- `pnpm check`: nenhum erro ou aviso.
- `pnpm test:run`: 147 testes passaram.
- `pnpm build`: compilação concluída.
- Revisão de 30 combinações de tela e tema; 13 rotas com animações de entrada concluídas.
- Fluxos de catálogo, conteúdo com mil mods, criação Java/Bedrock, ferramentas de instância, skins, links, idiomas, wallpapers após reinício, rede virtual e armazenamento passaram no navegador local.
- Confirmação de exclusão: cancelar, falhar com arquivo em uso e tentar novamente; instância conservada durante erro e removida após sucesso.
- Cards Bedrock: renomear, manter a instância quando a remoção falha e concluir a exclusão após nova tentativa.
- Busca e seleção de versão Bedrock dentro do launcher, sem botão que transfere a instalação do jogo para a Store.
- Indicadores de download giraram em teste com movimento reduzido e com modo de desempenho. Temas e fundos recuperaram as preferências mais recentes diante de um backup antigo.

## Windows real

O instalador local foi aplicado com `/UPDATE`. A conferência dos arquivos protegidos passou. O verificador de recuperação confirmou a integridade do executável; a diferença de três bytes em relação ao arquivo de compilação corresponde ao marcador de empacotamento NSIS, normalizado pelo verificador.

Foram criadas duas instâncias Java descartáveis. Um arquivo foi aberto com bloqueio exclusivo do Windows: a primeira exclusão retornou erro e manteve o registro. Depois de liberar o arquivo, a pasta e o registro foram removidos. O arquivo da segunda instância permaneceu intacto. As duas instâncias de teste foram removidas ao terminar.

O Bedrock 1.20.81.01 de teste foi instalado usando o pacote previamente baixado, desinstalado do Windows e removido da biblioteca e do cache do Luxmc. A versão atual 1.26.52.3 foi reinstalada ao terminar; o Windows confirmou o pacote 1.26.5203.0. Não havia arquivos de mundos clássicos UWP no diretório de preservação desse teste. A troca de versão conserva os backups dos dados Bedrock existentes.

O Luxmc foi aberto novamente e respondeu. As preferências gravadas confirmaram tema claro, fundo personalizado e animações ativadas.

## Limpeza e limites

Onze testes do verificador/limpeza passaram, incluindo preservação de Documentos, recusa de um destino que contém Documentos e seleção restrita de identidades de pacote Minecraft. A limpeza completa no desinstalador é testada em pastas descartáveis do executor Windows; não foi executada sobre os dados pessoais do autor para validar o código.

Pacotes Bedrock antigos dependem da disponibilidade da Microsoft, da licença e das condições do Windows. O Windows continua gerenciando os dados compartilhados dessa edição. A conexão LAN real entre dois computadores em redes distintas permanece aguardando o teste dos participantes.
