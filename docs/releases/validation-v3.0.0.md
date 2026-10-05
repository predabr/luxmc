A versão 3.0.0 mantém o nome, o identificador do aplicativo e os diretórios de dados do Luxmc.

Verificações locais concluídas antes da publicação:

- `pnpm check`: zero erros e zero avisos.
- `cargo check --locked`: sucesso, com quatro avisos existentes.
- Frontend: 121 testes aprovados.
- Backend: 114 testes aprovados, dois ignorados.
- Site: 29 testes aprovados, incluindo nome do download Windows e requisições parciais.
- Manifesto de atualização: nove testes aprovados, incluindo URLs de rascunho, arquivos ausentes, truncados e URLs externas.
- Navegador: catálogos, conteúdo de instâncias, Vanilla e NeoForge, laboratório, links, skins, funções Windows, hospedagem e armazenamento.
- Tema claro/escuro com e sem wallpaper: cartões, seleção de versão, criação e atualização automática das notícias.
- Workflows: validação com actionlint.

O fluxo de lançamento compila os instaladores das plataformas, confere os arquivos baixados e suas somas SHA-256, gera o manifesto do atualizador e então publica o lançamento. Os executáveis compilados são distribuídos como arquivos do release, sem incluir uma cópia antiga no código-fonte.

Os testes de navegador usam ambientes e contas simulados; o teste de túnel é local. Isso não substitui a validação de multiplayer entre dois computadores em redes diferentes. O site usa o banco existente; a estrutura de contas, avatares e convites foi conferida sem alterar os dados de usuários.
