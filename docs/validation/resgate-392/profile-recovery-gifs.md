# Identidade salva e GIFs por plano — 13/10/2026

A consulta autenticada ao perfil real confirmou que o nome personalizado `OWNER` e o avatar `image/gif` permaneciam no servidor. Depois da falha anterior de publicação, a consulta inicial do perfil próprio tinha falhado e o launcher não tentava carregá-lo novamente durante a reconexão normal da presença. As áreas que usam `friendsState.ownProfile` exibiam o nome da conta e a skin como alternativa.

O estado social agora repete a consulta quando o perfil está ausente e atualiza perfis próprios existentes a cada minuto. Consultas simultâneas são deduplicadas e resultados de uma conta antiga não atualizam a nova conta. Os snapshots de amigos incluem o nome de exibição e o status salvos; o cache de imagens segue o plano ativo do snapshot. Arquivos GIF não são incluídos nas notificações periódicas de presença.

Novos GIFs de avatar ou capa exigem Luxmc Plus ou Pro ativo. A interface avisa e bloqueia o envio em Free; a API verifica a assinatura real sem confiar no plano enviado pelo cliente. Ao vencer, o servidor oculta o GIF e mantém o arquivo salvo. Alterar outra informação em Free preserva esse arquivo; renovar restaura sua exibição. Nome, descrição e imagens estáticas continuam disponíveis no Free. A foto da sessão do site segue a mesma regra.

O site foi publicado em `https://fc1871d3.luxmc-r92.pages.dev`. O worker de amizade foi publicado com a versão `e25bec34-f576-40ad-9402-6ec2132f9b28`. O perfil de produção foi novamente consultado pelo executável instalado: nome salvo, GIF e os três perfis de amigos recuperados. Nenhum conteúdo desse perfil foi regravado para tentar restaurá-lo.

Verificações: `pnpm check` sem erros e avisos, builds do site e do frontend concluídos; 41 testes de perfil, mídia, amizade, planos, sessões e publicação aprovados. No navegador, uma falha inicial do perfil próprio é seguida de recuperação automática; Free, Plus e Pro seguem a regra de upload. O editor existente também passou nos testes de salvamento, GIF, enquadramento, prévia, ordenação e preservação de rascunhos.

O NSIS final foi compilado e extraído com verificação byte a byte do executável. No WebView2 nativo conectado ao serviço de produção, o nome salvo apareceu na área de amigos, o GIF carregou e o editor manteve ambos. O launcher permaneceu aberto durante o teste. Nenhum executável de reparo foi incluído.

A instalação local terminou com código zero, preservou os hashes da base e das configurações, e o programa instalado corresponde ao extraído do NSIS. O backup está em `C:/Users/preda/AppData/Local/Luxmc-build/backups/profile-gif-20261013-install`. O instalador local está em `C:/Users/preda/Downloads/Lux MC Launcher - Perfil 3.9.2.exe`; o launcher atualizado foi aberto para o usuário.

SHA256 do programa: `51f4dbe0afe5ed43bcdc838638e8cb6a1b4fd9387644fb1e6fe95ceef059ed81`.

SHA256 do instalador: `2ec6316d5851045964da78a6158b8f8c7742a4a2becaaf0d3652ca34ec9238a6`.

Esse mesmo instalador foi publicado no download do site em `https://ed350fa7.luxmc-r92.pages.dev`. A versão continua 3.9.2; não foi criada uma release nem enviada qualquer alteração ao GitHub.
