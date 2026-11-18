# Componentes gerenciados e revisão dos servidores

Revisão `.910` instalada e aberta em 29/10/2026 às 09:14 (São Paulo).

Instâncias Java novas, importadas e duplicadas recebem o agente de aparência do Luxmc. A inicialização verifica seus bytes contra o componente embarcado e repara arquivos ausentes ou alterados. O pacote próprio Luxmc Essentials é reativado sem remover os pacotes pessoais ou mudar sua prioridade. Arquivos de terceiros com o mesmo nome e sem marca de propriedade são preservados.

Servidores novos recebem o componente de suporte e um manifesto que identifica sua função. O componente também é reparado antes da preparação do servidor. A criação usa um diretório temporário próprio e só publica o servidor depois de terminar os arquivos iniciais.

A interface identifica componentes obrigatórios e bloqueia a exclusão dos seus arquivos e pastas. Os comandos de exclusão e edição também validam essa restrição. A exclusão integral da instância ou servidor continua disponível. Arquivos pessoais, mods comuns e históricos de reparo não se tornam obrigatórios.

O proprietário do computador ainda pode remover arquivos diretamente pelo sistema operacional; eles serão reparados na próxima preparação. O agente de skins atua no cliente Minecraft. Um servidor não consegue obrigar clientes de outros launchers, sem suporte equivalente, a renderizar skins personalizadas. As verificações de identidade oficial e de aparências autorizadas da sala foram mantidas. Este componente Java não substitui a integração nativa do Bedrock.

Nesta revisão também foram corrigidos o aviso de catálogo pendente após uma importação concluída e sua atualização ao fechar o criador. As configurações receberam grupos, controles adicionais e o botão de salvar no final do formulário. A preparação pode reaproveitar um JAR Vanilla de outro servidor somente depois de verificar tamanho e SHA-1 oficiais; os mundos continuam separados.

Conexão pública universal continua dependendo de uma entrada TCP alcançável na internet. O site Pages atual não oferece essa entrada. A alternativa de acesso direto por IPv6 foi rejeitada pelo usuário e removida. A ponte P2P existente permite usar o Minecraft de outro launcher mantendo o Luxmc responsável pela conexão. Não foi criado um domínio público, contratado serviço ou publicado código neste turno.

Validação final: Svelte sem erros nem avisos; 232 testes do frontend; 359 testes Rust aprovados, 18 testes optativos ignorados; interface em 1440, 1366 e 960 pixels, além do tema claro; matriz Java instalada de aparência local, compartilhada e de sala com identidade oficial preservada. Servidor Vanilla 1.21.1 isolado aprovado em 55,35 segundos: componente alterado reparado, status Minecraft, comando list via RCON, salvamento, parada normal e exclusão completa. Instalador gerado e conferido contra 557 fontes congeladas. A instalação verificou o conteúdo NSIS, preservou as 17 tabelas e criou o backup `backup-20261029-091440`. As quatro instâncias Java existentes receberam agente, manifesto e pacote próprio íntegros. O processo de servidores separado respondeu na revisão .910, e o catálogo nativo retornou 100 versões sem criar um servidor nem iniciar outro Minecraft.

Instalador local: `C:/Users/preda/Downloads/Luxmc 4.0 - servidores e skins.exe`. SHA-256: `1fc682ca95287d38025fc1d8602b87d603efce2cedab3792ac2768c397970ea9`. Executável instalado SHA-256: `bcaaa0c91be1c57ead226c1b566604f79b9ba69473b74c688bbd7504da70010f`. Recibos privados em `%LOCALAPPDATA%/Luxmc-build/requests/local-server-20261026/managed-delivery.json`. Nenhum push, publicação do site ou infraestrutura externa foi realizado.
