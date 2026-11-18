# Catálogo, personalizador e site — 4 de outubro de 2026

## Correções

- A seleção de instância inclui modpacks para mods, resource packs, shaders e datapacks. Metadados resumidos de busca não descartam instâncias por uma lista incompleta de versões. Antes da instalação, os arquivos disponíveis são consultados pela versão exata de Minecraft e pelo loader; incompatibilidades impedem a instalação.
- World usa a categoria de mapas do CurseForge. O backend do Modrinth retorna vazio para World, evitando a queda silenciosa para mods.
- Datapacks exigem um mundo existente escolhido pelo usuário. O destino é `saves/<mundo>/datapacks`, com validação do diretório e de `level.dat`.
- O relógio de animação não reutiliza tempo já consumido; pausas longas recebem um passo limitado. Camadas transparentes da skin não escrevem profundidade, preservando cores e reduzindo artefatos durante o movimento.
- O site recebeu uma apresentação interativa de biblioteca, mods e personalizador com capturas reais do launcher, layout responsivo e traduções em português, inglês e espanhol.

## Validação concluída

- `pnpm check`: zero erros e zero avisos.
- `pnpm build`: aprovado.
- `cargo check`: aprovado, com três avisos preexistentes.
- Build Windows com `tauri/custom-protocol`: aprovado. Frontend de produção incorporado; perfil de compilação `dev`.
- Nove testes unitários de compatibilidade e relógio: aprovados, incluindo 60, 90, 120 e 144 Hz.
- Teste nativo de World: aprovado; nenhum pedido de mapas vira busca de mods.
- Teste nativo de datapacks: aprovado; apenas o mundo escolhido recebe o diretório e nomes inválidos são rejeitados.
- Teste de navegador das quatro categorias: aprovado. Instância de modpack visível, arquivo Fabric escolhido em vez de Forge, mundo escolhido para datapack e World encaminhado ao CurseForge.
- Teste gráfico do personalizador: aprovado. Textura original preservada, 60.558 de 60.560 pixels opacos fiéis à textura, animação durante jogo e rolagem, 12 capas e recuperação após erro.
- Teste do site local e publicado: três capturas carregadas, seleção interativa funcionando, sem erros JavaScript e sem excesso de largura em 390, 768 e 1440 pixels.
- Publicação Cloudflare Pages concluída: `https://b0ae8a81.luxmc-r92.pages.dev`, acessível pelo domínio existente `https://luxmc-r92.pages.dev`.
- Rota publicada de convites conferida: código inexistente retorna HTTP 404 JSON, não HTML.

## Entrega e limites

Executável: `C:\Users\preda\Downloads\Luxmc-2.0.2-catalogo-skins.exe`. O hash e o tamanho estão em `catalog-delivery.json`. Subsistema gráfico Windows 2, sem console principal.

O controle nativo de mouse não iniciou nesta sessão por falha do helper de automação Windows. A tentativa de abrir o novo executável foi rejeitada pela política automática; nenhuma proteção do Windows foi alterada. A interface foi validada em navegador com respostas nativas simuladas e o backend nos testes Rust; isto não equivale a percorrer todas as telas do aplicativo nativo.

Uma partida P2P real entre dois computadores ainda precisa de validação com o convidado. Os testes anteriores de túnel e sala não permitem prometer funcionamento universal de redes ou modpacks.
