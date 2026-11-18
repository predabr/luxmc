# Arte das instâncias — Vanilla Perfected

Causa: a reconstrução do banco após a corrupção conservou o nome e os arquivos do modpack, mas recuperou o ícone como grass_block. A seleção de banners também tratava qualquer nome contendo Vanilla como Minecraft comum.

Correção: logo oficial VP incluída localmente e restaurada na instância existente pelo comando profiles_update. Banner oficial incluído no catálogo de arte; somente loader vanilla recebe o fallback de Minecraft Vanilla. Modpacks que incluem Vanilla no nome preservam sua própria imagem. Banner personalizado do usuário continua tendo prioridade.

Origem dos arquivos: https://modrinth.com/modpack/vanilla-perfected e https://api.modrinth.com/v2/project/vanilla-perfected. Metadados consultados e URLs originais em vanilla-perfected-artwork-source.json.

Verificação: svelte-check sem erros ou avisos; 121 testes passaram. Teste de navegador confirmou a logo no início e na barra lateral, o banner próprio na instância, ausência do banner genérico e carregamento completo das imagens. A restauração nativa verifica que somente icon e updatedAt mudam, que o valor permanece salvo e que a integridade do banco fica ok. Build e entrega registrados em modpack-artwork-windows-build.log e modpack-artwork-delivery.json.

Entrega concluída: instalação retornou 0 e preservou o banco, contas e configurações. A restauração via profiles_update alterou somente icon/updatedAt da instância confirmada pelo manifesto, manteve os demais campos e passou integrity_check. O launcher atualizado foi reaberto e está respondendo. Instalador em C:\Users\preda\Downloads\Lux MC Launcher.exe, SHA-256 E82201B29F00E7CBA488B197C08B7B52BF51D3F677502618066568F3810A8B33.
