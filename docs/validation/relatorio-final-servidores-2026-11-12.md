# LUXMC — Relatório Final da Reconstrução do Motor e da Gestão de Servidores

> **Data:** 12/11/2026 · **Plataforma de verificação:** Windows 11, build de desenvolvimento (`tauri dev`, revisão local) · **Idioma:** pt-BR
>
> Este relatório é a Fase 15 da missão *LUXMC — Ultimate Server Engineering & Management Reconstruction*.
> Todo status de diagnóstico usa apenas: **Aprovado / Atenção / Falhou / Não verificado**.
> Nenhuma afirmação abaixo é baseada em simulação: tudo o que marca **Verificado** foi executado
> nesta máquina, com processo Java real, em12/11/2026.

---

## 1. Resumo executivo

| Área | Resultado |
| --- | --- |
| Aplicativo real em execução | **Verificado** — app Tauri + painel de servidor abertos, sem simulação |
| Início do servidor pela interface | **Verificado** — botão `INICIAR SERVIDOR`, fases `preparing → downloading → starting → running` |
| Server List Ping real (externo ao processo) | **Verificado** — resposta legítima do servidor vanilla em duas instâncias |
| Memória Java medida de verdade | **Verificado** — `jcmd` no PID real, heap em uso vs. commitado vs. máximo |
| Isolamento de duas instâncias simultâneas | **Verificado** — dois servidores rodando ao mesmo tempo, pastas e portas separadas |
| Sugestão de porta em conflito | **Verificado** — falha com mensagem PT-BR indicando porta livre verificada |
| Janelas de CMD/terminal piscando | **Corrigido e verificado** — zero janelas em amostragem de400 ms por40 s |
| Gamerules na versão correta (26.x) | **Corrigido e verificado** — zero `Incorrect argument` no console real |
| Honestidade de status e métricas | **Verificado** — sem status falso; offline mostra ausência de métricas |
| Redesign (Servidores + console) | **Aplicado e verificado** por screenshot |
| Bateria de testes automatizados | **Verificado** — Rust373/0, vitest243/243, svelte-check0/0, browser test PASS |
| Conectividade externa (IP público/encaminhamento) | **Não verificado** — nenhum teste externo foi executado |

---

## 2. Requisitos de integridade da missão e como foram cumpridos

###2.1 Nada de status falso
- `local_server_telemetry` retorna apenas medições reais: PID do processo Java, memória do processo,
  heap via `jcmd`, CPU real do sistema. Com o servidor parado, a resposta é literalmente
  `Servidor offline — sem métricas de processo.` com todos os campos nulos.
- `tps` e `mspt` só aparecem quando há fonte real; hoje retornam `null` (exibidos como “Não disponível”).
- RAM é distinguível em três camadas: **configurada** (`configuredRamMb` / `heapMaxMb`),
  **utilizada do heap** (`heapUsedMb` / `heapCommittedMb`) e **processo Java** (`processMemoryMb`).
- Verificado ao vivo: PID22768, processo≈597 MB, heap229 MB em uso de366 MB commitados, máximo2048 MB
  (= RAM configurada), sistema18,9/32,6 GiB — todos os valores vindos do sistema, não de estimativas.

###2.2 Sem início automático após a instalação
- A instalação não sobe nenhum servidor; o servidor só inicia quando o usuário clica em
  `INICIAR SERVIDOR`. Verificado nesta sessão: após reinstalar o app e abri-lo, o estado permanecia
  `stopped` até o clique explícito.

###2.3 Sem pastas compartilhadas entre instâncias
- Cada servidor vive em `%APPDATA%\github\Luxmc\data\local-servers\<uuid>\` com `server.properties`,
  `world/`, `luxmc-settings.json` e `logs/` próprios. Verificado com dois servidores simultâneos:
  os dois diretórios existiam e continham arquivos independentes, sem qualquer link/symlink.

###2.4 Sem lançamentos duplicados
- O `local_server_start` usa trava de lançamento (`launch_lock`) e o worker é singleton com
  `worker.lock` + `endpoint.json`; requisições concorrentes recebem erro PT-BR em vez de fila infinita.
- Verificado: clicar/start em servidor já ativo não gerou segundo processo Java (um único PID por servidor).

###2.5 Sem endereços ou portas inventados
- Toda porta exibida vem do `TcpListener` real do processo (`local_addr()`), não de configuração presumida.
- O Server List Ping foi feito **de fora do processo** com script próprio (TCP + handshake varint) e
  recebeu a resposta real do servidor.

###2.6 Sem mudanças inseguras de firewall
- O diagnóstico **informa** o estado do firewall (`Firewall do Windows` → status `Atenção` quando não há
  regra) e declara que o Luxmc não altera o firewall automaticamente. Nenhuma regra foi criada ou
  modificada por este trabalho.

###2.7 Sem RCON público
- O RCON é usado apenas em `127.0.0.1` para parada graciosa durante os testes (grava o mundo com
  segurança). `enable-rcon=true` fica restrito ao localhost conforme o `server.properties` gerido;
  nenhuma porta RCON foi publicada ou testada de fora.

###2.8 Conectividade externa
- **Não verificado.** Nenhum teste de IP público, NAT, UPnP ou encaminhamento de portas foi executado.
  Nenhuma afirmação de acesso externo é feita neste relatório.

###2.9 Diagnósticos
- Os status exibidos pelo painel de diagnóstico são apenas **Aprovado / Atenção / Falhou / Não verificado**
  (codificados como `approved / attention / failed / unverified`), incluindo o teste
  `Conexão local (127.0.0.1) e Server List Ping`.

###2.10 Dashboard e instalador
- Textos do dashboard preservados sem alteração (`LUXMC | SERVIDOR DEDICATO - GERENCIADOR DE SERVIDOR`,
  `LUXMC CORE`,5 cabos, string da EULA). Instalador sem arte3D (nenhum elemento3D adicionado).

---

##3. Correções de engenharia aplicadas nesta reconstrução

###3.1 Janelas de CMD piscando na tela do usuário — **Causa raiz corrigida**
- **Causa:** o processo `luxmc-server.exe` (worker) é uma **cópia** de `luxmc.exe` guardada em
  `data\server-worker\<revisao>\`, copiada uma única vez (`if !executable.exists()`). A cópia era de
  um build anterior à correção `CREATE_NO_WINDOW`, então a telemetria rodava no binário antigo e o
  `jcmd` abria um terminal visível a cada1–2 s.
- **Correções em `server_worker.rs`:** atualização do exe quando a fonte é mais nova (staging `.new` →
  rename → fallback), guarda de lock em `connect()` com auto-reparo de15 s e erro PT-BR claro,
  reescrita do `endpoint.json` ausente no loop do worker, e resposta quando o semáforo de16 requisições
  está esgotado (antes: silêncio eterno).
- **Todos os spawns do código** usam `CREATE_NO_WINDOW (0x08000000)`.
- **Verificação:** amostragem de janelas (`WindowsTerminal/cmd/conhost/powershell/jcmd`) a cada400 ms
  por40 s com servidor online e telemetria ativa, em três execuções independentes → **NENHUMA janela**.

###3.2 Gamerules erradas na versão26.x — **Causa raiz corrigida**
- **Causa:** a versão1.21.11/26.x renomeou regras para snake_case; o código enviava nomes antigos e o
  servidor respondia `Incorrect argument`.
- **Correção:** backend `settings.rs::startup_commands()` com faixas por versão (snake_case a partir de
  1.21.11/26.x/25w44a; camelCase em1.21.9–1.21.10; básicos desde1.6.1), sem regras inexistentes
  (`doAnimalSpawning` nunca existiu no Java; `functionCommandLimit` é Bedrock). Frontend
  `gamerules.ts` com mapa `SNAKE_CASE_RENAMES` + `mapValue`.
- **Verificado ao vivo:** console do servidor26.3 mostra `gamerule spawn_monsters true` etc.
  respondidos pelo RCON — **0** `Incorrect argument`, **0** linhas `[ERROR]`, `Done (0.243s)`.

###3.3 Telemetria sem heap e com memória do sistema zerada — **Corrigido**
- Parser do `jcmd` agora aceita o formato do Java25 (`total reserved N K, committed N K, used N K`)
  além do legado; teste unitário dedicado incluído.
- `system_resource_sample` passou a chamar `refresh_memory()` (antes retornava0).

###3.4 Congelamento da IPC do worker — **Corrigido**
- Sintoma: toda invocação travava≈14 s. Causa: `endpoint.json` removido enquanto o worker vivia, e cada
  chamada tentava recopiar um exe de≈100 MB. Corrigido com guarda de lock + auto-reparo + resposta de
  “serviço ocupado”. Hoje: `local_server_list`≈6 ms, `local_server_telemetry`≈300–520 ms.

---

##4. Evidências dos testes de engenharia (executados em12/11/2026)

###4.1 Server List Ping real (script externo, TCP puro)
```
$ node ping-minecraft.cjs 127.0.0.1 33047   # servidor "oi"
{ ok: true, latencyMs: 3, version: "26.3", protocol: 777,
  playersOnline: 0, playersMax: 10, motd: "oi" }

$ node ping-minecraft.cjs 127.0.0.1 25565   # servidor "isola-b"
{ ok: true, latencyMs: 3, version: "26.3", protocol: 777,
  playersOnline: 0, playersMax: 5, motd: "isola-b" }
```

###4.2 Isolamento de duas instâncias simultâneas
| | Servidor A (“oi”) | Servidor B (“isola-b”) |
| --- | --- | --- |
| Estado | `running` | `running` |
| Porta real (listener) | 33047 | 25565 |
| PID Java | 22768 | 17956 |
| RAM configurada / heap máximo | 2048 MB | 1024 MB |
| Diretório | `local-servers\b855869b-…` | `local-servers\d722ff20-…` |
| Arquivos próprios | `world/`, `server.properties`, `luxmc-settings.json` | idem, independentes |

- Servidor B criado pela API real `local_server_create` (reaproveitou o artefato oficial com verificação
  SHA-1, sem download duplicado) e iniciado enquanto A permanecia online.
- Após o teste, ambos foram **parados com segurança** (mundo gravado) e B foi excluída.

###4.3 Conflito de porta com sugestão de porta livre
- Com A em33047, B foi configurada para33047 e iniciada. Resultado (estado `failed`):

> `A porta 33047 já está em uso por outro processo (…). A porta25600 foi verificada e está livre;
> selecione-a em Configurações do servidor.`

- A sugestão vem de `free_port_suggestion()` (faixa25565–25600), que **testa** a porta antes de sugerir.

###4.4 Telemetria real (servidor online)
```
pid:22768, processMemoryMb≈597, heapUsedMb229, heapCommittedMb366,
heapMaxMb2048 (= RAM configurada), heapSource: "jcmd",
systemUsedMemoryMb18946/32639, tps/mspt: null,
detail: "Medições reais do processo Java (PID22768) via jcmd local; heap máximo = RAM configurada."
```

###4.5 Bateria automatizada
| Suíte | Resultado |
| --- | --- |
| `cargo test --lib` | **373 passed,0 failed**,19 ignored |
| `vitest run` | **243/243 passed** (41 arquivos) |
| `svelte-check` | **0 erros,0 warnings** |
| `tests/local-server.browser.cjs` | **PASS** (wizard de5 passos, EULA, timeline de instalação, round-trip de console, gamerules por versão, backups, persistência de configurações, exclusão confirmada) |

---

##5. Redesign aplicado (pedido do usuário)

- **Página Servidores** (`ServersPanel.svelte`): blocos arrastáveis (drag HTML5 com indicador
  antes/depois), paleta de8 cores por bloco, ordem e cores persistidas em `localStorage`
  (`luxmc.localServerOrder` / `luxmc.localServerColors`), faixa de acento colorida, ações de
  iniciar/paleta/excluir.
- **Console** (`ConsolePanel.svelte`): chips pílula, terminal em20 px, linhas em bolhas arredondadas
  com barra warn/error, timestamps em pílula, input redondo com badge `>`.
- **Tokens de raio** (`src/app.css`): `--radius-*` arredondados nos dois temas;52 `border-radius`
  atualizados em todos os painéis do servidor.
- Conferido por screenshot nas4 seções (`inicio`, `servidores`, `console`, `desempenho`).

---

##6. Lista de verificação final da missão

| # | Requisito | Status |
| --- | --- | --- |
| 1 | App real rodando e testado de ponta a ponta | **Verificado** |
| 2 | Servidor real iniciado pela UI e respondendo | **Verificado** |
| 3 | Conectividade local provada por Server List Ping externo | **Verificado** |
| 4 | RAM: configurada vs. heap vs. processo Java distinguíveis e reais | **Verificado** |
| 5 | Isolamento de2 instâncias (pastas, portas, processos) | **Verificado** |
| 6 | Sugestão de porta em conflito (testada, não presumida) | **Verificado** |
| 7 | Zero janelas de CMD em operação normal | **Verificado** |
| 8 | Gamerules corretas por versão, zero erro no console | **Verificado** |
| 9 | Sem status falso, sem auto-start, sem RCON público, sem firewall alterado | **Verificado** |
| 10 | Diagnósticos apenas com os4 status permitidos | **Verificado** |
| 11 | Dashboard e instalador preservados | **Verificado** |
| 12 | Testes automatizados completos | **Verificado** (373 +243 +0/0 + browser PASS) |
| 13 | Conectividade externa (IP público/NAT) | **Não verificado** |

---

##7. Como reproduzir as verificações

1. `pnpm tauri dev` (com `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` para CDP).
2. Criar/iniciar um servidor pela página `hosting/server` → botão `INICIAR SERVIDOR`.
3. Ping externo: `node ping-minecraft.cjs 127.0.0.1 <porta>` (TCP + handshake varint, sem uso do processo).
4. Telemetria: `local_server_telemetry` pelo painel **Desempenho** (valores do `jcmd` no PID real).
5. Isolamento: criar um2º servidor e iniciá-lo com o1º online; comparar portas, PIDs e diretórios.
6. Conflito: apontar a porta do2º servidor para a do1º e iniciar — a falha sugere porta livre verificada.
7. Suítes: `cargo test --lib` (em `src-tauri/`), `node node_modules/vitest/vitest.mjs run`,
   `node node_modules/svelte-check/bin/svelte-check --tsconfig ./tsconfig.json`,
   `LUXMC_CHROMIUM_EXECUTABLE=<Edge> node tests/local-server.browser.cjs`.
