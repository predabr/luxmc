# Sincronização de skin e sessão Microsoft — 2026-10-08

A tentativa de sincronizar a skin usava o access token salvo, sem renovar sessões vencidas nem recuperar uma resposta HTTP 401. A imagem permanecia salva localmente, mas a sincronização Microsoft falhava.

A aplicação agora renova a sessão quando o token está ausente ou expira em até 60 segundos. Uma resposta 401 provoca renovação e um único reenvio, recriando o corpo de URL ou multipart PNG. Uma segunda recusa orienta novo login. Erros fora de 401 e falha de renovação não entram em repetição.

O refresh usa o client ID configurado e confere se o UUID renovado corresponde à conta selecionada. Atualiza somente as credenciais dessa conta, sem trocar a conta ativa nem a skin/capa local. A gravação final de skin atualiza somente aparência e não sobrescreve as credenciais renovadas. O daemon compartilha a mesma implementação.

## Verificação

- cargo check: aprovado.
- Quatro testes de HTTP local: envio válido sem refresh; 401 seguido de sucesso com URL e PNG; segunda recusa sem loop; falha de refresh e 403 sem novas tentativas. Todos aprovados.
- Dois testes de regressão: identidade da conta e preservação da aparência local durante refresh. Aprovados.
- Testes não alteraram uma skin real na Microsoft. O teste final com a conta do usuário depende de executar o novo instalador e aplicar a seleção.
- Versão mantida em 3.2.0. Inclui também compactação automática de GIFs do ajuste anterior.
- Sem publicação no site/GitHub ou substituição automática do aplicativo instalado.

## Instalador local

Build Windows concluído e cabeçalho PE/versão conferidos.

- Arquivo: release-windows/Luxmc-3.2.0-skin-sessao.exe.
- Versão: 3.2.0.
- Tamanho: 20825924 bytes.
- SHA-256: 7e01921d6fc82b706c297e2b1d3c3d4d845d4f4dfaf2e962f6628f9a54a1276a.
- Aplicativo instalado preservado (hash antes/depois igual): True.
- Sem publicação ou instalação automática.
