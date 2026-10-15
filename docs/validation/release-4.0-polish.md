# Revisão da 4.0 — 15/10/2026

O usuário autorizou novamente a publicação da 4.0 no GitHub, após os ajustes desta revisão. Não substituir uma compilação pública sem mudar sua identidade de build.

## Correções

- Medição de armazenamento usa identidades de arquivos obtidas por diretório no Windows, com fallback para sistemas sem suporte. A leitura roda fora da interface; hard links são deduplicados e o relatório é reutilizado por até dois minutos.
- Instalação e reparo atualizam a quantidade real de JARs e invalidam o relatório. A interface conserva o relatório entre páginas; a limpeza não reinicia ao navegar.
- Modrinth ordena as versões por publicação. A verificação de atualização considera a edição de Minecraft, loader e versão instalada; não oferece um downgrade como atualização. A conclusão da instalação invalida respostas antigas em andamento.
- Ícones locais são antecipados em lotes, com duas tarefas, cache limitado a 6 MiB e pausa enquanto o jogo inicia ou está aberto. O primeiro resultado da lista já pode trazer até 1 MiB de miniaturas salvas.
- O seletor usa a camada de popover quando disponível, posição limitada à janela e portal dentro do diálogo quando necessário para preservar o foco.
- A seleção de skin abre confirmação central sem aplicar automaticamente. Continuar editando preserva a alteração pendente. O renderizador e as animações 3D existentes foram mantidos.
- Login, consulta de plano e desinstalador receberam correções de apresentação e preservação de dados. A identidade social é incluída nos próximos backups SQLite.

## Evidência real local

Os quatro modpacks recuperados tinham manifestos e configurações, mas os JARs não estavam completos após a reinstalação. O reparo nativo foi concluído para ATM10, Zombie Invade, RLCraft e Better MC. Essa conferência não executa o Minecraft e não substitui a confirmação do menu de cada jogo.

Após o reparo, o ATM10 contém 4.944 arquivos, 490 JARs diretamente em `mods` e 3.065.272.807 bytes únicos por identidade de arquivo. A nova leitura nativa encontrou esse valor em 2.236 ms; uma leitura independente em Python confirmou os mesmos bytes. O caminho antigo que abre cada arquivo individualmente ficou bloqueado na comparação e foi interrompido; não foi calculado um percentual de ganho.

Uma cópia privada do banco foi salva antes de atualizar os tamanhos e contagens das instâncias recuperadas. Contas, amizades, mundos existentes e assinatura não foram apagados. Os arquivos dos mundos anteriores que já estavam ausentes após a desinstalação não foram recuperados por este reparo.

## Verificação em andamento

- `pnpm check`: sem erros ou avisos.
- Vitest: 35 arquivos e 216 testes passaram após corrigir o fallback do seletor sem popover.
- Site: 101 testes passaram, incluindo os oito cenários de login Microsoft.
- Manifesto, assinatura e profundidade de padrões: 16 testes passaram.
- Teste visual novo: seletor longo dentro da janela, seleção por teclado e confirmação da skin apenas com ação explícita, em 1440 e 960 pixels.
- A bateria de navegador passou em execuções consecutivas retomadas após as correções: os 36 casos anteriores e os dois novos casos de recuperação e revisão da 4.0. O caso de wallpaper também foi repetido com o original indisponível, cópia interna persistida e nova porta após reiniciar. O teste separado de NameMC passou com a confirmação central.
- Rust: 265 testes passaram, nove integrações optativas ficaram ignoradas. O teste nativo explícito de armazenamento confirmou os mesmos 3.065.272.807 bytes em 1.095 ms com o cache do sistema aquecido. As últimas verificações específicas de seleção de cliente/servidor seguem em execução.
- Compilação de produção do frontend concluída. Publicação e instaladores finais ainda em andamento.

O armazenamento também inclui agora os arquivos mantidos no diretório de dados local do sistema, como wallpapers. Eles não pertencem ao cache descartável. O registro de origem dos wallpapers é interno e contém apenas a referência ao nome da cópia e seu tamanho; ele permite resolver um caminho antigo cujo original já foi removido. O teste nativo usa um MP4 real e confirma os bytes da cópia após remover o original e o cache de teste.

O login Microsoft do site depende das configurações Entra/Cloudflare descritas em `docs/plans/login-microsoft-site.md`; testes simulados não comprovam o login de produção. Nenhuma configuração de proteção do Windows foi alterada nesta revisão. Assinatura de manifesto não é assinatura Authenticode.
