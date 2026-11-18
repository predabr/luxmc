# Próximos diferenciais do Luxmc

Propostas, não funcionalidades implementadas nesta revisão. A prioridade considera o projeto atual e os problemas relatados pelo usuário.

## Diferenciais

1. **Preparar para a sala em um clique.** A partir do diagnóstico P2P existente, criar uma instância separada com as versões exatas dos mods exigidos pelo anfitrião. Mostrar o plano antes de baixar, obter arquivos dos provedores autorizados e manter os mundos pessoais intactos. Autenticação continua sendo requisito independente da conexão.
2. **Planejador de atualização de modpacks.** Antes de atualizar, mostrar mods adicionados/removidos, dependências alteradas, configurações afetadas e riscos para mundos. Aproveitar backups e cápsulas existentes para oferecer retorno simples, com uma linha do tempo das alterações.
3. **Recomendação de packs baseada em medidas.** Usar tempos de preparação e memória já coletados para explicar quais packs cabem no PC. Comparações precisam considerar versão, quantidade de mods e hardware equivalente. Compartilhamento de medidas seria opcional, sem identificadores pessoais.
4. **Coleções públicas com instalação precisa.** Evoluir favoritos do perfil para coleções com IDs reais dos provedores, versão escolhida, descrição e instruções do criador. Abrir exatamente o projeto recomendado, evitando buscas por nomes ambíguos.
5. **Mapa interativo de dependências.** Ao instalar ou remover um mod, mostrar quais outros dependem dele, por que a dependência existe e o efeito da ação. Integrar isso às mensagens de compatibilidade atuais.
6. **Jogar em grupo com preparação coordenada.** Convite mostra o progresso de preparação dos participantes que consentirem, versões compatíveis e quem está pronto. Útil para evitar começar uma sessão enquanto alguém ainda baixa centenas de arquivos.

## Melhorias prioritárias

1. Medir instalação fria e abertura com cache em packs representativos. Exibir separadamente download, conferência, extração, loaders e inicialização Java; otimizar a fase realmente lenta.
2. Testar P2P entre redes e PCs distintos, com reconexão, troca de mundo, suspensão do Windows e firewall. Os testes locais de túnel não substituem esse cenário.
3. Criar uma matriz de regressão por versão/loader/Java e testes de atualização interrompida em máquinas virtuais, sem usar dados pessoais.
4. Organizar uma auditoria de teclado, leitor de tela, escala de 125–200% e telas menores. Controles visuais precisam continuar utilizáveis nessas condições.
5. Medir CPU, memória, disco e atividade de rede com o launcher oculto, sem downloads, com wallpaper e com P2P; definir limites a partir desses dados.
6. Reforçar observabilidade do atualizador: arquivo instalado, hash, origem, etapa atual e motivo de recuperação, com relatório local sem tokens.

## Ordem recomendada

Primeiro estabilidade e medições. Depois preparação automática para salas e planejamento de atualizações. Em seguida coleções e recursos sociais adicionais.

Exportação de packs e cópia de instâncias já existem em outros launchers, então o diferencial deve ser a qualidade do fluxo integrado. Referências consultadas: [Modrinth: modpacks](https://support.modrinth.com/en/articles/8802250-modpacks-on-modrinth) e [Prism: cópia de instância](https://www.prismlauncher.org/wiki/help-pages/instance-copy/).
