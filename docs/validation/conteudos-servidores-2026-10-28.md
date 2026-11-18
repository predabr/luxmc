# Conteúdos de servidores — revisão .907

## Erro de criação

A aplicação dos arquivos já removia o marcador de preparação. A finalização do catálogo tentava removê-lo novamente, retornando erro de arquivo inexistente mesmo depois de aplicar o pacote. A limpeza passou a tolerar somente ausência do arquivo, preservando os outros erros. O teste de regressão cobre marcador já removido, limpeza repetida e falha real de acesso a um diretório.

Recibos antigos do mesmo modpack podem ser reparados pelo fluxo de catálogo sem criar outro servidor. A reaplicação mantém o controle de arquivos e configurações locais existentes.

## Conteúdos

A lista usa cards com nome, capa, arquivo, tamanho, estado, busca e páginas de 24. Metadados conhecidos consultam os projetos do Modrinth; arquivos sem esse registro usam os metadados e a imagem do próprio pacote quando disponíveis. Consultas não repetem continuamente ao atualizar a capa.

Ativar e desativar mods e datapacks altera o sufixo .disabled, com servidor parado, sem importações ou pendências e com backup. Exclusão exige confirmação e é aplicada imediatamente com o servidor parado; durante execução permanece pendente. O botão Arquivos usa largura adequada ao texto.

Texturas incluídas no modpack são preservadas mesmo quando declaradas como conteúdo do cliente. Pastas válidas de texturas são empacotadas como ZIP, incluindo seu pack.mcmeta; datapacks são colocados em world/datapacks. Isso se aplica a Modrinth e ZIP de servidor. Um único ZIP elegível pode ser selecionado automaticamente. Uma textura opcional inválida é informada como aviso, sem impedir a criação do servidor inteiro.

Os cards têm animação de entrada e destaque ao passar o mouse, respeitando a preferência por movimento reduzido.

## Endereços

O proprietário pode copiar localhost:porta e acessar a configuração de porta. O nome do servidor no Luxmc não cria um domínio na internet. A entrada pública com um nome personalizado continua dependendo de infraestrutura TCP e DNS sob controle do projeto; esta revisão não contrata essa infraestrutura.

## Validação e entrega

Fonte congelada em 540 arquivos. Svelte sem erros e sem avisos, cargo check aprovado e 232 testes frontend aprovados. A rodada final Rust aprovou 348 testes, com 16 ignorados. Interface de produção aprovada em 1440, 1366 e 960 pixels; tema claro também aprovado. Importação real em pasta temporária aprovada: Pixelmon 571 aplicados/1 excluído; Vanilla Perfected 163 aplicados/62 excluídos; Aged 2320 aplicados/46 excluídos. Os testes de importação usam diretórios temporários e não iniciam Minecraft ou Java.

Revisão .907 instalada e aberta em 28/10/2026 às 12:36 (Brasília), após o usuário encerrar o aplicativo. Executável conferido contra o payload NSIS. Backup backup-20261028-123612 preservou 2 contas, 4 perfis, 17 tabelas e 60 arquivos protegidos. Serviço separado confirmou revisão 40255548682907 e catálogo retornou 100 versões. Nenhum servidor ou Java foi iniciado. Recibos contents-ready.json, contents-installation.json e contents-delivery.json na pasta privada de build.
