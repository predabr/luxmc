# Aparência, fluidez, tradução e Shield — 14/10/2026

Trabalho retomado após a pausa solicitada pelo usuário. A publicação no GitHub permanece cancelada; não foi criada release, enviado commit ou reaberto o PR 6.

## Alterações

O editor de aparência recebeu uma prévia com o avatar e a capa salvos, amostras de cores distintas e opções separadas de moldura, efeito e composição. Cada assinatura visual agora reinicia suas variáveis de estilo: a moldura selecionada no perfil externo não contamina os exemplos internos. As miniaturas são estáticas; a prévia mantém os efeitos. O nome personalizado e o GIF existentes são preservados. Novos GIFs exigem Plus ou Pro, com verificação no servidor.

As partículas deixam de desenhar durante a abertura do jogo e retomam depois. Os efeitos decorativos da assinatura pausam durante abertura/jogo, preservando os indicadores operacionais. A lista de amigos consulta imagens de perfis próximos da área visível, usa cache e limita a três consultas simultâneas. No cenário de 100 amigos, a posição inicial fez dez consultas; rolar até o último amigo carregou seu perfil. Isso comprova redução de trabalho, sem constituir medição da memória total do launcher.

O Shield inspeciona também bibliotecas JAR embutidas, com limites de profundidade, bytes e entradas. Arquivos incompletos ou que ultrapassam o orçamento não são tratados como limpos. O cache de conteúdo continua condicionado ao hash completo do arquivo. Dez testes do scanner passaram, incluindo arquivos embutidos e limites. Nenhuma configuração do antivírus do Windows foi alterada.

O launcher recebeu textos de aparência e segurança em português, inglês e espanhol. As três traduções têm as mesmas chaves e parâmetros. A página de planos recebeu seletor de idioma e tradução dos textos e estados principais, preservando a conta, a seleção e o rascunho do pagamento. Os efeitos de clique são finitos e respeitam movimento reduzido.

## Verificação concluída antes da pausa

- `pnpm check`: zero erros e avisos; frontend de produção compilado.
- `cargo check --locked`: concluído, com cinco avisos existentes de código não utilizado.
- Testes de servidor e UI do site, traduções, perfil/GIF, benefícios dos planos e efeitos aprovados.
- Navegador: aparência nos três idiomas, temas claro/escuro e tamanhos compacto/amplo; molduras isoladas e sem overflow. Andar, Correr e Voar mantiveram movimento no canvas do personalizador.
- Navegador: partículas pausam durante abertura/jogo e retomam; consultas de perfis limitadas e carregadas por visibilidade.
- Site em tela de 430 pixels: idiomas, identidade e plano ativo, sem overflow horizontal ou erros registrados.

## Pacote e teste nativo

O instalador final foi extraído e seu executável comparado byte a byte ao binário de entrada. A única diferença foi o marcador do tipo de bundle. Nenhum executável de reparo foi incluído.

Pasta privada: `C:/Users/preda/AppData/Local/Luxmc-build/candidates/studio-shield-20261014`.

SHA256 do programa: `d4efdd78d217ad541f40407b0cf96009e47c115cd53be15ae74de1bd0fa79b30`.

SHA256 do instalador: `3f84a3397d744a6e7fd3cf13257e4cc73dcf3ac583fa91843b904af7f528d297`.

O WebView2 nativo do pacote final consultou a produção, exibiu o nome e o GIF salvos e manteve ambos no editor. A moldura original apresentou zero pixels de contorno; a dupla, quatro pixels, sem alterar a forma arredondada. O launcher permaneceu aberto.

Inspeção real de segurança no executável release, sem modificar os mods:

| Instância | JARs | Primeira inspeção | Repetição com cache | Resultado |
| --- | ---: | ---: | ---: | --- |
| All the Mods 10 | 490 | 2,294 s | 0,581 s | Sem achados |
| RLCraft | 178 | 0,997 s | 0,171 s | Sem achados |
| Zombie Invade | 143 | 0,543 s | 0,223 s | Sem achados |

Esses tempos são da inspeção, não da inicialização dos modpacks. Sem achados não equivale a certificação de ausência de malware.

## Publicação e continuação

O site com o novo editor de estilos compartilhados e idiomas dos planos já estava publicado em `https://f6b719ec.luxmc-r92.pages.dev`. O instalador público, antes da retomada, ainda era o pacote de recuperação de perfil: SHA256 `2ec6316d5851045964da78a6158b8f8c7742a4a2becaaf0d3652ca34ec9238a6`.

A retomada concluiu a abertura real do Vanilla 26.3 e Zombie Invade com o candidato final:

| Instância | Janela | Recursos | Estabilidade adicional |
| --- | ---: | ---: | ---: |
| Vanilla 26.3 | 21,17 s | 23,20 s | 60 s |
| Zombie Invade / Forge 1.20.1 | 50,40 s | 169,59 s | 60 s |

O log do Zombie Invade confirmou a inicialização dos mods e informou 172,512 segundos de início pelo loader. Não foram registrados erros fatais nos critérios do teste, os quatro perfis originais foram preservados e somente os jogos iniciados pela validação foram encerrados. Uma única execução não comprova ganho causal de desempenho, nem cobre todas as combinações de modpacks/loaders.

A instalação local terminou com código zero, preservou os hashes da base e dos arquivos de configuração e confirmou que o programa instalado corresponde ao extraído do pacote. Backup consistente: `C:/Users/preda/AppData/Local/Luxmc-build/backups/studio-shield-20261014-install`. Instalador para teste: `C:/Users/preda/Downloads/Lux MC Launcher - Aparencia e Shield 3.9.2.exe`.

A publicação do instalador no site usou `scripts/deploy-website.mjs`, com diretório `website/`, e terminou em `https://f8e2c60a.luxmc-r92.pages.dev`. O GET completo de `https://luxmc-r92.pages.dev/download/windows` retornou HTTP 200; seu SHA256 corresponde ao candidato `3f84a3397d744a6e7fd3cf13257e4cc73dcf3ac583fa91843b904af7f528d297`. A página de planos pública manteve o seletor de idioma. O endpoint social sem sessão retornou HTTP 401.

A versão permanece 3.9.2; não há nova release nem atualização automática publicada no GitHub. O computador do usuário já recebeu o pacote novo por instalação local.

A revisão automática de aprovação rejeitou a ação que combinava verificação de download e abertura do aplicativo, com a resposta “blocked by policy”, sem motivo específico. A verificação foi concluída separadamente, sem executar arquivos. O programa instalado não foi aberto após a instalação; essa abertura permanece para o usuário. Nenhum processo do usuário foi encerrado.

O experimento de índice ZIP continua desativado por padrão: os testes anteriores não demonstraram aceleração relevante da abertura real. Não há comprovação de inicialização rápida de todos os modpacks, comparação com CurseForge ou redução percentual da memória total.
