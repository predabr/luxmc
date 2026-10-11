# Diagnóstico de confiança do Windows — Luxmc 3.8.0

Verificação em 11 de outubro de 2026. Este relatório trata do instalador já publicado; não declara sua aprovação pelos fornecedores de antivírus nem compatibilidade com o Controle Inteligente de Aplicativos.

| Campo | Resultado |
| --- | --- |
| Arquivo | `Lux MC Launcher.exe` |
| Tamanho | 21.371.759 bytes |
| SHA-256 | `e7c1ed8b3ec9971bb32c9068d66a7db9cee1d793c1e6b6a27f7c9c2cac19f017` |
| Authenticode | `NotSigned` |
| Política de bloqueio nos eventos 3077 | `VerifiedAndReputableDesktop` |
| Status registrado pelo Windows | `0xc0e90002` |
| Defender, análise personalizada do instalador | Nenhuma ameaça detectada, código de saída 0 |
| Definições do Defender utilizadas | `1.459.660.0` |
| Certificado de assinatura de código configurado | Ausente |

O arquivo na pasta Downloads do usuário tem o mesmo tamanho e SHA-256 do instalador oficial validado anteriormente. A revisão dos hooks NSIS e do auxiliar de recuperação não encontrou comandos para desabilitar antivírus, alterar o Controle Inteligente ou criar exclusões. Essa revisão é limitada a esses componentes e não substitui análise independente do executável e de suas dependências.

## Detecções comunicadas pelo usuário

O print do VirusTotal mostra 3/68 motores sinalizando o arquivo:

| Fornecedor | Nome da detecção |
| --- | --- |
| Avast | `NSIS:MalwareX-gen [Misc]` |
| AVG | `NSIS:MalwareX-gen [Misc]` |
| Trapmine | `Suspicious.low.ml.score` |

São rótulos genéricos e heurísticos. A quantidade de motores e esses nomes não bastam para determinar se são falsos positivos. Não foi possível consultar o relatório completo do VirusTotal pelo navegador de pesquisa; os nomes acima vêm da captura enviada pelo usuário. [Relatório identificado pelo SHA-256](https://www.virustotal.com/gui/file/e7c1ed8b3ec9971bb32c9068d66a7db9cee1d793c1e6b6a27f7c9c2cac19f017/detection).

## Pendências para resolver o bloqueio

1. Obter um certificado público de assinatura de código ou um serviço elegível com validação da identidade do editor. O mantenedor confirmou que ainda não possui certificado. O código original atual migrou para GPLv3 para preparar uma candidatura à SignPath; o programa ainda não aprovou o projeto. Essa preparação não altera o instalador analisado.
2. Assinar e verificar todos os componentes descritos em [assinatura-windows.md](../assinatura-windows.md), mantendo a recuperação e a assinatura Ed25519 das atualizações.
3. Solicitar revisão das detecções aos fornecedores com o instalador original, SHA-256, origem e versão. Não foram enviados formulários nem mensagens em nome do usuário.
4. Validar a nova distribuição em um Windows com o Controle Inteligente ativo. A assinatura de editor não garante que um antivírus retire uma detecção.

[Requisitos de assinatura para o Controle Inteligente](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control), [opções de distribuição e assinatura](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options), [formulário de revisão da Avast](https://support.avast.com/en-ae/article/submit-file-url/).

O atualizador foi ajustado no código local para distinguir recusas de política do Windows (577, 1260 e 4551) de detecções do antivírus (225 e 226). Nesses casos ele informa que a atualização não foi aplicada. A alteração não desbloqueia o instalador nem modifica a proteção do PC e ainda não foi incorporada ao executável publicado.

Validação local: `cargo check --tests --locked` passou, incluindo a compilação dos três testes novos de classificação de erros. Os testes não foram executados nesta revisão; a execução local de binários de teste já havia sido bloqueada pela política de integridade. Não foi iniciado o instalador bloqueado, alterada a versão ou publicada uma nova release.
