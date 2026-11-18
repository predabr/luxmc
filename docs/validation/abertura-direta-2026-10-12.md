# Luxmc 3.9.1 — Abertura Direta

O mantenedor solicitou retirar Lux Repair depois de o Controle Inteligente bloquear esse auxiliar. O componente e seu processo de recuperação automática foram removidos do código, da compilação, do empacotamento, das configurações de assinatura e dos workflows ativos. Não foi alterada a política de execução do Windows.

Atalhos, associações de arquivos, links `luxmc://` e abertura ao finalizar a instalação usam `luxmc.exe`. A instalação reconhece atalhos de versões anteriores que apontavam para Lux Repair, migra esses atalhos e remove o auxiliar antigo. As verificações Ed25519 e SHA-256 das atualizações foram preservadas, assim como a proteção de arquivos de mods.

A rotina de limpeza da desinstalação foi movida para `core/uninstall.rs`. O modo explícito `--uninstall-data` termina antes de criar o runtime ou abrir a interface. O desinstalador continua preservando dados por padrão, e só executa essa limpeza quando o usuário escolhe apagar dados. Atualizações não executam a limpeza.

## Verificação

- `cargo check --release --features tauri/custom-protocol --locked`: passou, com cinco avisos preexistentes.
- Testes reais do módulo de limpeza, compilados com `rustc --test -C lto=thin`: três passaram. Conferem preservação de documentos e outros aplicativos, recusa de caminhos que contenham documentos e restrição das identidades Bedrock.
- Testes de manifestos e assinaturas da distribuição: 12 passaram, incluindo recusa de adulteração e chaves diferentes da oficial.
- Scripts PowerShell de instalação e ciclo do instalador: analisados sem erros de sintaxe.
- O teste de ciclo do instalador foi ampliado para instalação sem auxiliar, migração de atalhos antigos e links diretos. Esse ciclo ainda não foi executado nesta máquina, que possui instalação e dados reais; sua limpeza completa exige um ambiente descartável.

O instalador local mantém o número 3.9.1 e o nome **Lux MC Launcher - Abertura Direta.exe**. A release pública existente não deve ser substituída silenciosamente. Resultados de compilação, pacote, Defender e instalação são registrados em `windows-repair-trust/`.

## Aplicação local

A compilação de produção passou em 12 min 18 s. O NSIS foi gerado sem auxiliar; seu conteúdo foi extraído para conferência sem executar o instalador. O programa do pacote corresponde exatamente ao programa compilado, exceto pelos três bytes do marcador de tipo de bundle que o próprio Tauri altera de `UNK` para `NSS`. A diferença foi conferida por posição e conteúdo, e nenhum outro byte foi aceito como diferente.

O Defender não encontrou ameaças no instalador, programa e desinstalador. Os três continuam com Authenticode `NotSigned`. O programa e o desinstalador verificados foram aplicados na instalação existente; o auxiliar foi removido e dois atalhos foram migrados. Três arquivos de banco e configuração mantiveram os hashes anteriores. O launcher permaneceu fechado. O ciclo completo de instalação e a abertura real não foram executados.

- Executável instalado: SHA-256 `b13bbf8087b92091035edf421ba74e3c584b2e67a1af38c3efa112a71845cdcb`.
- Instalador local: SHA-256 `84fe3902b1013b5c26b54fdc49bb94d55d7c8be754545c1924d3bcd991c92dba`.
- Backup: `C:/Users/preda/AppData/Local/Luxmc-build/backups/abertura-direta-20261012-131658`.
- Instalador disponível em `C:/Users/preda/Downloads/Lux MC Launcher - Abertura Direta.exe`.

Nenhuma release, tag, arquivo público do GitHub ou publicação do site foi alterada nesta remoção.

Não há certificado Authenticode ativo. Remover um componente não estabelece confiança de editor para os componentes restantes. A abertura no Windows permanece não confirmada enquanto não houver teste de execução permitido pela política do sistema. O launcher não deve ser iniciado automaticamente para esse teste sem solicitação do mantenedor.

## Tentativa do mantenedor após a aplicação

O mantenedor tentou abrir o launcher às 13:19:41. Os eventos 3033 e 3077 confirmaram que o Raycast passou a executar `luxmc.exe`, mas o Controle Inteligente bloqueou o próprio programa principal por requisitos de assinatura/confiança. A remoção do Repair e a migração dos atalhos estão confirmadas; a abertura continua bloqueada. Não foi encontrada detecção recente do Luxmc nos registros consultados do Defender. Esse resultado não certifica ausência de malware. O diagnóstico está em `windows-repair-trust/main-blocked.json`.
