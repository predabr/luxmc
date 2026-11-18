# Capas, compatibilidade e fluidez — 25/10/2026

Pedido posterior ao cancelamento da release: remover Mais catálogos, corrigir capas cortadas/invisíveis/lentas e barra de salvar sobrepondo campos, preservar fluidez e conferir o loader dos pacotes. Publicação permanece cancelada; somente instalador local para o usuário testar.

Alterações:
- Retirados FTB, Technic e ATLauncher da interface de busca. Modrinth e CurseForge permanecem.
- Capas quadradas no catálogo, nos cartões de instância e no destaque da instância usam imagem inteira e espaçamento; paisagens continuam preenchendo o banner. Removida ampliação de hover no cartão de instância/catalogo; logos não recebem parallax no destaque.
- Ícones de mods recebem margem interna. Recuperação de imagens restabelece visibilidade ao trocar a origem e reconhece imagens já carregadas. A mesma imagem não repete a animação a cada remontagem de linha virtualizada.
- Linhas virtualizadas mantêm feedback de interação sem repetir entrada ao rolar.
- Miniaturas persistidas são consultadas antes de esperar a fila de rede; conversão de imagens em lotes usa duas tarefas limitadas. Falhas de ícones expiram em dez minutos, inclusive na memória, em vez de bloquear a tentativa por um dia/sessão inteira.
- Salvar/Cancelar ficam no fluxo depois dos campos, eliminando a sobreposição provocada por position:sticky.
- O loader selecionado na busca acompanha a instalação e a retomada da fila de modpacks. A versão escolhida precisa declarar esse loader. Instalação de mods reutiliza a comparação de famílias Fabric/Legacy Fabric/Babric e fallback Fabric em Quilt; versões de Minecraft e validação nativa permanecem obrigatórias.

Verificação: pnpm check zero erros/avisos; cargo check aprovado com seis avisos preexistentes. Sem navegação na instalação do usuário, partida, benchmark ou novas suítes de testes, conforme orientação anterior. Não promete que arquivos sem logo passem a ter uma capa oficial nem compatibilidade universal de mods. Não altera mundos ou executa updates automáticos nos modpacks existentes.

Build local 4.0.0 40133561065.1 em compilação; usuário confirmou encerramento pela bandeja. Aplicar somente após hash e cópia de segurança atual, conferir dados antes/depois e abrir para ele testar. Sem commit/push/deploy/release desta correção.

## Interrupção pelo usuário

Usuário vai desligar o PC. Compilação interrompida antes da conclusão; nenhum instalador desta revisão aplicado ou publicado. Código salvo e checagens aprovadas. Retomar somente após novo pedido. Backup atual contém dez perfis e 1.827 arquivos protegidos.
