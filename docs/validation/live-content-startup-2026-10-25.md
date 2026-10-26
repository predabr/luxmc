# Conteúdos com o jogo aberto e início do cliente

Revisão local 4.0.0, build `40133561069.1`. Sem publicação de release.

## Evidência e alterações

Os dois registros recentes de preparação do Vanilla 26.3 totalizaram 45,32 s e 12,55 s. A etapa agregada `javaLoaderAndProcess` respondeu por 45,27 s e 12,52 s; a verificação dos arquivos levou milissegundos. Os registros do processo indicam que a maior espera ocorreu após iniciar o Java. Esses valores são anteriores à revisão e não constituem um benchmark da versão nova.

O Vanilla 26.3 declara `org.lwjgl:lwjgl-sdl:3.4.3`. O launcher agora mantém os agentes de aparência, LAN e recursos, mas evita iniciar os hooks opcionais de GLFW nesse cliente SDL. Nos clientes LWJGL compatíveis, teclado e Swing aguardam a confirmação da janela pelo monitor nativo. A confirmação continua exigindo uma janela estável; um processo vivo sem janela não é considerado um início bem-sucedido. Um novo registro mede o intervalo entre iniciar o processo e confirmar sua janela.

O bloqueio de edição durante a execução foi restringido aos mods. Resource packs, shaders, datapacks e mundos podem ser instalados com o Minecraft aberto. Downloads continuam passando por verificação de tamanho e hash e publicação a partir de um arquivo temporário. Se um pack diferente tiver o mesmo nome de um arquivo existente, recebe outro nome baseado no hash; o arquivo que o jogo pode estar usando é preservado. Repetir o mesmo download não cria outra cópia.

Mundos são extraídos fora de `saves`, em uma pasta temporária própria. Somente após validar o `level.dat` são movidos para uma nova pasta de mundo. A importação não substitui um mundo existente e limpa sua pasta temporária em caso de erro. Datapacks continuam limitados à pasta do mundo explicitamente selecionado.

Mensagens em português, inglês e espanhol explicam a ativação de cada conteúdo no Minecraft. O launcher não executa comandos no mundo nem altera a seleção de packs automaticamente. O tipo de conteúdo e o mundo de destino ficam vinculados à operação, mesmo se o usuário mudar de aba durante o download.

## Verificação

- `pnpm check`: zero erros e zero avisos.
- `cargo check`: aprovado; seis avisos preexistentes.
- Doze testes nativos aprovados: compatibilidade de mods, tipos permitidos em execução, preservação de packs existentes, importação de pastas e ZIPs, limpeza de importações inválidas, proteção de mundos existentes, destino de datapacks e confirmação de janela.
- Agente Java recompilado; teste isolado confirmou que o HUD aguarda o sinal de janela e continua sendo ativado depois dele.
- Fixtures de autenticação LAN, encerramento do HUD e descoberta de mundos passaram, incluindo inspeção do bytecode disponível de Vanilla 1.20.1, 1.21.1, 26.3 e Forge 1.20.1. Não foi iniciado um cliente Minecraft.

A medição real de abertura da nova versão fica para o teste do usuário. Não há garantia de um tempo fixo: inicialização do Java, drivers e consultas do próprio Minecraft também participam dessa etapa.
