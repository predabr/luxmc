# Assinatura e distribuição do Lux MC Launcher

Verificado em 12 de outubro de 2026. O instalador publicado é `Lux MC Launcher.exe` (NSIS). O código original atual migrou para `GPL-3.0-only`; tags e binários anteriores preservam seus registros históricos de licença. A SignPath Foundation recusou a candidatura por falta dos sinais de adoção e reconhecimento público exigidos pelo programa. Nenhum certificado está ativo.

## Bloqueio do Lux Repair na compilação local 3.9.1

Em 12 de outubro, os eventos 3033 e 3077 do Code Integrity registraram o bloqueio de `luxmc-repair.exe` às 11:57, quando o Raycast tentou abrir o Luxmc. Naquela compilação, os atalhos iniciavam esse auxiliar antes do programa principal. O mantenedor solicitou sua remoção: o código atual não compila nem inclui Lux Repair, e atalhos e protocolos executam o launcher diretamente. A limpeza explicitamente solicitada na desinstalação foi integrada ao programa principal, sem abrir sua interface. A assinatura de editor continua necessária para os componentes restantes.

O SHA-256 do auxiliar instalado, `e4ff1ffc8f06755c8fe74fbb887c1692cd210c2ae537f4a56668ba5dc4574f87`, corresponde ao artefato da [compilação verificável](https://github.com/predabr/luxmc/actions/runs/38264425892). O programa principal e o auxiliar apresentam Authenticode `NotSigned`. Os mesmos bytes foram examinados pelo Defender sem detecções no relatório `validation/modpack-followup/windows-local-audit.json`; isso não estabelece confiança de editor nem comprova ausência de malware.

O pacote local de revisão em `validation/windows-repair-trust/` contém somente o auxiliar e metadados de origem daquela compilação. Sua preparação não significa envio ou aprovação pela Microsoft. O diagnóstico inicial não alterou atalhos ou executáveis. A remoção posterior foi solicitada pelo mantenedor; políticas do Windows e recursos de proteção foram preservados. A compatibilidade permanece não confirmada até que os componentes restantes sejam aceitos pela política do Windows e o ciclo de instalação e execução seja testado.

## Situação da versão 3.8.0

O instalador publicado não tem assinatura Authenticode. O Controle Inteligente de Aplicativos bloqueou sua execução com a política `VerifiedAndReputableDesktop`. A assinatura Ed25519 dos manifestos e a conferência do SHA-256 continuam válidas, mas não identificam o editor para o Windows. Um download íntegro pode ser bloqueado por falta de confiança.

O usuário informou três detecções no VirusTotal para o mesmo SHA-256 do instalador oficial. A verificação local do Defender não encontrou ameaças. Esses resultados divergentes não confirmam nem descartam as detecções dos demais fornecedores; a análise precisa continuar com eles. Veja o [relatório de diagnóstico](releases/windows-trust-v3.8.0.md).

Não existe certificado de assinatura de código configurado no projeto. Publicar novamente o mesmo instalador, renomeá-lo ou criar um certificado autoassinado não resolve sua confiança em PCs com o Controle Inteligente ativo. A Microsoft exige assinatura de editor por um fornecedor confiável para esse caminho de distribuição. [Requisitos oficiais](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control).

## Preparação da publicação com Authenticode

A integração deve assinar o programa principal, os plugins DLL utilizados pelo NSIS, o desinstalador e o instalador final. O template já preserva `signed_plugins_path` e `uninstaller_sign_cmd` do Tauri. O hook adicional utiliza `nsExec.dll`, que também precisa entrar na verificação de assinatura; conferir somente o EXE externo é insuficiente.

A release só estará validada quando instalação, atualização e desinstalação funcionarem com os executáveis efetivamente assinados e o Controle Inteligente habilitado. As assinaturas Ed25519 e os hashes das atualizações continuam independentes da remoção do Repair. Os testes atuais em runners não comprovam essa compatibilidade.

Depois da assinatura final, gerar novamente `SHA256SUMS`, `latest.json` e suas assinaturas Ed25519, pois os bytes e os hashes mudam. Não substituir silenciosamente o instalador da release publicada.

## Alternativa pela Microsoft Store

A Microsoft oferece assinatura automática de pacotes **MSIX** enviados à Microsoft Store após certificação. O caminho é criar uma conta de desenvolvedor, preparar o pacote MSIX, cadastrar o aplicativo no Partner Center e enviar para análise. Isso exige uma etapa de empacotamento e verificação específica do Luxmc, incluindo gravação das instâncias fora do diretório instalado, atualização e integração `luxmc://`.

Enviar o nosso instalador EXE para a Store não recebe essa assinatura automática: EXE/MSI e seus executáveis continuam exigindo assinatura do publicador. [Documentação oficial da Microsoft](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options).

## Caminho gratuito para EXE, caso o projeto seja elegível

A SignPath Foundation oferece assinatura gratuita a projetos de código aberto qualificados. Exige licença aprovada pela OSI, ausência de componentes proprietários, manutenção ativa e construção verificável a partir do repositório. A migração do código original para GPLv3 foi autorizada para seguir esse caminho. Dependências preservam suas licenças; imagens de terceiros e o runtime de sistema precisam de avaliação do programa. Não devemos presumir elegibilidade ou aprovação. [Requisitos da SignPath](https://signpath.org/terms).

O formulário de [solicitação](https://signpath.org/apply) foi enviado com os dados fornecidos pelo mantenedor, após sua autorização para aceitar os termos e o tratamento dos dados de contato. A política pública está em [CODE_SIGNING_POLICY.md](../CODE_SIGNING_POLICY.md) e as configurações de artefatos para avaliação estão em [.signpath/](../.signpath/README.md). A integração final exige aprovação do programa e credenciais próprias; ainda não foi ativada. Cada release precisa de aprovação manual. A aceitação não é automática.

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

O formulário exibiu “Form submitted” e “Thank you, we'll be in touch soon.” O envio da candidatura não significa aprovação do programa nem emissão de certificado. Nenhum certificado autoassinado foi criado para apresentá-lo como assinatura confiável.
