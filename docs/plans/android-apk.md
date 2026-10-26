# Luxmc para Android

Status: proposta técnica; nenhum APK implementado ou publicado nesta revisão.

## Primeira entrega

Um aplicativo para entrar com Microsoft, sincronizar o perfil, consultar planos, conversar com amigos e explorar o catálogo. A interface Svelte pode ser adaptada para toque com navegação inferior, áreas de toque maiores e tratamento do teclado. Os serviços de identidade, catálogo e perfis podem ser compartilhados com o desktop.

Tauri 2 fornece suporte Android e geração de APK. Será necessário preparar Android Studio, JDK, SDK, NDK e o alvo Rust ARM64; separar os módulos de desktop por plataforma; substituir bandeja, processos, atalhos e janelas por integrações Android. O redirecionamento OAuth deve ser registrado para o aplicativo, com PKCE e armazenamento seguro da sessão.

Após essa adaptação, a compilação de distribuição usa `pnpm tauri android build --apk`. O APK precisa de uma chave de assinatura permanente, mantida fora do repositório e dos logs. Atualizações devem usar a mesma chave e um código de versão crescente. O Google Play usa o fluxo de AAB descrito pela documentação do Tauri.

## Executar Minecraft Java

O APK da interface não executa automaticamente a edição Java. Essa entrega exige uma camada Android própria para JVM, bibliotecas nativas ARM64, integração gráfica de LWJGL/SDL, renderização compatível com o driver do aparelho, áudio, controles de toque e ciclo de vida do Android.

ZalithLauncher 2 é uma referência pública de implementação Android com base no núcleo Pojav. Antes de incorporar componentes, será necessário verificar as licenças de cada componente e os requisitos de distribuição; esta proposta não incorpora código desses projetos.

A sequência recomendada é validar Vanilla em ARM64, acrescentar Fabric e então avaliar Forge, NeoForge e outros loaders por versão. Cada combinação precisa registrar aparelho, GPU, Android, Java, loader e resultado. Shaders, mods com bibliotecas nativas e modpacks grandes exigem verificação específica e não devem receber compatibilidade presumida.

## Critérios de entrega

- Login oficial pelo navegador, sem recolher a senha Microsoft no aplicativo.
- Arquivos restritos ao armazenamento do aplicativo, com importação e exportação pelo seletor do Android.
- Downloads retomáveis, conferência de integridade e tratamento de interrupção pelo sistema.
- Recuperação de perfil e mundos após atualização, troca de orientação e suspensão.
- APK assinado, checksum e fonte correspondente publicados no GitHub; AAB e requisitos da loja avaliados separadamente.
- Testes em celulares reais antes de oferecer execução do jogo como recurso disponível.

## Referências

- [Tauri: requisitos Android](https://v2.tauri.app/start/prerequisites/#android)
- [Tauri: distribuição APK e Google Play](https://v2.tauri.app/distribute/google-play/)
- [ZalithLauncher 2: implementação Android](https://github.com/ZalithLauncher/ZalithLauncher2)
