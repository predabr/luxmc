# Modpacks e texturas — revisão local .906

## Correções

O importador de servidor passou a excluir bibliotecas declaradas exclusivas do cliente e arquivos de apoio não JAR dentro de mods. O relatório mantém os nomes excluídos. A importação avulsa de um mod exclusivo do cliente continua sendo rejeitada. Integridade, hashes, limites e caminhos seguros continuam validados.

Selecionar Modpack abre a busca inicial automaticamente. Conteúdos ganhou ícones, contadores e ativação de ZIP de texturas importado. O pacote é entregue por HTTP somente no loopback do computador; participantes P2P recebem os bytes pelo túnel QUIC autenticado e os expõem localmente ao Minecraft. Limite de 128 MiB; os participantes precisam desta revisão do Luxmc. O jogador confirma o download dentro do jogo.

Proteção de spawn, voo, command blocks e tempo máximo de inatividade são persistidos nas propriedades reais do servidor. As opções de compatibilidade Java são condicionadas à versão; os outros avisos do jogo continuam visíveis.

O carregamento de capas consulta metadados conhecidos antes de abrir JARs grandes. Downloads de servidor respeitam o limite salvo e reutilizam arquivos pequenos do cache compartilhado somente após conferir hash e tamanho.

## Evidências

- svelte-check: 0 erros e 0 avisos.
- Frontend: 232 testes aprovados.
- Rust: 345 aprovados, 16 ignorados; cargo check aprovado.
- Interface: 1440, 1366 e 960 pixels em tema escuro; 1440 em tema claro. Produção também passou nos três tamanhos, sem erros de execução.
- Importação real em pasta temporária: Vanilla Perfected 163 aplicados/62 excluídos; Aged 2320 aplicados/46 excluídos. Pixelmon 571 aplicados/1 excluído na repetição final.
- Transferência de texturas testada por HTTP e entre dois processos através de QUIC criptografado.

Os testes reais de pacotes verificam importação, aplicação e validação das bibliotecas. Não iniciam Minecraft/Java nem demonstram que todos os mods inicializam juntos. Não alteram as instâncias do usuário.

## Entrega

Revisão .906 instalada e aberta em 28/10/2026, às 08:57 (Brasília). Fonte congelada: 538 arquivos. Payload NSIS conferido; backup backup-20261028-085748; 2 contas, 4 perfis, 17 tabelas e 60 arquivos protegidos preservados. Smoke do serviço separado confirmou revisão 40255548682906; catálogo nativo retornou 100 versões e bloqueou instalação sem Pro verificado. Nenhum servidor ou Minecraft foi iniciado. Recibos locais: packs-ready.json, packs-installation.json e packs-delivery.json na pasta privada de build.

O endereço público para clientes externos continua dependendo de infraestrutura TCP e DNS. Esta revisão não cria um domínio público, não contrata infraestrutura e não reintroduz o Playit.
