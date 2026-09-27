# Fluidez e uso de CPU — 26/09/2026

## Escopo desta revisão

Redução do trabalho contínuo de renderização do launcher, preservando o wallpaper animado, sua imagem durante pausas e o mecanismo de recuperação. Nenhuma alteração em arquivos de instâncias, contas ou modpacks nesta revisão.

## Alterações

- `src/lib/components/visuals/VideoWallpaper.svelte`: desenho do vídeo diretamente na superfície persistente, eliminando a cópia intermediária em resolução completa. A leitura de pixels para rejeitar quadros pretos fica restrita à primeira imagem e às extremidades do loop.
- O agendamento usa `requestVideoFrameCallback` quando disponível, com limite de aproximadamente 30 FPS para o wallpaper. Há alternativa com `requestAnimationFrame` para outros motores. A interface continua livre para atualizar na frequência da tela.
- Os callbacks são cancelados ao pausar o vídeo e retomados pelo evento de reprodução. Foram mantidos o loop nativo, a pausa durante jogos, a prévia estática em cache e o watchdog de recuperação.
- `src/app.css`: os controles sobre o wallpaper deixam de acumular desfoques individuais. Os painéis e as barras laterais mantêm seus efeitos de vidro.
- `tests/webkit-audit.py`: medição opcional da árvore de processos, intervalos de apresentação, contagem de quadros efetivamente desenhados e verificação de pausa/retomada durante jogos. Permite forçar o caminho alternativo de agendamento.

## Comparação controlada no WebKitGTK

Mesmo vídeo H.264 de 1920×1080 a 60 FPS, tela de skins, área de janela de 926×1001 pixels e mesmas opções de renderização. Janela de medição de 50 segundos depois do aquecimento. A conta e as chamadas nativas foram simuladas; o motor gráfico, o vídeo e o componente são reais.

| Indicador | Antes | Depois |
| --- | ---: | ---: |
| CPU da árvore de processos, 100% = um núcleo | 105,38% | 80,74% |
| Intervalo mediano dos callbacks de apresentação | 17 ms | 17 ms |
| Percentil 95 dos intervalos | 32 ms | 18 ms |
| Intervalos acima de 50 ms | 10 | 10 |

Redução de CPU de aproximadamente 23,4% nessa medição. O P95 melhorou, mas os eventos mais longos não foram eliminados. Isso não representa uma garantia universal de FPS ou consumo; vídeo, resolução, drivers e outras aplicações influenciam o resultado. Os intervalos são medidos por `requestAnimationFrame`, não por rastreamento físico do monitor.

## Evidências

Logs e capturas desta revisão: `/home/pedro/.cache/luxmc-validation/2026-09-26-performance/`.

- `before.log` e `after.log`: comparação controlada e loops do vídeo.
- `check.log`: Svelte/TypeScript, zero erros e zero avisos.
- `unit.log`: 52 testes aprovados em sete arquivos.
- `build.log`: compilação do executável de entrega.
- `endurance.log`: reprodução prolongada e pausa/retomada.
- `native-before.json` e `native-after.json`: comparação do executável real na página inicial com as preferências locais.

A análise de CPU soma o processo principal e seus descendentes. O RSS somado pode contar páginas compartilhadas mais de uma vez e não equivale a memória exclusiva.

## Executável real, primeira etapa

Na página inicial com as preferências do usuário, a soma de CPU do launcher e seus dois processos auxiliares caiu de 94,09% para 78,96% de um núcleo em amostras de 30 segundos: redução de aproximadamente 16,1%. O RSS somado variou de 628 para 714 MiB; essa medição não demonstra redução de memória. A comparação pertence à etapa de otimização do wallpaper anterior à revisão ampliada do personalizador e das listas.

A reprodução prolongada dessa etapa passou em 180 segundos, com 34 amostras, 31 imagens distintas e média de 27,4 desenhos por segundo, incluindo pausas provocadas. A retomada após pausa inesperada e após jogo foi confirmada.
