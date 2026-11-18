# Luxmc 3.6 — melhorias locais

## Implementação

- Entrada e troca de telas com transições mais visíveis, configuração de intensidade e velocidade, redução de movimento e modo de desempenho preservados.
- Ícones locais de mods e pacotes são solicitados antes da rolagem, com filas limitadas, cache e cancelamento ao trocar de instância. Imagens remotas continuam dependendo da conexão.
- Diagnóstico usa metadados do mod para reconhecer a edição e o loader. A versão do próprio mod deixa de ser confundida com a versão do Minecraft. O loader continua responsável por validar os intervalos de dependência declarados.
- Java automático valida o executável, confere arquivos e checksums, limita downloads paralelos e oferece fallback oficial Temurin no Windows. Arquivos temporários têm nomes exclusivos.
- Exclusão de instância mantém a linha do banco até confirmar a remoção dos arquivos. Os leitores de logs são encerrados; o auxiliar Crash Assistant é identificado pela sessão e pela pasta exata da instância ao encerrar normalmente o jogo.
- Pastas da biblioteca podem ser removidas preservando suas instâncias. Linhas recentes da página inicial abrem diretamente a instância.
- Bedrock instalado pelo Luxmc ganha página com recursos, add-ons e mundos, diretório próprio, importação e remoção reais de conteúdo. A seleção de canais usa cores com contraste verificado. Pacotes, licença e componentes do Windows continuam oficiais.
- Amigos exibem a identidade pública salva no Luxmc: nome, imagem animada, status e papel confirmado pelo servidor. O layout foi simplificado. O perfil autenticado do proprietário foi conferido e a configuração do servidor corrigida.
- Novos GIFs grandes são convertidos para WebP animado quando suportado, mantendo quadros, duração e repetição. Há fallback GIF e configuração de qualidade. A resolução pode diminuir para respeitar o limite de envio.
- O design do personalizador foi preservado.

## Evidências

- `pnpm check`: zero erros e avisos; frontend compilado.
- `cargo check`: aprovado. Rust: 187 testes aprovados e quatro integrações ignoradas por padrão; duas integrações oficiais Java foram executadas separadamente.
- Vitest: 147 testes aprovados. Revisão de navegação, modais, spinners, conteúdos em duas larguras e 30 telas aprovada.
- Teste de melhorias no navegador: remoção de pasta, navegação recente, 128 solicitações de ícones antes da rolagem, identidade pública, conteúdos Bedrock e 42 combinações reais de tema/cor/canal aprovadas. Contraste mínimo de 4,69:1.
- GIFs reais dos Downloads: 25 e 150 quadros; duração e repetição preservadas. 2,69 MB → 147 KB e 6,76 MB → 669 KB.
- Java 8: downloads oficiais Mojang e Temurin executados e validados. Minecraft 1.8.9 abriu, carregou recursos e permaneceu aberto em uma instância temporária; os arquivos de teste foram removidos e as três instâncias originais preservadas.
- Bedrock 1.26.52.3: instalação integrada real, 229 versões no catálogo consultado, importação e remoção físicas de um pacote de recurso temporário aprovadas. Troca de junctions entre dois diretórios testada no Windows. Abertura real pelo Luxmc manteve o processo ativo por 20 segundos e confirmou o isolamento da pasta. Não houve inspeção visual de uma partida.
- Teste real Fabulously Optimized 6.5.0 / Minecraft 1.21.1: o pacote foi conferido por hash em 50 arquivos. Os testes iniciais revelaram avisos falsos e bloqueio por um auxiliar Java órfão; ambos receberam correções. Verificação final aprovada: duas aberturas com recursos inicializados, oito segundos de estabilidade por abertura, nenhum aviso falso e exclusão física em 4,24 segundos com o mesmo processo do launcher aberto. As três instâncias originais foram preservadas. Esse teste usou a compilação debug final. A instalação otimizada final também foi validada: 50 hashes preservados, importação em 14,63 segundos, exclusão física em 1,75 segundo e três perfis originais preservados (`native-modpack-release.json`).
- O instalador local-upgrades final foi instalado com sucesso: SHA256 `6b65de9b2c5103a502d291c9a21888b17d8813e3dc6d889d97708e3fcac20bef`, 21.193.409 bytes. Cinco arquivos protegidos e todas as pastas de instâncias preservados; o executável instalado corresponde ao compilado e o verificador de reparo confirmou integridade (`installed.json`). Ajustes de layout posteriores possuem evidências separadas em `layout-polish`.

## Limites e publicação

Os números de preparação são medições deste PC durante os testes, sem promessa de desempenho equivalente em outros computadores. O Windows modificado do amigo e a LAN entre dois PCs ainda precisam de validação real. A abertura do processo Bedrock não equivale a uma partida renderizada e jogada; o relatório distingue essas verificações.

Código e instalador permanecem locais, sem publicação no GitHub. As novas fotos do site e do README serão fornecidas pelo usuário; nenhuma captura com o Warframe sobreposto será publicada.

Referências primárias consultadas: [metadados Fabric](https://wiki.fabricmc.net/documentation:fabric_mod_json_spec) e [inicialização do Crash Assistant](https://github.com/KostromDan/Crash-Assistant/blob/app-common-config/common_config/src/main/java/dev/kostromdan/mods/crash_assistant/common_config/loading_utils/JarInJarHelper.java).
