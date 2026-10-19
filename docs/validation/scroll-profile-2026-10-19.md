# Fluidez, miniaturas e perfil — 19/10/2026

## Problemas reproduzidos

- O vídeo de fundo recebia `pause()` sempre que `appState.isScrolling` ficava ativo. A rolagem permanece com eventos passivos e redução de sombras/blur; o vídeo continua tocando. Pausas por jogo aberto, janela oculta e preferência de foco foram preservadas.
- A interface restaurava o diagnóstico de `launch-latest.json` como uma falha nova. O registro observado era de 16/10, com 2084 segundos de sessão e saída -8; não era uma nova falha de inicialização.
- O monitor gravava a última linha do stderr antes de drenar os leitores e procurar a causa nos logs. Isso promovia um aviso de depreciação do Java a diagnóstico. Agora o registro e o evento usam a causa extraída depois da drenagem. A leitura do final dos logs é limitada a 256 KiB e ocorre fora do executor assíncrono. Os avisos permanecem nos logs.
- A antecipação de miniaturas só fazia extração local. Imagens remotas eram solicitadas ao abrir a página. A antecipação agora cobre imagens dos provedores, prioriza a extração local de todas as seções e limita downloads de imagens a quatro simultâneos, com fila e memória limitadas. Requisições idênticas em andamento são compartilhadas. Trabalho pendente aguarda quando o jogo inicia.
- Mods já registrados podem usar o projeto conhecido para obter uma miniatura, sem reler todo o JAR para calcular SHA-1. Isso é metadado visual e não altera verificações de integridade, downloads ou execução de mods. O fallback por hash continua disponível.
- Metadados Forge/NeoForge agora usam o parser TOML, incluindo comentários e propriedades do primeiro mod. O cache v3 reaproveita imagens positivas anteriores e permite nova extração para falhas antigas.

## Interface e planos

- Paletas, molduras, efeitos e composições usam identidade real nas amostras. Efeitos pequenos animam ao passar o mouse ou focar, evitando animar toda a grade continuamente. As preferências de movimento reduzido continuam respeitadas.
- Escolhas de aparência bloqueadas abrem informação de plano sem aplicar o estilo.
- Upload de GIF em perfil ou pasta no Free abre um diálogo de Plus/Pro com ação explícita para conhecer planos. A identificação usa o cabeçalho do arquivo, inclusive quando o GIF foi renomeado para PNG. Plus/Pro preservam o GIF após reinício. Wallpaper permanece gratuito.
- Feedback dos botões acontece antes do handler de clique, inclusive quando esse handler inicia o jogo. Os efeitos têm limpeza e ficam desativados no modo de desempenho e durante o jogo.
- Novos textos em português, inglês e espanhol.

## Validação

- `pnpm check`: zero erros e zero avisos.
- Vitest: 218 testes aprovados em 36 arquivos, incluindo antecipação, prioridade local, cache e limitação/suspensão de downloads.
- Rust: 268 testes aprovados; nove testes de integração que exigem condições externas foram ignorados pela configuração existente. `cargo check` aprovado, com avisos preexistentes de código não utilizado.
- Políticas de perfil, mídia e traduções do site: 11 testes aprovados. Regras de plano do servidor preservadas.
- Teste de vídeo real: 15 eventos de roda do mouse, nenhum evento de pausa e avanço do tempo de reprodução; pausa e retomada por jogo aberto continuam funcionando. Reinício com arquivo original removido e porta de mídia alterada também passou.
- Fluxo de sessão: erro histórico não reaparece; cancelamento e navegação preservam a sessão; falha atual aparece; evento de sessão antiga é ignorado.
- Perfil: recuperação de dados e uploads Free/Plus/Pro; aparência em português, inglês e espanhol, temas claro/escuro e tela compacta; diálogo de plano e destino do portal; GIF renomeado bloqueado no Free; pastas Plus/Pro preservadas após recarga.
- Lista com mil mods: virtualização, último item, filtro, seleção, congelamento, configuração e recuperação de erro de gravação passaram em 1440 e 960 px.
- Os 39 scripts da suíte de navegador passaram em lotes, incluindo a nova verificação de GIFs nas pastas. Dois checks de apresentação exigiram repetição durante a compilação concorrente: troca de imagem do catálogo e largura inicial do login. O teste de login agora aguarda fontes e layout antes de verificar a largura; as repetições passaram.

## Medição em dados locais

O executável anterior foi medido com os arquivos reais de Vanilla Perfected, ATM10, RLCraft e Fabulously Optimized. As primeiras 32 miniaturas levaram 57–151 ms no backend; o ícone do Luxmc Essentials foi encontrado em todas as quatro instâncias. Isso delimita o problema de antecipação/exibição; não demonstra redução do tempo até o menu do Minecraft.

Cópia privada do banco, autenticação, preferências e executável anterior: `C:/Users/preda/AppData/Local/Luxmc-build/backups/scroll-profile-20261019`. Resultados detalhados e logs locais: `C:/Users/preda/AppData/Local/Luxmc-build/requests/scroll-profile-20261019`.

## Distribuição e confiança

Estas correções são uma revisão local de 4.0.0. A publicação imutável `v4.0.0` não foi sobrescrita. Nenhuma proteção do Windows foi desativada. Assinatura do atualizador e assinatura Authenticode são mecanismos distintos; popularidade não garante aceitação pelo Windows. Uma assinatura Authenticode confiável e o processo de análise de falsos positivos da Microsoft continuam sendo os caminhos de distribuição apropriados.

Referências: [reputação SmartScreen](https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation), [envio para análise](https://www.microsoft.com/en-us/wdsi/filesubmission).
