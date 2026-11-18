# Instalador Luxmc para Windows

Instalador NSIS x64 da versão 2.0.2, com imagens exclusivas de identidade visual, ícone do Luxmc, telas de boas-vindas e conclusão em português brasileiro, escolha de pasta, atalhos no desktop e menu Iniciar, registro na lista de aplicativos e desinstalador. O bootstrapper oficial do WebView2 está incluído; instalar o runtime ausente exige internet. Java e Minecraft continuam sendo preparados pelo launcher para cada instância.

Geração: `pnpm build:windows`. As imagens estão em `packaging/windows` e podem ser regeneradas com `python scripts/build-installer-art.py`. Para empacotar um executável de produção já compilado, use `pnpm tauri bundle --bundles nsis --features tauri/custom-protocol` com o mesmo `CARGO_TARGET_DIR` utilizado para compilá-lo.

Verificação: `cargo check` passou com quatro avisos já presentes. Empacotamento NSIS concluído. Instalação silenciosa real neste Windows retornou código 0, com atalhos, registro versão 2.0.2 e desinstalador presentes. O executável instalado tem o mesmo tamanho do executável de produção e difere somente em três bytes do marcador interno de empacotamento Tauri, de `UNK` para `NSS`. O empacotamento não recompilou nem alterou funcionalidades do launcher.

Dados e resultado estruturado: `windows-installer-2026-10-04.json`. A desinstalação não foi executada e as telas interativas não foram percorridas por automação nesta verificação. As imagens foram inspecionadas visualmente antes do empacotamento. O instalador permanece sem assinatura digital; o empacotamento não garante aprovação pelo SmartScreen ou Controle Inteligente de Aplicativos.

Referência técnica: [instaladores Windows do Tauri](https://v2.tauri.app/distribute/windows-installer/) e [configuração NSIS](https://v2.tauri.app/reference/config/#nsisconfig).
