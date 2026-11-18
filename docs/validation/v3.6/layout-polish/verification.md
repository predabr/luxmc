# Refinamento local do Luxmc 3.6 — 2026-10-09

A escolha de edição ganhou cartões Java e Bedrock com ícones, recursos, entrada animada e resposta ao passar o mouse. A busca e os filtros Java passaram a ocupar linhas independentes; IDs longos quebram linha e a seleção usa explicitamente as cores do tema. A tela de amigos ocupa a largura disponível, distribui os contatos por largura do painel e tem movimento nos cartões.

No site, a abertura, os blocos de recursos, os cartões do catálogo, as capturas, a troca de tela e as respostas do FAQ receberam animações. A captura inicial responde levemente ao ponteiro, sem loop contínuo. Pausa, movimento reduzido e ocultação da página cancelam o trabalho de animação. O estilo visual e as fotos existentes foram preservados.

## Verificação

- Svelte: zero erros e avisos na checagem final; frontend de produção compilado.
- Website: TypeScript, servidor, cliente e bundle visual compilados; seis testes de interface aprovados.
- Java: 56 combinações de tema/cor/filtro, contraste mínimo 4,70:1; quatro larguras verificadas sem sobreposição ou rolagem horizontal. Versão longa visível por inteiro.
- Perfil autenticado após o deploy do site: conta esperada confirmada, papel `owner`, nome personalizado, retrato GIF, banner e três instâncias originais; nenhuma instância temporária restante (`social-native.json`).
- Amigos: cabeçalho e painéis medem a mesma largura do contêiner em 1668, 1440, 960 e 640 pixels. Busca e entrada na LAN verificadas com dados simulados.
- Movimento: teste específico aprovado para a preferência do sistema, animações desativadas e modo econômico, incluindo os ícones durante o hover. O primeiro teste revelou um conflito de especificidade; ele foi corrigido nos três componentes antes do pacote final.
- Site: teste no navegador local e no endereço público aprovou abertura, ponteiro, revelação, troca de captura, pausa, movimento reduzido, FAQ pelo teclado e largura de 390 pixels. CSS e JavaScript publicados correspondem exatamente aos hashes dos arquivos locais.
- LAN atual: nove testes Rust aprovados. Eles cobrem preparação e formatos do componente, convites inválidos, proteção contra parada indevida de sessão já conectada, autenticação, ping Minecraft através dos dois proxies, transmissão TCP bidirecional e manutenção do endereço após consulta perdida. O teste de interface atual aprovou preparação automática, convite, persistência de navegação, saída e entrada. Nenhuma configuração do antivírus ou adaptador foi alterada.

## Entrega e limites

Animações publicadas em https://luxmc-r92.pages.dev, deployment `90c2e775`; evidência em `published-site.json`. Nenhum código, instalador, tag, release ou imagem foi enviado ao GitHub. As capturas neste diretório são evidências locais com dados simulados.

O instalador final `release-windows/Luxmc_3.6.0_local-polish_x64-setup.exe` foi compilado e instalado: 21.196.612 bytes, SHA256 `3dec0ed15d8ddd5d863d69a607381c7dc1d4eaaa5a8c0cf045240d4c848395cf`. Instalação concluída com código zero, cinco arquivos protegidos e todas as pastas de instâncias preservados, executável instalado equivalente ao compilado e verificador de reparo íntegro (`installer.json`, `installed.json`). O aplicativo foi reaberto e confirmou as três instâncias originais, o selo Dono e a navegação real para um mundo existente. O teste de clique nativo no seletor não foi concluído porque o usuário começou a usar a janela; os controles de edição/versão foram validados no navegador. Nenhuma captura nativa foi publicada.

Os testes de LAN executados neste PC não comprovam uma partida entre computadores em redes diferentes. Essa validação continua dependendo do usuário e do amigo.

## Imagem do personalizador — correção posterior

A captura antiga da seção de personalização ainda continha a barra de tarefas. A captura nova enviada pelo usuário foi incorporada com seus pixels intactos em um viewport SVG de 1919 × 1022, excluindo os oito pixels inferiores fora da janela. A seção e o passeio de capturas passaram a usar esse mesmo arquivo, incluindo a visualização ampliada. O endereço do arquivo mudou para evitar a imagem antiga em cache. Publicação Cloudflare `6286092e`; teste real no navegador no domínio público aprovado para as duas apresentações e hash do asset igual ao arquivo local. Seis testes de interface do site também aprovados. Evidências: `personalizer-capture.json`, `personalizer-site-browser.json`, `personalizer-site-section.png` e `personalizer-site-expanded.png`. Nenhuma alteração no launcher/instalador ou no GitHub nesta correção.