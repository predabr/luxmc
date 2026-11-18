# Preparação de início no Windows — 4 de outubro de 2026

O início rápido agora reaproveita as bibliotecas nativas extraídas quando os JARs de origem e as DLLs permanecem iguais. Antes, cada início reabria os arquivos ZIP e regravava todas as bibliotecas, inclusive candidatos repetidos. O estado fica em `.luxmc-native-cache.json`, dentro da pasta `natives` da versão, e não modifica mundos ou configurações de jogo.

O cache usa tamanho e data de modificação com precisão de nanossegundos dos arquivos de entrada e saída, além da arquitetura do launcher. Alteração, exclusão ou troca dos arquivos, JSON inválido ou alteração do JAR durante a extração invalidam o reaproveitamento. Não substitui verificação criptográfica de downloads: essa permanece no fluxo existente. A primeira extração segue o processamento completo e erros de gravação não produzem um estado válido.

A seleção de bibliotecas agora respeita classificadores explícitos de arquitetura e os campos `os.arch` dos manifestos. O manifesto local de Minecraft 26.2 contém pacotes `natives-windows` e `natives-windows-arm64`: anteriormente ambos passavam na seleção do Windows x64, podendo sobrescrever uma DLL com a variante incorreta. Pacotes antigos com classificador genérico são mantidos. Download, validação e extração usam o mesmo filtro, evitando downloads desnecessários de variantes ARM64/x86 em um launcher x64.

O perfil do Quilt com versão fixa agora usa cache por URL, como o Fabric já fazia. A lista de versões mais recentes continua consultando a API. Instalar uma nova versão ou reparar arquivos faltantes continua exigindo os arquivos correspondentes.

O fluxo rápido passou a validar a instalação uma vez, removendo a segunda verificação idêntica. Os logs registram o tempo total de preparação do launcher até criar o processo Java, separando esse trabalho do carregamento interno do jogo.

O heap padrão foi corrigido: `-Xms1024M` podia exceder `-Xmx512M`. Agora o heap inicial respeita o limite e fica entre 256 MiB e 1 GiB, sem pré-toque. Os presets integrados da interface foram coordenados com a tarefa de configurações para usar Automático ou G1GC compatível. Argumentos avançados continuam disponíveis.

O instalador mantém sua arte 3D, com instruções de primeiro início e atualização mais claras e o botão final “Abrir Lux MC Launcher”. Os comandos Java da área revisada já usam `CREATE_NO_WINDOW` pelo wrapper de processos existente.

## Verificação

- `cargo check`: passou, com quatro avisos existentes.
- Suíte Rust completa final: 113 testes passaram, um ignorado, nenhuma falha.
- Cache nativo: cinco testes passaram, incluindo extração real e restauração de uma DLL excluída.
- Heap padrão/cliente: dois testes passaram.
- Perfil fixo do Quilt: teste sem rede passou na suíte completa.
- Regras de arquitetura e ordem das regras: testes adicionados à suíte completa.

No teste de extração real de 24 DLLs / 6 MiB, a primeira passagem levou 260,64 ms e a passagem com cache 2,67 ms. Uma segunda execução da suíte mediu 302,75 ms e 3,19 ms; a última, sob maior carga de compilação concorrente, mediu 826,12 ms e 4,21 ms. Esses números medem somente a preparação das bibliotecas nativas em uma fixture, não o tempo completo de abrir um modpack, que também depende do disco, processador, Java e dos próprios mods.

Logs: `design-startup-rust-check.log`, `design-startup-rust-all-tests.log`, `design-startup-rust-tests.log`, `design-jvm-presets-tests.log`.
