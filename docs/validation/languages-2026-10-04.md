# Idiomas do Luxmc — 4 de outubro de 2026

O launcher resolve variantes de português (incluindo Portugal) para pt-BR, variantes de espanhol para es e demais idiomas para inglês. No Windows, lê o idioma de interface do usuário com GetUserDefaultUILanguage e LCIDToLocaleName. Em caso de indisponibilidade da integração nativa, usa o idioma do navegador.

O modo automático é o padrão de instalações novas. A escolha manual já salva é preservada; o botão para usar o idioma do sistema restaura o modo automático. Preferências de idioma da conta online não substituem a detecção enquanto esse modo está ativo.

Textos de interface foram integrados aos catálogos pt-BR/en/es, com fallback em inglês e parâmetros de interpolação preservados. Datas e números usam o idioma selecionado. Os recursos são distribuídos localmente, sem serviço externo de tradução durante o uso.

Validação: 119 testes unitários passaram; svelte-check sem erros nem avisos; cargo check passou com quatro avisos preexistentes. Os testes de navegador cobrem sete cenários de idioma, escolha manual e retorno ao modo automático. Também passaram os fluxos de catálogo, aparência, P2P e atualizador. A compilação e a entrega estão registradas nos arquivos languages-windows-build.log, languages-delivery.json e languages-native-verify.json.

Referências da implementação Windows: https://learn.microsoft.com/windows/win32/api/winnls/nf-winnls-getuserdefaultuilanguage e https://learn.microsoft.com/windows/win32/api/winnls/nf-winnls-lcidtolocalename.

Entrega concluída: compilação release e NSIS passaram; instalação retornou 0 e preservou hashes de banco, contas e configurações. O executável instalado leu pt-BR diretamente do Windows, inicializou o banco, retornou uma conta e duas instâncias e persistiu a aparência atual sem erro; integrity_check permaneceu ok. Instalador atualizado em C:\Users\preda\Downloads\Lux MC Launcher.exe (19.330.209 bytes, SHA-256 382C2E655D7C4244DC30A873D449AF35BA9BCEC9293AFEF1B8BEF619441756DC).
