# Transparência com wallpaper — 4 de outubro de 2026

Causa: no-blur substituía os fundos bg-bg/35 por alfa 0,94 e barras laterais por 0,96. A regra configurável do wallpaper cobria os cards do catálogo, mas deixava de fora os painéis bg-bg/35 do início e as barras laterais. Isso criava regiões opacas no mesmo wallpaper.

Correção em src/app.css: o fallback sólido de no-blur agora vale somente sem wallpaper personalizado. Painéis bg-bg/35, barras laterais, controles, cards de notícias, navegação e grupos de configurações usam --interface-opacity quando há wallpaper. O fundo principal mantém os tokens de tema; o alfa embutido antigo em --bg-overlay foi retirado para preservar a sintaxe dos tokens. O desfoque e a transparência são independentes. Diálogos e listas suspensas mantêm as regras de legibilidade existentes.

Validação: build do frontend passou. Cinco cenários de navegador passaram: imagem/vídeo com desfoque ligado ou desligado, e vídeo com modo de desempenho. Cada cenário verificou oito painéis do início/barras laterais, alteração de 18% para 45% pelo controle de Aparência, transparência do catálogo, texto sem redução de opacidade e ausência de erros de página. Vídeos reproduzem normalmente e são pausados no modo de desempenho conforme o comportamento existente. O fallback sem wallpaper foi verificado. Evidências em wallpaper-transparency-browser.log e wallpaper-transparency-home.png.

Entrega concluída: build release e NSIS passaram. Instalação retornou 0 e preservou hashes do banco, contas e configurações. Launcher atualizado reaberto; instalador em C:\Users\preda\Downloads\Lux MC Launcher.exe, SHA-256 F0D4691C740704FBC66EE06D2D4A8206A06B816FEBC5177EFA9A25F78FD7FDE8.
