# Pastas como cards, armazenamento e downloads

## Implementação

- Pastas criadas pelo usuário aparecem como cards na mesma grade das instâncias. Clicar abre a pasta; os cards internos ficam nesse espaço. Na biblioteca principal, aparecem as instâncias sem pasta e os cards das pastas. A busca pode localizar instâncias dentro das pastas.
- Arrastar um card para uma pasta altera sua associação. Dentro da pasta, arrastar para o botão Biblioteca retira a instância da pasta. Os diretórios e arquivos das instâncias permanecem separados e intactos.
- Foram removidas a faixa horizontal de pastas, a escolha de grupo no filtro e a barra de escolha acima de cada card. O grupo automático Salas não cria uma pasta na interface.
- A lista persistente contém apenas as pastas cadastradas pelo usuário, com migração das configurações antigas. Nomes de pastas não colidem com o estado interno da biblioteca, inclusive o nome “all”.
- Foi removida a barra de resumo de memória, Java, tempo e mods do cabeçalho da instância.
- A medição individual de armazenamento usa a varredura iterativa compartilhada, sem seguir links, com detecção de erros de leitura. O valor medido atualiza o registro da instância. Arquivos profundos de mundos são incluídos. O total do aplicativo inclui também verificador e desinstalador, quando presentes.
- Modpacks reutilizam um cache de arquivos identificados por hash; conteúdo e tamanho são conferidos antes do reaproveitamento. Cache corrompido não é aplicado. Arquivos maiores que 64 MiB não são armazenados nesse cache. Ele fica no cache de dados do launcher, já coberto pelo relatório e pela limpeza de cache.
- Consultas de versões de manifests Modrinth antigos passam a ocorrer com limite de oito em paralelo. A instalação CurseForge deixa de esperar uma consulta de nomes que não era usada. Os downloads mantêm o limite existente de 24 e a validação de integridade.

## Verificação

- `pnpm check`: zero erros e zero avisos.
- `cargo check`: aprovado, com os avisos preexistentes do projeto.
- Testes de biblioteca Rust: 148 aprovados, zero falhas, dois ignorados por integração externa.
- Browser em claro e escuro: criação de pasta, card na grade, arraste para dentro, conteúdo isolado, reabertura, retirada por arraste e preservação dos diretórios aprovados. Ausência dos seletores e da barra de resumo verificada.
- Capturas da biblioteca e da instância revisadas visualmente nos dois temas.

As melhorias de download evitam solicitações e transferências repetidas. Não foi medido um ganho universal de velocidade em todos os packs ou provedores; a rede e os servidores continuam influenciando o primeiro download. Os testes de integridade verificam reutilização, tamanho inválido e cache corrompido.

Versão mantida em 3.2.0. Sem publicação no GitHub ou site e sem substituir o aplicativo instalado neste PC.

## Instalador

Build Windows de produção concluído com sucesso, incluindo o verificador de integridade. Instalador local: `release-windows/Luxmc-3.2.0-pastas-cards.exe`, com 20.813.582 bytes e versão de produto 3.2.0.

SHA-256: `5c71fb001424afc7289cadd509797e8ceeeb910a55718f4755172db1efa01a74`. O hash também está no arquivo `.exe.sha256` ao lado do instalador. O frontend compilado contém os cards de pasta.

O executável instalado continua na 3.0.2, com SHA-256 `680e6ac799828d8939bb10bcde0607950cc59f2b3f31ac585c23318f2f924ea6`, sem alteração durante este trabalho. O novo instalador foi gerado e conferido; não foi executado no PC do usuário.
