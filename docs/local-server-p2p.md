# Servidor local P2P — revisão local em desenvolvimento

Esta documentação descreve a reforma `LUXMC-P2P-REFORMA-COMPLETA-01`. A compilação e a validação desta revisão são locais; os instaladores públicos e a infraestrutura de conexão não são publicados automaticamente por este trabalho.

## Criar e entrar

1. Na instância, escolha **Hospedar servidor local**. O botão **Rede LAN** mantém o caminho separado para compartilhar um mundo aberto no Minecraft.
2. Com Pro confirmado, crie um servidor com nome, Minecraft, 512–8192 MiB de RAM e até vinte jogadores, incluindo quem hospeda. A memória do servidor soma-se à memória do cliente e do sistema.
3. Leia a [EULA do Minecraft](https://www.minecraft.net/eula) e aceite somente se concordar. O launcher não aceita por você.
4. Se usar mods, configure a família e a versão do loader em **Configurações**, ou escolha o modpack no catálogo do assistente. Não é necessário ter uma instância instalada. Os instaladores oficiais de Fabric, Quilt, Forge e NeoForge são preparados quando o usuário inicia o servidor. Isso não converte o mundo de um loader para outro.
5. Inicie e acompanhe o estado. A primeira preparação inclui download verificado, Java, instalação do loader e geração do mundo. **Parar e salvar** também cancela uma preparação em andamento.
6. No Minecraft deste PC, com conta autenticada, mesma versão e mods necessários, abra **Multijogador → Conexão direta** e cole `127.0.0.1:porta` mostrado no painel.

O servidor gerenciado usa autenticação oficial (`online-mode=true`) e loopback. RCON e query ficam desativados. Um servidor novo tem acesso aberto; quem consegue alcançar seu endereço pode tentar entrar, sujeito à autenticação, banimentos e capacidade. A whitelist é uma opção explícita. Configurações existentes são importadas; listas de acesso e operadores não são apagadas. Com whitelist ativa, o dono precisa usar uma conta Minecraft Microsoft verificada, incluída antes do início.

## Um painel, seis áreas

| Área | Controles |
| --- | --- |
| Visão geral | Estado, entrada local, compartilhamento, capacidade e último salvamento confirmado |
| Conteúdos | Mods, datapacks, texturas, mundo, arquivos e pendências; importação de pacotes em Arquivos |
| Jogadores | Online no Minecraft, histórico por UUID, whitelist, operadores, expulsar, banir e desbanir |
| Console | Até 500 linhas por execução, acompanhamento opcional e um comando por envio |
| Configurações | MOTD, dificuldade, modo de jogo, PvP, distâncias, porta, whitelist, salvamento, pacote de recursos, loader e ícone PNG 64 × 64 |
| Backups | Criar, exportar ZIP, restaurar com cópia anterior e remover cópias antigas |

O painel mantém a lista lateral e rola o conteúdo da aba. Os participantes do túnel são launchers conectados; a contagem de jogadores vem do Minecraft. O histórico registra entradas confirmadas após autenticação, sem contar recusas por whitelist. Logs diferentes de um loader podem tornar o histórico incompleto; isso é informado. O envio de um comando confirma o envio ao processo; a resposta do Minecraft e as listas atualizadas indicam seu resultado.

## Mods e modpacks

Downloads e arquivos importados ficam pendentes enquanto o servidor roda. Aplicar exige parar e salvar. O lote é validado antes de substituir arquivos, cria backup e tenta restaurar os arquivos anteriores se houver falha. Mods exclusivos do cliente declarados por Modrinth, Fabric, Quilt ou Forge são rejeitados. A ausência dessa declaração não prova que qualquer mod funciona em servidor.

- **Mods:** Modrinth e CurseForge com chave configurada. O servidor usa versões compatíveis informadas pelo provedor, dependências obrigatórias e integridade dos arquivos. Uma troca de mod prepara a remoção da versão anterior identificada pelo mesmo ID. Duplicações ainda presentes impedem aplicar o lote.
- **Instância instalada:** criar a partir de uma instância copia e aplica os conteúdos locais reconhecidos, mantendo versão e loader. Exclui mods declarados exclusivos do cliente e opções do jogo. A origem permanece intacta. Mundos são importados separadamente para evitar substituir um mundo existente. Mods sem declaração de ambiente exigem conferência de compatibilidade no servidor.
- **Modrinth:** respeita o ambiente `server`, exclui arquivos não suportados/opcionais nessa instalação e aplica `server-overrides` depois de `overrides`. `client-overrides` e arquivos exclusivos do cliente não entram no servidor. Os downloads usam arquivos em disco e quatro transferências simultâneas, com hashes e tentativas limitadas.
- **Pacote ZIP de servidor:** importa os conteúdos reconhecidos, incluindo configurações e recursos de mods. Não executa `.bat`, shell ou instaladores incluídos pelo pacote. Pacotes que dependem desses scripts podem exigir preparação externa; uma pasta de configuração sozinha não é apresentada como servidor pronto.
- **Atualização:** arquivos locais modificados são preservados e informados. O registro dos arquivos do pacote permite retirar versões antigas ainda intactas; alterações locais não são apagadas silenciosamente. Confira a compatibilidade das configurações preservadas antes de iniciar.
- **Datapacks:** entram em `world/datapacks`. Aplicar a alteração exige servidor parado nesta revisão, evitando modificar o mundo durante a execução.
- **Texturas/resource packs:** um ZIP local não vira uma URL pública. A distribuição automática aos jogadores exige URL HTTPS acessível e SHA-1 nas configurações. Shaders pertencem ao cliente.
- **Mundos:** a pasta deve conter um `level.dat` válido. Feche o Minecraft e pare o servidor antes de copiar. Uma versão diferente, quando declarada pelo mundo, é rejeitada. Há backup anterior e o original selecionado é preservado; copiar não converte versões nem substitui os mods necessários.

As fontes oficiais documentam [o formato Modrinth](https://support.modrinth.com/en/articles/8802351-modrinth-modpack-format-mrpack), [o servidor Fabric](https://fabricmc.net/use/?page=server), [o servidor Quilt](https://quiltmc.org/en/install/server/) e [a instalação NeoForge](https://docs.neoforged.net/user/docs/server/).

## Salvar, recuperar e ocupar espaço

O Minecraft mantém seu salvamento nativo. Por padrão, o Luxmc também solicita `save-all flush` a cada cinco minutos; o intervalo pode ser alterado. A interface só registra salvamento confirmado por resposta do servidor. Encerramento forçado mostra uma advertência e não é apresentado como salvamento garantido.

Backups de mundo e conteúdo exigem servidor parado. Bibliotecas, Java, logs e arquivos temporários não entram no ZIP. São permitidas até dez cópias, com limites de arquivos/espaço e verificação de espaço livre quando o sistema o informa. Nenhuma é apagada automaticamente e a última cópia é protegida. Exporte antes de remover cópias antigas. Restaurar cria antes uma cópia do estado atual.

Os dados ficam em `local-servers/<UUID>` dentro da pasta de dados do Luxmc; backups ficam separados em `local-servers/backups/<UUID>`. Eles são conteúdo de jogo, fora da limpeza de cache. A versão do Minecraft permanece vinculada ao servidor. O launcher mantém um servidor gerenciado ativo por vez.

## P2P, LAN e endereço público

Nesta integração, **Compartilhar por P2P** usa a porta real do servidor gerenciado. O amigo conecta pelo Luxmc e recebe seu endereço local de entrada no Minecraft; o código conecta os launchers. O anúncio LAN e a gravação no Multijogador do convidado continuam no fluxo de salas. Não é necessário abrir um mundo de um jogador para LAN quando se usa o servidor dedicado.

**Servidor externo…** é um fluxo distinto: informe a porta efetiva de um processo iniciado fora do gerenciador. O launcher compartilha a conexão e não assume o controle de parar esse processo ou alterar seu mundo. Configure capacidade e autenticação no servidor externo. Portas UDP adicionais, como certos mods de voz, não são encaminhadas pelo túnel TCP.

O fluxo de conta e agente Playit foi retirado da revisão em desenvolvimento. A conexão P2P integrada continua usando o Luxmc nos participantes. Entrada por endereço público em qualquer Minecraft depende de infraestrutura TCP e DNS operados pelo Luxmc, ainda indisponíveis; o site em Cloudflare Pages não substitui essa infraestrutura. Nenhum host, domínio ou plano pago é contratado automaticamente.

Hospedagem e gestão de servidor dedicado exigem **Pro verificado**, com até vinte participantes. Plus mantém o caminho LAN com até cinco participantes; Pro permite até dez na LAN, incluindo o anfitrião. Convidados não precisam de Pro. Se a verificação do Pro falhar, não há liberação automática do dedicado; parar, salvar, fazer backup, exportar e recuperar continuam disponíveis. As regras anteriores do Free para LAN foram preservadas.

O serviço dedicado fica separado da interface: fechar o launcher não encerra o servidor. Use **Parar e salvar** para encerrar a hospedagem. Quando não há servidor ou sessão e a interface não está consultando o serviço, ele encerra por inatividade. O PC hospedeiro precisa permanecer ligado. [Iroh tenta conexão direta e pode usar relay criptografado](https://docs.iroh.computer/concepts/relays); NAT, firewall, upload, disponibilidade do relay, CPU e mods influenciam o resultado. Os testes automatizados usam processos e endpoints isolados, sem iniciar Minecraft do usuário. A partida real entre dois PCs e as combinações de loaders/modpacks ainda precisam de validação humana.

## Revisão de 27/10/2026

A revisão .905 cria servidores independentes por Vanilla, loader ou catálogo de modpacks. Importar arquivos de uma instância instalada continua sendo uma opção avançada posterior, sem ser requisito de criação. Os arquivos são apresentados por categoria; mods declarados exclusivos do cliente são excluídos. Mundos são escolhidos e importados separadamente. Se a cópia falhar, conclua a importação em Conteúdos → Arquivos antes de iniciar. Configurações podem ser salvas com o servidor ligado e entram em vigor no próximo início. A opção de aplicar modo de jogo ao entrar controla jogadores existentes; PvP usa a regra do mundo nas versões recentes. Consulte `docs/validation/server-followup-2026-10-27.md` para as evidências e limites desta revisão local.
