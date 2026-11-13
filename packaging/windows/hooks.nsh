!macro NSIS_HOOK_POSTINSTALL
  WriteRegDWORD HKCU "${UNINSTKEY}" "LuxmcRevision" 0
  WriteRegStr HKCU "${UNINSTKEY}" "QuietUninstallString" "$\"$INSTDIR\uninstall.exe$\" /S"
  Delete "$INSTDIR\luxmc-repair.exe"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  SetOutPath "$TEMP"
  !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
  ${If} $UpdateMode != 1
    ClearErrors
    ${GetOptions} $CMDLINE "/PURGE" $R0
    ${IfNot} ${Errors}
      StrCpy $DeleteAppDataCheckboxState 1
    ${EndIf}
    ${If} $DeleteAppDataCheckboxState = 1
      IfSilent luxmc_purge_confirmed
        MessageBox MB_YESNO|MB_ICONEXCLAMATION|MB_DEFBUTTON2 "Apagar permanentemente contas salvas, identidade de amigos, instâncias, mods e TODOS os mundos do Luxmc? Os arquivos não irão para a Lixeira. Para apenas reinstalar, escolha Não e desmarque a opção de apagar dados." IDYES luxmc_purge_confirmed
        Abort
      luxmc_purge_confirmed:
      GetTempFileName $R4
      Delete $R4
      CreateDirectory $R4
      CopyFiles /SILENT "$INSTDIR\${MAINBINARYNAME}.exe" "$R4\luxmc-uninstall.exe"
      nsExec::ExecToStack '$\"$R4\luxmc-uninstall.exe$\" --uninstall-data'
      Pop $0
      Pop $1
      Delete "$R4\luxmc-uninstall.exe"
      RMDir $R4
      ${If} $0 != 0
        SetErrorLevel 1
        IfSilent +2
          MessageBox MB_OK|MB_ICONSTOP "Não foi possível apagar todos os dados do Luxmc. A desinstalação não foi concluída; feche o launcher e tente novamente. Documentos e projeto foram preservados."
        Abort
      ${EndIf}
    ${EndIf}
  ${EndIf}
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
    ${If} $DeleteAppDataCheckboxState = 1
      MessageBox MB_OK|MB_ICONINFORMATION "A remoção do Luxmc será concluída ao reiniciar o Windows. A remoção dos dados foi solicitada nesta desinstalação."
    ${Else}
      MessageBox MB_OK|MB_ICONINFORMATION "A remoção do Luxmc será concluída ao reiniciar o Windows. Suas instâncias e seus mundos foram preservados."
    ${EndIf}
  luxmc_uninstall_done:
!macroend
