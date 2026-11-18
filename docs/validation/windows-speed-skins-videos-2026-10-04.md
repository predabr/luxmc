# Windows, skins e vídeos — 4 de outubro de 2026

## Implementação

- Preset automático Windows usa G1GC, processamento paralelo de referências e deduplicação de strings; memória inicial entre 256 e 1024 MB, com o limite máximo da instância preservado. O preset automático não usa AlwaysPreTouch nem ZGC. Argumentos personalizados continuam com prioridade.
- Aberturas com integridade preservada consultam os arquivos locais; se a validação ou os marcadores de assets falharem, o caminho normal de instalação e reparo é mantido.
- Não há download automático de shaders antes de iniciar Minecraft. Uma sessão Microsoft válida por mais de dez minutos não é renovada em cada abertura.
- Perfis Fabric de versões fixas são armazenados em disco, com bibliotecas ainda validadas e cache inválido descartado.
- Detecção de GPU e análise de mods usam tarefas próprias para não ocupar o executor assíncrono. A preferência de GPU dedicada contempla `java.exe` e `javaw.exe`.
- Windows deixa de reduzir repetidamente o working set do launcher enquanto o jogo está aberto ou durante os downloads de mods.
- Renovação Microsoft, inicialização do aplicativo e novo login Luxmc preservam a aparência local aplicada. A seleção aplicada conserva a identidade da capa entre reaberturas.
- “Aplicar e sincronizar com Microsoft” é uma única ação: salva PNG localmente, envia a skin ao perfil oficial e permite tentar novamente quando a Microsoft falha.
- Troca de conta durante a aplicação não envia a skin para outra conta.
- Botões de adicionar skin e capa foram ampliados. Launcher e site distinguem aparência local de conta Luxmc, skin oficial Microsoft e capas importadas locais.
- Detalhes do catálogo aparecem antes da consulta de todas as versões e são reutilizados por até cinco minutos, com limite de 30 projetos.
- Vídeos incorporados usam o player HTTPS do portal, com Referrer correto e domínio permitido pela CSP do launcher. Vídeos diretos conservam controles de reprodução.

## Testes

- `pnpm check`: zero erros e zero avisos; `pnpm build`: aprovado.
- `cargo check`: aprovado, com quatro avisos; um deles é o helper de shader automático agora sem chamadas.
- Cinco testes Rust passaram, incluindo limites do heap, preservação de skin na renovação e perfil Fabric fixo disponível sem rede.
- Java 8, 17, 21 e 25 instalados aceitaram o preset com heap de teste de 512 MB. Isto valida argumentos, não mede FPS.
- Teste de importação de skin: PNG original preservado, 12 capas, fidelidade de cores e animação durante jogo e rolagem aprovadas.
- Teste Microsoft com respostas simuladas: salvar antes de enviar, manter PNG ao falhar, repetir envio, restaurar capa após reload e botões maiores aprovados; zero erros da interface. Nenhuma skin de uma conta real foi enviada neste teste.
- Teste de vídeo: descrição exibida antes da lista de versões, MP4 avançando e pedido real ao embed YouTube com Referer `https://luxmc-r92.pages.dev/`.
- Player publicado: HTTP 200, política de Referrer correta, sem X-Frame-Options impedindo o launcher, CSP restrita. Identificadores de vídeo inválidos retornam HTTP 400.

## Limites

Os testes não medem o tempo de abertura de uma partida real nem garantem aumento de FPS. Hardware, mods, shaders, Java e configurações do mundo continuam influenciando o resultado.

Conta Luxmc mostra a aparência no cliente iniciado pelo launcher; não faz clientes externos receberem automaticamente skins personalizadas. Microsoft publica a skin oficial; capas importadas não são publicadas como capas oficiais.

A configuração do player não contorna restrições do autor, região, idade ou disponibilidade de um vídeo YouTube. O link para abrir no navegador permanece disponível.

O executável entregue e seu hash constam em `windows-speed-delivery.json`.

## Referências consultadas

- [Oracle Java 21 — opções da JVM](https://docs.oracle.com/en/java/javase/21/docs/specs/man/java.html): AlwaysPreTouch realiza antecipadamente o trabalho de tocar as páginas do heap. O preset Windows evita essa etapa na inicialização.
- [YouTube — IFrame Player API](https://developers.google.com/youtube/iframe_api_reference): erro 153 indica ausência de Referer HTTP ou identificação equivalente do cliente.
