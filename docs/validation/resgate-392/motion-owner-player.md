# Personalizador, controles do dono e player

Fonte do candidato: `282596fd38b5280bb027cb7655880456eaaf5064`, Windows x64, 13/10/2026.

O personalizador principal continua animando quando o modo de desempenho ou a pausa de previews durante o jogo estão ativos. Previews secundários continuam pausando; o principal usa até 30 FPS nesses cenários e interrompe o trabalho fora de tela ou sem foco. A barra de abertura usa a mesma rotação contínua dos indicadores de operação, independentemente das animações decorativas.

A central do dono exige simultaneamente assinatura consultada para a conta atual, identidade social correspondente e papel `owner` retornado pelo servidor. Um plano Pro e informação antiga de proprietário não bastam. A autorização das rotas privadas continua no servidor.

O middleware do site sobrescrevia a política do player público com `X-Frame-Options: DENY` e `frame-ancestors 'none'`. Somente a resposta HTML bem-sucedida de GET `/api/player` preserva agora sua política de incorporação restrita. Outros endpoints e vídeos inválidos continuam bloqueando frames. Publicado em `https://1acf51d7.luxmc-r92.pages.dev`, com confirmação também no domínio de produção.

Validações:

- `pnpm check`: zero erros e avisos; `pnpm build`, build do site, `cargo check --locked` e instalador NSIS concluídos.
- Cinco testes da assinatura e dois testes do endpoint/middleware passaram. O conjunto com comércio teve 19 testes aprovados.
- Navegador: poses reais de andar, correr e voar avançam no modo de desempenho e durante o jogo; Pro comum não vê a central, proprietário correspondente vê; indicadores de abertura mantêm animação; MP4 avança e YouTube é solicitado através do player HTTPS publicado com Referer correto.
- Executável extraído do instalador: diferenças entre frames confirmadas nas três animações do personalizador, sem encerrar o launcher.
- Zombie Invade 100 Days/Forge 1.20.1: janela em 70,27 s, recursos em 233,80 s, processo estável por mais 60 s. Vanilla 26.3: janela em 19,92 s, recursos em 22,19 s, estável por mais 60 s. Instâncias originais preservadas.
- Instalação local concluída, executável igual ao extraído do pacote, backup consistente e hashes da base/configurações preservados.

A preparação de aparência e a inspeção de segurança executam em paralelo e ambas terminam antes de iniciar Java. A inspeção completa não foi removida. Não se confirmou redução no tempo total do Zombie Invade: a medição anterior era janela em 57,16 s e recursos em 200,53 s; o tempo de conferência de arquivos permanece abaixo de 0,1 s. Esses resultados não comprovam abertura rápida de todos os modpacks, funcionalidade de todos os mods, LAN ou skins entre dois computadores. Vídeos ainda dependem das permissões do fornecedor. O instalador não tem certificado Authenticode confiável.

SHA256 do programa instalado: `edce71a9f3420a17a8f5711ba188d003adc63f8b194c5480f32f85a6e1a62318`.
SHA256 do instalador: `1b19e5eee77c36f838c9252110b300e605f73de00861101e3f8d66ea285115bc`.
