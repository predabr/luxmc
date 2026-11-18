# Luxmc 4.0 — site, mundos e distribuição — 20/10/2026

Fonte do instalador: `e224c2315aa5ecee49ec4e3396959dee74cc67d6`. Revisão adicional apenas do site: `8b8af43f6fcb7296b6b41735a340eb89358431b7`. Versão do produto preservada em `4.0.0`.

## Correções

- Capas: recuperação nativa limitada aos CDNs autorizados e miniatura persistida; notificação da pré-carga para a lista aberta e retenção de imagem positiva diante de resposta negativa antiga.
- Listas: atualização por chave sem recriar todos os itens a cada lote. Abrir Mods não varre mundos.
- Mundos: leitura da aba selecionada, renovação no foco e saída do jogo, atualização posterior preservada quando há uma leitura em andamento.
- Arquivos: leitura única por navegação e descarte de resposta de um diretório já abandonado.
- Site: correção do número selecionado do tour; contraste medido de 8.12:1 nos quatro cartões, em desktop e celular, antes 1:1.
- Movimento: destaque inicial, iluminação dos cartões, resposta de botões, transição do tour e FAQ, teclado e antecipação limitada das imagens. Pausa persistente, redução de movimento e suspensão fora da aba preservadas. Controlador compilado de 7.811 bytes, sem nova dependência.
- Distribuição: `If-Range`, ETag e Last-Modified preservados; arquivo alterado requer resposta completa. `WINDOWS_LOCAL_BUILD` permite comparar a compilação hospedada com a release atual sem mudar o número do produto.
- Identidade do site: sufixo da compilação aparece na identificação detalhada; a versão visível permanece `4.0.0`.
- Navegação móvel: textos traduzidos não deslocam o menu para fora da tela, e os controles de idioma dentro do menu permanecem visíveis. Os quatro cartões, seus nomes acessíveis e a chamada para os planos acompanham o idioma escolhido.

## Verificações concluídas antes da publicação do instalador

- `pnpm check`: zero erros e avisos; TypeScript do site e compilação do site aprovados.
- Site: 119 testes aprovados; cinco scripts de navegador aprovados (movimento, cartões/teclado, idiomas, identidade de release e sessão/conta). A troca real de idioma e os controles móveis foram conferidos em 1440, 390 e 320 px.
- O teste de limite de tentativas do dono tinha dependência do relógio real e falhava ao atravessar a virada do minuto. A reprodução confirmou o problema; o teste agora controla o relógio e confere bloqueio e renovação do prazo. As regras do servidor não foram alteradas.
- Site público: HTML, CSS e scripts conferidos por SHA-256 após deployment `55c2b4ca`; teste de interação executado no endereço público, desktop e celular. A revisão adicional `ff532bcc` também teve os cinco arquivos alterados conferidos por SHA-256 e os 24 cenários de idioma/cabeçalho passaram no endereço canônico. As chamadas autenticadas foram simuladas, sem modificar contas reais. Download público respondeu 206 para trecho válido e 200 quando If-Range mudou.
- Launcher: nova regressão de leitura concorrente e testes de mundos/capas e mil mods aprovados. Os 20 scripts restantes da sessão interrompida foram concluídos. O teste de aparência foi repetido em Vite novo após invalidação por HMR de compilação concorrente, passando.
- Rust: 268 testes passaram na execução completa, nove ignorados existentes. Um teste externo de CurseForge falhou por DNS do CDN e passou na repetição isolada, sem alteração de código.
- Manifestos e assinatura de release: 16 testes aprovados.
- Cópia privada consistente de banco, configurações e mundos das oito instâncias, mantida fora do repositório e dos arquivos publicados.

A busca exata por Jenny Mod não identificou uma publicação verificável nas fontes do catálogo. Nenhum binário alternativo ou alias de outro projeto foi adicionado.

## Publicação e instalação

[Execução oficial de release](https://github.com/predabr/luxmc/actions/runs/39440259091) concluída com os seis jobs aprovados. [Release publicada](https://github.com/predabr/luxmc/releases/tag/v4.0.0-build.39440259091.1) às15:31:10UTC, sem sobrescrever a release antiga. Manifestos Ed25519 e instalador Windows conferidos; SHA256 do instalador `5f123006ed3898fa5c99b97a0d3dfd8f0832e5c9553d58799f710e7cc681edd3`. O Defender do CI aprovou o programa e o instalador, que permanecem sem Authenticode.

A atualização local oficial terminou às15:38:27UTC com cópia nova e verificação de preservação: oito perfis,182 arquivos dos mundos,15 arquivos pessoais protegidos e25.820 arquivos das instâncias. O executável corresponde ao hash oficial, com a única diferença conhecida de três bytes do marcador Tauri comprovada por SHA256 do arquivo completo. Launcher deixado fechado.

A checagem pública detectou que o novo asset estático do Pages não respondia, bloqueando a espera local sem prazo. A rota funcional via GitHub foi restaurada no deployment `4d288277`: HEAD200 com21.633.575bytes e GETRange206 com1024bytes. O tratamento definitivo da espera local e a verificação pública completa ficaram pendentes quando o usuário precisou desligar o PC. O estado de retomada foi salvo localmente.

## Verificação nativa concluída

O executável otimizado local completou a compilação em 22m13s. A interface compilada exibiu a capa real de AccuracyCrosshair recuperada em miniatura 96×96; a primeira recuperação pelo backend levou 656ms. Os quatro mundos existentes foram exibidos com dados reais de Vanilla, Fabulously Optimized, Pixelmon e Vanilla Perfected. O contexto de conta e amigos desse teste de interface foi simulado; a listagem de mundos e a recuperação da imagem vieram do executável e dos arquivos reais.

A checagem de abertura real usou a conta selecionada, sem abrir mundos. O critério foi a inicialização de áudio/atlas registrada no log novo e mais oito segundos de processo ativo; isso não comprova todos os menus. Todos os processos criados pelo teste foram encerrados ao final.

| Instância | Retorno de abertura | Recursos inicializados |
| --- | ---: | ---: |
| Vanilla 26.3 | 17.3s | 19.4s |
| Fabulously Optimized Fabric | 27.6s | 30.6s |
| RLCraft Forge | 23.3s | 120.6s |
| ATM10 NeoForge | 122.7s | 344.3s |

O ATM10 continua levando minutos para carregar seu próprio conteúdo. Não há comparação controlada com CurseForge nem demonstração de ganho universal de tempo. As oito instâncias e os 182 arquivos dos mundos foram comparados com a cópia anterior; os hashes SHA-256 dos mundos permaneceram iguais. As mensagens FATAL encontradas no RLCraft eram integrações opcionais de B.A.S.E/ContentTweaker com mods ausentes; não houve encerramento de processo nem relatório novo de crash durante a checagem.
