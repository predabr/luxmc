# Verificação da versão 3.6

Resultados obtidos em 9 de outubro de 2026. Esta revisão descreve o que foi executado, sem estender um teste a todas as combinações de mods, versões ou redes.

| Área | Verificação | Resultado |
| --- | --- | --- |
| Compilação | CI em Windows e Linux; instaladores Windows, Linux e macOS | Todos os jobs concluídos com sucesso |
| Interface | 15 rotas em tema claro e escuro, incluindo redirecionamentos | 30 capturas, sem erros de página ou transbordamento da raiz em 1440 × 1000 |
| Movimento | Entradas em 13 rotas e aba Java dos ajustes | Transições executam e terminam; movimento reduzido suprime a entrada |
| Frontend | svelte-check e testes Vitest | Sem erros ou avisos; 145 testes passaram na revisão |
| Site | Conta, catálogo, downloads, mídia, segurança e interface | 47 testes passaram; visual conferido em 1440 e 390 pixels |
| Instalação Windows | Atualização local de 3.5 para 3.6 | Instalador encerrou com código zero; cinco arquivos de dados/configuração/mundos conferidos permaneceram iguais; verificador independente confirmou integridade |
| Modpack real | Fabulously Optimized 6.5.0, Minecraft 1.21.1 | Instalação em 24,4 s; 50 arquivos conferidos por hash; preparação em 7,9 s com caches existentes; renderizador abriu, permaneceu vivo por mais 90 s e encerrou normalmente |
| Bedrock real | Catálogo e instalação gerenciada pelo Luxmc | 229 entradas observadas; 1.20.81.01 UWP e 1.26.52.3 GDK instalados durante a revisão; 1.26.52.3 abriu novamente pelo executável 3.6 instalado |
| Rede | Dois processos independentes do motor, relay e reinício de um participante | Conexão e reconexão passaram, sem perda nas 24 amostras; esse teste usou modo sem adaptador e não comprova uma partida entre PCs |
| Distribuição | Todos os instaladores; manifestos SHA256SUMS e latest.json | SHA-256 e assinaturas Ed25519 verificados com a chave oficial |

O teste de partida LAN entre dois computadores em redes diferentes continua pendente de retorno dos participantes. Não há resultado de suspensão do Windows, troca de roteador ou estabilidade prolongada em uma partida real. O launcher prepara um adaptador Wintun no fluxo de produção; o teste de relay sem adaptador não substitui a verificação desse fluxo.

Na observação de consumo da instalação local, foram coletadas 30 amostras dos sete processos do launcher e WebView, na tela inicial com wallpaper e GIFs personalizados e outros aplicativos/jogo ativos: 711,23 MiB de memória residente somada e CPU média de 4,40%, normalizada por 12 processadores lógicos. Páginas compartilhadas podem aparecer em mais de um working set. Esses valores não são garantia de consumo nem comparação controlada com versões anteriores. Os dados estão em [windows-3.6-idle.json](../../website/benchmarks/windows-3.6-idle.json).

O teste Bedrock final comprovou catálogo, instalação e abertura do processo respondendo. Não criou um mundo novo nem substituiu uma sessão Microsoft ou licença do Windows. Os dados Bedrock são gerenciados pela identidade do pacote Windows; não há isolamento de mundos Bedrock por cartão da biblioteca.

As referências de implementação foram consultadas nas fontes dos próprios projetos: [instalação de packs no Prism Launcher](https://prismlauncher.org/wiki/help-pages/modrinth-platform/), [metadados, dependências e hashes das versões Modrinth](https://docs.modrinth.com/api/operations/getversion/) e [rede e relay EasyTier](https://doc.easytier.cn/guide/network/fast-networking.html). Integrações preservam a edição e a versão do pack e conferem os arquivos fornecidos pelo autor.

O [GitHub documenta que repositórios públicos são acessíveis a qualquer pessoa](https://docs.github.com/en/repositories/creating-and-managing-repositories/about-repositories). A licença e a política de marca restringem o uso permitido, enquanto a assinatura permite identificar a distribuição oficial. Nenhuma dessas medidas impede tecnicamente copiar código público ou examinar um executável.
