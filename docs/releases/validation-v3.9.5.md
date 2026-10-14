# Verificação da base Luxmc 3.9.5

Verificação local de preparação da release, em 14/10/2026:

- Metadados de versão e identidade do produto consistentes.
- `pnpm check`: zero erros e avisos.
- Frontend: 205 testes aprovados e build de produção concluído.
- Site: 90 testes aprovados, TypeScript verificado e artefatos gerados.
- Manifestos e assinaturas de atualização: 15 testes aprovados.
- Rust: `cargo check --locked` concluído; 252 testes de biblioteca aprovados, sete integrações explícitas ignoradas.
- Todos os 35 casos da suíte de navegador concluídos, incluindo os casos retomados após atualizar três fixtures antigas. A prévia principal continua animada durante o jogo com trabalho limitado; a fixture de GIF informa Plus ativo; o teste do Shield usa o rótulo traduzido atual.

As verificações de navegador exercitaram abertura persistente ao navegar, diagnósticos, atualizador, histórico completo de versões, recuperação de capas, importação/aplicação de skins, animações 3D, acesso exclusivo do dono, idiomas, wallpaper, fila persistente, salas, perfis e aparência.

No cenário de cem amigos, dez perfis foram consultados inicialmente; rolar até o último carregou seu perfil. O máximo foi de três consultas simultâneas. O canvas de partículas respeitou os limites de FPS e deixou de desenhar durante abertura/jogo.

O pacote 3.9.2 da mesma base de execução foi também testado no WebView2 nativo, com recuperação da identidade e GIF salvos, editor com prévias isoladas e inspeção dos três modpacks instalados. Vanilla 26.3 e Zombie Invade abriram em testes reais e ficaram estáveis por mais sessenta segundos. Esses testes não representam uma garantia de todos os modpacks, nem comprovam grande aceleração de início ou redução percentual da memória total.

A publicação 3.9.5 usa o workflow Release do repositório, com validação da fonte, builds por plataforma, teste do instalador Windows e conferência/assinatura dos manifestos antes de tornar a release pública. A assinatura de manifestos não equivale a Authenticode do editor Windows.
