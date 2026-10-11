# Assinatura e distribuição do Lux MC Launcher

Verificado em 11 de outubro de 2026. O instalador publicado é `Lux MC Launcher.exe` (NSIS). O código original atual migrou para `GPL-3.0-only`; tags e binários anteriores preservam seus registros históricos de licença. A candidatura à SignPath está em preparação, sem aprovação ou certificado ativo.

## Situação da versão 3.8.0

O instalador publicado não tem assinatura Authenticode. O Controle Inteligente de Aplicativos bloqueou sua execução com a política `VerifiedAndReputableDesktop`. A assinatura Ed25519 dos manifestos e a conferência do SHA-256 continuam válidas, mas não identificam o editor para o Windows. Um download íntegro pode ser bloqueado por falta de confiança.

O usuário informou três detecções no VirusTotal para o mesmo SHA-256 do instalador oficial. A verificação local do Defender não encontrou ameaças. Esses resultados divergentes não confirmam nem descartam as detecções dos demais fornecedores; a análise precisa continuar com eles. Veja o [relatório de diagnóstico](releases/windows-trust-v3.8.0.md).

Não existe certificado de assinatura de código configurado no projeto. Publicar novamente o mesmo instalador, renomeá-lo ou criar um certificado autoassinado não resolve sua confiança em PCs com o Controle Inteligente ativo. A Microsoft exige assinatura de editor por um fornecedor confiável para esse caminho de distribuição. [Requisitos oficiais](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control).

## Preparação da publicação com Authenticode

A integração deve assinar o programa principal, o auxiliar `luxmc-repair.exe`, os plugins DLL utilizados pelo NSIS, o desinstalador e o instalador final. O template já preserva `signed_plugins_path` e `uninstaller_sign_cmd` do Tauri. O hook adicional utiliza `nsExec.dll`, que também precisa entrar na verificação de assinatura; conferir somente o EXE externo é insuficiente.

O auxiliar contém o hash de referência do programa principal. A ordem de assinatura e geração desse hash precisa preservar a verificação após o empacotamento. A release só estará validada quando instalação, atualização, recuperação e desinstalação funcionarem com os executáveis efetivamente assinados e o Controle Inteligente habilitado. Os testes atuais em runners não comprovam essa compatibilidade.

Depois da assinatura final, gerar novamente `SHA256SUMS`, `latest.json` e suas assinaturas Ed25519, pois os bytes e os hashes mudam. Não substituir silenciosamente o instalador da release publicada.

## Alternativa pela Microsoft Store

A Microsoft oferece assinatura automática de pacotes **MSIX** enviados à Microsoft Store após certificação. O caminho é criar uma conta de desenvolvedor, preparar o pacote MSIX, cadastrar o aplicativo no Partner Center e enviar para análise. Isso exige uma etapa de empacotamento e verificação específica do Luxmc, incluindo gravação das instâncias fora do diretório instalado, atualização e integração `luxmc://`.

Enviar o nosso instalador EXE para a Store não recebe essa assinatura automática: EXE/MSI e seus executáveis continuam exigindo assinatura do publicador. [Documentação oficial da Microsoft](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options).

## Caminho gratuito para EXE, caso o projeto seja elegível

A SignPath Foundation oferece assinatura gratuita a projetos de código aberto qualificados. Exige licença aprovada pela OSI, ausência de componentes proprietários, manutenção ativa e construção verificável a partir do repositório. A migração do código original para GPLv3 foi autorizada para seguir esse caminho. Dependências preservam suas licenças; imagens de terceiros e o runtime de sistema precisam de avaliação do programa. Não devemos presumir elegibilidade ou aprovação. [Requisitos da SignPath](https://signpath.org/terms).

O formulário de [solicitação](https://signpath.org/apply) está sendo preparado com os dados fornecidos pelo mantenedor. A política pública está em [CODE_SIGNING_POLICY.md](../CODE_SIGNING_POLICY.md) e as configurações de artefatos para avaliação estão em [.signpath/](../.signpath/README.md). A integração final exige aprovação do programa e credenciais próprias; ainda não foi ativada. Cada release precisa de aprovação manual. A aceitação não é automática.

## O que conferir em uma release assinada

Execute no PowerShell:

```powershell
Get-AuthenticodeSignature -LiteralPath "$env:LOCALAPPDATA\Luxmc\luxmc.exe" |
    Select-Object Status, StatusMessage, SignerCertificate
Get-AuthenticodeSignature -LiteralPath "$env:USERPROFILE\Downloads\Lux MC Launcher.exe" |
    Select-Object Status, StatusMessage, SignerCertificate
Get-FileHash -LiteralPath "$env:USERPROFILE\Downloads\Lux MC Launcher.exe" -Algorithm SHA256
```

Uma assinatura válida confirma a origem e a integridade. Certificado autoassinado serve para testes e não é confiável por padrão nos computadores dos amigos. Renomear o arquivo também não estabelece confiança. Assinatura não garante ausência de falso positivo ou avisos de reputação. [Explicação da Microsoft](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options).

Se o Defender detectar incorretamente uma release, envie o arquivo pelo formulário oficial como desenvolvedor de software, com a versão e o hash. Não é necessário orientar usuários a desligarem a proteção. [Enviar para análise da Microsoft](https://www.microsoft.com/en-us/wdsi/filesubmission).

Nenhum pedido foi submetido até esta preparação. O envio depende da revisão final dos termos pelo mantenedor. Nenhum certificado autoassinado foi criado para apresentá-lo como assinatura confiável.
