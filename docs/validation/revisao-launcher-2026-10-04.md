# Revisão do launcher — 04/10/2026

Removido o preset animado de cerejeiras do launcher e da biblioteca/preferência quando estava selecionado. O arquivo MP4 deixou de ser empacotado no launcher. Wallpapers importados pelo usuário são preservados.

Botões e painéis sobre imagens e vídeos usam opacidade configurável (5–90%, padrão 18%). Filtros não acumulam duas camadas de fundo. Diálogos e listas preservam contraste para leitura. Configurações de aparência incluem opacidade, escurecimento do wallpaper, densidade, pausa ao perder foco, blur do vídeo, movimento da capa e restauração dessas opções. Animações de navegação e clique respeitam a preferência de movimento reduzido e a opção de animações.

Nova instância inicia com Vanilla na release mais recente do manifesto, sem pacote de mods automático. Versões são consultadas novamente a cada minuto enquanto a tela está visível e ao voltar ao launcher; o cache local preserva a lista se a rede falhar. O manifesto contempla release, snapshot, beta e alpha. A opção de seleção de loader e versão específica permanece disponível.

Ícones dos mods priorizam o cache de arquivo/projeto antes de abrir o JAR. Nome e ícone são resolvidos em paralelo, sem o atraso artificial de 1,5 segundo, e os ícones de catálogo são persistidos por nome de arquivo.

A migração usa um diálogo de 896px, controles maiores e rolagem. O diálogo permite selecionar uma instância e gerar/copiar código de compartilhamento. Os novos códigos usam LUXMC1- e 16 números aleatórios: salas expiram em uma hora, compartilhamentos de instâncias em sete dias. A nova API foi publicada na conta Cloudflare do portal existente e o banco recebeu a migração 0006. Convites completos antigos permanecem compatíveis. Códigos de instância compartilham versão, loader e mods identificados no catálogo; arquivos particulares, mundos e JVM args não são enviados. Para transferir todos os arquivos, permanece a exportação ZIP.

O preview da capa usa geometria subdividida e ondulação influenciada por caminhada, corrida, voo, giro e intensidade configurável. O ponto de fixação no pescoço fica ancorado e desativar a opção restaura a geometria. Esse efeito pertence ao personalizador; a animação dentro do Minecraft depende do jogo/mods.

## Verificação

- pnpm check: zero erros e zero avisos de Svelte/TypeScript.
- pnpm build: concluído.
- cargo check: concluído, com três avisos previamente existentes.
- Sete testes nativos de P2P: transporte criptografado, múltiplas conexões Minecraft por convidado, participantes, limite de dez pessoas, bloqueio/expulsão, convites e preservação de servers.dat.
- Teste separado usando relay público, sem transporte IP direto: passou.
- Validação nativa do formato dos códigos de 16 números: passou.
- Teste da geometria da capa: ancoragem, deformação, valores finitos, restauração e reutilização passaram.
- Testes da API: códigos de sala/instância, expiração e rejeição de entradas inválidas passaram.
- Publicação e resolução de ambos os tipos de código no portal real: passaram. Somente os dois registros sintéticos criados para o teste foram apagados.
- Browser: Vanilla padrão, versões estável/beta do NeoForge, navegação P2P, participantes, endereço local, servidor na instância, erro com prazo e recuperação passaram.
- Browser: remoção do preset salvo, reprodução/pausa/retomada de vídeo importado, opacidade configurável, filtro sem fundo duplicado e migração com geração de código passaram.

Não foi possível testar uma partida entre os dois computadores do usuário nesta execução. O teste real exige que ambos utilizem esta nova compilação e instâncias compatíveis em versão/loader/mods. Novas versões e geração/resolução de códigos dependem de acesso à rede; as listas já consultadas continuam disponíveis offline.
