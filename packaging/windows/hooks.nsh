!define MUI_WELCOMEPAGE_TITLE "Bem-vindo ao Lux MC Launcher"
!define MUI_WELCOMEPAGE_TEXT "Seu próximo mundo começa aqui.$\r$\n$\r$\nInstale o launcher, escolha sua conta e organize Minecraft, mods e amigos em um só lugar.$\r$\n$\r$\nAtualizar o Luxmc preserva suas contas, skins, instâncias e mundos.$\r$\n$\r$\nO Java e os arquivos do Minecraft são preparados quando você escolher sua primeira instância.$\r$\n$\r$\nClique em Avançar para continuar."
!define MUI_FINISHPAGE_TITLE "Lux MC Launcher instalado!"
!define MUI_FINISHPAGE_TEXT "O launcher está pronto.$\r$\n$\r$\n1. Abra o Luxmc e conecte sua conta.$\r$\n2. Escolha uma versão ou um modpack.$\r$\n3. Clique em Jogar.$\r$\n$\r$\nNo primeiro início, aguarde a preparação do Java e dos arquivos do jogo. Os próximos inícios reutilizam os arquivos já instalados."
!define MUI_FINISHPAGE_RUN_TEXT "Abrir Lux MC Launcher"
!define MUI_ABORTWARNING

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKCU "${UNINSTKEY}" "QuietUninstallString" "$\"$INSTDIR\uninstall.exe$\" /S"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  SetOutPath "$TEMP"
  !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
  ClearErrors
  Delete "$INSTDIR\${MAINBINARYNAME}.exe"
  ${If} ${Errors}
    Sleep 1000
    ClearErrors
    Delete /REBOOTOK "$INSTDIR\${MAINBINARYNAME}.exe"
    ${If} ${Errors}
      SetErrorLevel 1
      IfSilent +2
        MessageBox MB_OK|MB_ICONSTOP "Não foi possível remover o Luxmc. Feche o launcher e tente novamente. Seus dados foram preservados."
      Abort
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  IfRebootFlag 0 luxmc_uninstall_done
    IfSilent luxmc_uninstall_done
    MessageBox MB_OK|MB_ICONINFORMATION "A remoção do Luxmc será concluída ao reiniciar o Windows. Suas instâncias e seus mundos foram preservados."
  luxmc_uninstall_done:
!macroend
