# Validação de desempenho da linha 4.0.1

As alterações preservam as funções e a frequência configurada das animações. O objetivo é eliminar trabalho repetido da interface e do cache, não prometer um ganho de FPS do Minecraft.

## Agendamento de animações

Reprodução: `node scripts/benchmark-frame-loop.mjs`.

O teste determinístico simula dois segundos em uma tela de 144 Hz, alternando prévias de 60 e 30 FPS. Cada prévia continua renderizando respectivamente 120 ou 60 quadros, antes e depois.

| Prévias | Callbacks RAF anteriores | Callbacks RAF compartilhados | Timers anteriores | Timers compartilhados |
| --- | --- | --- | --- | --- |
| 1 | 168 | 168 | 119 | 119 |
| 8 | 1.008 | 168 | 712 | 119 |
| 24 | 3.024 | 168 | 2.136 | 119 |

Isso mede trabalho de agendamento. O custo de renderizar cada cena permanece e o resultado não é uma medição de FPS em hardware real.

## Cache de ícones

Reprodução: `cargo test --manifest-path src-tauri/Cargo.toml --lib commands::content_icons::tests::benchmark_batched_icon_cache -- --ignored --nocapture`.

Em uma tabela SQLite temporária em arquivo, com journal DELETE, sincronização FULL e 32 registros por grupo, cinco amostras compararam gravações individuais e uma transação por grupo. A mediana nesta máquina foi de **1.103,96 ms** para gravações individuais e **23,48 ms** em lote. A primeira amostra individual, com cache frio, demorou 12.272,49 ms; todas as amostras foram mantidas no relatório privado. O desempenho de outros discos pode variar.

A aplicação grava o cache em transações curtas antes ou depois das consultas de rede; não mantém a conexão SQLite ocupada enquanto espera os provedores. Um teste confirma que copiar um cache antigo não substitui uma imagem mais nova já persistida e que as atualizações resolvidas são gravadas corretamente.

## Carregamento da interface

As prévias 3D passam a criar o visualizador quando estão visíveis e ativas. Os módulos dos editores de perfil são carregados ao abrir; o perfil público permanece montado depois da primeira abertura para preservar suas transições. A preparação local de ícones de outra instância avança mesmo quando um provedor da primeira está lento, mantendo duas filas de consulta remota para limitar a concorrência.

Um ensaio de navegador com dados fictícios abriu início, skins, amigos e detalhes de instância sem erros. Tempos de abertura e uso de CPU desse ensaio variaram com a compilação e outros processos da máquina, portanto não são usados como promessa de aceleração global. A navegação nas 15 telas da suíte de desempenho passou em claro e escuro, com cancelamento de instalação e movimento reduzido preservados.

Os testes não substituem uma partida de modpack em todos os computadores. Downloads, autenticação externa, primeiro carregamento do mundo e inicialização dos mods continuam sujeitos aos provedores, ao hardware e ao conteúdo instalado.
