# Hospedagem, capas e armazenamento — 3 de outubro de 2026

## Alterações

- Capas dos modpacks ocupam o cabeçalho inteiro. A seleção prioriza galeria, capa e logo original, com fallback para o ícone. CurseForge utiliza o logo em resolução completa quando não há screenshot.
- Hospedar um mundo direciona à tela Minha hospedagem. A sala aceita até 10 conexões de participantes, incluindo o hospedeiro, com lista, avatares, convite privado, bloqueio de novas entradas, renovação do convite, expulsão e encerramento.
- A expulsão fecha a conexão real do túnel e impede a readmissão daquele endpoint durante a sessão. Cada convidado admite apenas uma ponte Minecraft simultânea. A lista mostra contas declaradas pelo cliente, sem afirmar verificação independente dessas identidades.
- Convites expiram em uma hora; isso não encerra conexões existentes. Renovar revoga o convite anterior sem desconectar os participantes.
- Banners decorativos removidos das ferramentas e da hospedagem. Capturas reais de Minecraft com Complementary Shaders permanecem em contextos selecionados; o personalizador foi preservado.
- Armazenamento inclui dados, banco, configurações, Java, caches, executável e gameDir de instâncias externas. A varredura evita duplicações e links simbólicos e informa falhas de leitura.
- Limpeza de logs abrange os gameDir registrados, preserva mundos e conta apenas arquivos efetivamente removidos. Não foi executada limpeza nos dados pessoais do usuário.

## Validação

- pnpm check: zero erros e zero avisos.
- pnpm build: concluído com saída estática.
- cargo check --locked: concluído; três avisos preexistentes em updater.rs e core/java/mod.rs.
- Testes Rust do túnel: cinco testes passaram, incluindo transferência criptografada de 1 MiB, recusa de convite inválido, limite de participantes, fechamento de entradas, renovação mantendo conexão ativa e expulsão encerrando TCP.
- Teste adicional de infraestrutura pública, normalmente ignorado: relay_only_transfers_without_direct_ip_transport passou com transportes IP diretos desabilitados.
- Testes do site: imagens locais, troca de cena, perspectiva, pausa, suspensão fora da tela, redução de movimento e layouts responsivos passaram.

- Oito roteiros de interface passaram: catálogo, biblioteca com 1.000 mods, criação de instâncias, laboratório, links, personalizador, Windows e hospedagem/armazenamento. O teste de armazenamento nativo passou, incluindo limpeza que preserva arquivos do mundo e não duplica logs.

## Limites práticos

O limite de 10 vale para conexões através da sala Luxmc. O launcher não controla conexões feitas diretamente ao mundo pelo Minecraft fora do túnel. Os dois computadores devem usar esta atualização: o protocolo de sala mudou para v2. O teste com Minecraft aberto em dois computadores físicos continua dependendo desse ambiente; os testes de túnel e interface não equivalem a essa sessão completa de jogo.

Fontes das capturas: https://www.complementary.dev/shaders/ e arquivos sources.json em static/cinema e website/assets/cinema.


## Executável Windows verificado

Build nativo final concluído. Arquivo: release-windows/Luxmc-2.0.2-hospedagem.exe. Janela Luxmc aberta e respondendo no Windows 11 (26300). Verificação pelo WebView real, sem mocks: versão 2.0.2, armazenamento total 93.792.000 bytes (executável 92.222.464 e dados 1.569.536), zero falhas de leitura. Relatório em room-windows-native-report.log e hash em room-windows-artifact.json. Após corrigir o manifesto Windows, armazenamento passou novamente e os cinco testes do túnel passaram; o teste separado de relay público já havia passado. Ambos os modos de compilação utilizam uma única dependência de Common Controls v6.
