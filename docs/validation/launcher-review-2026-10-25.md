# Revisão do launcher — 25/10/2026

## Implementação

- NameMC abre diretamente na URL oficial; a navegação deixa de depender de about:blank e de uma chamada COM após instalar o filtro. Janela existente recebe nova navegação ao reabrir. Filtros cosmético e publicitário preservados. Validação real da janela ainda pendente neste registro.
- Primeiro preparo do Minecraft separado do fluxo de reparo do modpack. O pipeline normal continua validando e baixando arquivos ausentes/corrompidos. Reparo explícito e bloqueios por conflitos continuam ativos. Duas verificações independentes passam a rodar em paralelo; reconciliação não grava mod_count quando o valor não mudou.
- Configurações com navegação mais espaçada, textos sem aperto e painéis maiores. Lista de mods usa cartões sólidos, capas maiores, status traduzido, tamanho em KB para arquivos pequenos e controles com espaço próprio. Virtualização preservada. Novas animações finitas de entrada e seleção respeitam movimento reduzido.
- As primeiras doze capas locais podem vir na própria resposta da árvore de conteúdo, sem aguardar outro round trip ou consulta remota. Demais capas conservam carregamento e cache com limites de trabalho.
- Aurora e Sinal luminoso são auras Pro opcionais em profileStyle.ambientEffect. A lista antiga de effect permanece igual, para que clientes antigos continuem lendo os perfis. Salvamento antigo preserva a aura; expiração oculta sem apagar. Miniaturas estáticas só animam sob interação. Benefícios publicados no site PT/EN/ES; deploy 5870f7c8.
- Legacy Fabric e Babric têm seleção de versões e preparação pelos metadados oficiais. Nativas específicas do loader são baixadas pelo classificador do sistema e extraídas numa pasta separada, com cache e sem modificar as nativas originais. Loaders desconhecidos não viram Vanilla silenciosamente. Versão de Minecraft e família do mod permanecem critérios de compatibilidade.
- Modrinth e CurseForge continuam integrados. FTB, Technic e ATLauncher aparecem como catálogos externos, abertos por comando tipado com lista fixa de URLs. Não são apresentados como provedores de instalação/atualização integrada.

## Verificação concluída

- pnpm check: zero erros/avisos; cargo check Windows aprovado, seis avisos preexistentes.
- 230 testes frontend e 127 de site aprovados.
- Dois testes dos novos loaders aprovados, incluindo download real de bibliotecas dos endpoints Legacy Fabric e Babric. Regressões adicionais: um teste do Fabric, dois do Quilt e um de compatibilidade de mods aprovados.
- Teste de nativas aprovado: isolamento, cache sem regravar, recuperação de arquivo removido e rejeição de caminho relativo malicioso.
- Navegador: configurações em 1440/1000, mil mods em 1440/960, primeiro preparo/reparo/bloqueio, aparência PT/EN/ES em dois temas e tela compacta, auras Pro/Plus/movimento reduzido aprovados.
- Produção /planos, /i18n.js e /profile-signature.css respondem 200 com os benefícios e estilos novos.

## Pendente até conclusão da instalação

Instalador 4.0.0 local build 40133561063.1 em compilação. Não houve publicação de binário no GitHub nesta revisão. Backup atual concluído: dez perfis e 1.827 arquivos protegidos. Conferir snapshot antes/depois da instalação, testar abertura real do NameMC e só então registrar resultado. Não houve partida completa com os loaders novos nem novo benchmark até o menu dos modpacks; preparação de bibliotecas não comprova compatibilidade de todo mod de terceiros.

## Referências primárias

- https://meta.legacyfabric.net/
- https://meta.babric.glass-launcher.net/
- https://docs.modrinth.com/api/operations/loaderlist/
- https://docs.curseforge.com/rest-api/
- https://www.feed-the-beast.com/blog/p/ftb-api

## Conclusão da instalação

Build 40133561063.1 instalada com /S /UPDATE, saída zero. Comparação de dados aprovada: dez perfis, contas, identidades, mundos/arquivos protegidos, wallpapers e 26.305 arquivos de instâncias preservados. Binário corresponde à compilação após normalização em memória do marcador Tauri. Controle de janela foi interrompido pelo usuário com Escape; em seguida ele confirmou que o NameMC abriu. Nenhuma partida com os novos loaders foi executada.
