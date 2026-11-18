# Lux MC Launcher — instalador 3D e revisão geral

Arte refeita com cena Three.js: ilha com terreno em camadas, vegetação, personagem, portal, água, iluminação e sombras. A cena é renderizada em alta resolução e exportada para os bitmaps do instalador NSIS; a interface do instalador usa imagens estáticas. Fontes: `packaging/windows/scene.html`, `scripts/render-installer-art.cjs` e `scripts/build-installer-art.py`. Prévia em `packaging/windows/sidebar-preview.png`.

O arquivo distribuído tem o nome fixo `Lux MC Launcher.exe`, sem versão e sem a palavra Instalador. `scripts/build-windows.mjs` mantém esse nome nas próximas compilações e resolve o artefato interno pela versão atual da configuração.

Correções encontradas na revisão:

- Atualizações do Windows selecionavam o primeiro executável da lista, inclusive um portátil. O launcher agora exige um instalador reconhecido ou MSI para atualização automática; reconhece também o nome fixo solicitado. O portal dá prioridade ao instalador sobre executáveis portáteis.
- Falhas na consulta das versões podiam resultar numa mensagem incorreta de versão atualizada. Consultas agora têm limite de tempo e só mostram confirmação quando a consulta teve sucesso. Antes da consulta aparece a versão instalada; na falha aparece Verificação indisponível.

Verificações concluídas: 96 testes do frontend, 100 testes do backend, 24 testes do portal, `pnpm check` sem erros ou avisos e `cargo check` com quatro avisos já existentes. Fluxos de catálogo e instalação de conteúdo, skins com sincronização e recuperação, sala P2P e configurações Windows passaram nos testes de navegador com backend simulado. O teste Rust de transporte criptografado passou; um teste que depende de relay público ficou ignorado. Não foi feita uma partida multiplayer entre dois computadores.

O teste de navegador da atualização também passou: consulta falha sem confirmar versão atualizada, recuperação da consulta e escolha do instalador com nome fixo, em vez do executável portátil.

O portal com a prioridade de instaladores foi publicado em `https://e9f2acba.luxmc-r92.pages.dev`, disponível pelo endereço habitual `https://luxmc-r92.pages.dev`.

Build Tauri de produção concluído em perfil release otimizado. Artefato final: `C:/Users/preda/Downloads/Lux MC Launcher.exe`, 19.215.583 bytes, SHA-256 `E1E1C5B80D8E2470A61B92E379AF48D50333FC12D7AC280A1F12B9A93A2CBBA3`. Reinstalação silenciosa real concluída com código 0. Atalhos, executável e desinstalador foram verificados. Os hashes do banco de instâncias, auth.json e settings.json permaneceram iguais antes e depois da reinstalação. Resultado em `installer-3d-delivery.json`.

Evidências estão nos arquivos `general-review-*.log` e `installer-art-render.json`. Os testes são verificações das áreas listadas e não garantem ausência de qualquer outro erro. O pacote continua sem assinatura digital.
