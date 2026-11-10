!include "FileFunc.nsh"
!include "nsDialogs.nsh"
Var ExistingDir
Var Mode
Var ResetData
Var RepairRadio
Var UninstallRadio
Var ResetCheck
Var KeepShellForRepair
Var ExistingVersion
Var UpdateRepairButton
!macro PrepareHelper
  InitPluginsDir
  File /oname=$PLUGINSDIR\anemorph-maintenance.exe "${MAINTENANCE_EXE}"
!macroend
!macro UnregisterVideo EXT
  DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.${EXT}\shell\Anemorph"
!macroend

Function InitializeMaintenance
  !insertmacro PrepareHelper
  SetOutPath "$TEMP"
  StrCpy $Mode "repair"
  StrCpy $ResetData 0
  ReadRegStr $ExistingDir HKCU "Software\Anemorph" "InstallDir"
  ${If} $ExistingDir == ""
    ReadRegStr $ExistingDir HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "InstallLocation"
  ${EndIf}
  ${If} $ExistingDir != ""
    ReadRegStr $ExistingVersion HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "DisplayVersion"
    ReadRegStr $0 HKCU "Software\Anemorph" "BuildId"
    ${If} $ExistingVersion != "${APP_VERSION}"
      StrCpy $Mode "update"
    ${ElseIf} $0 != ""
      ${If} $0 != "${APP_BUILD_ID}"
        StrCpy $Mode "update"
      ${EndIf}
    ${Else}
      ; Older installers did not record a build ID. Compare executable times.
      GetFileTime "$ExistingDir\Anemorph.exe" $0 $1
      ${If} $0 != ${APP_FILE_TIME_HIGH}
      ${OrIf} $1 != ${APP_FILE_TIME_LOW}
        StrCpy $Mode "update"
      ${EndIf}
    ${EndIf}
  ${EndIf}
  ${GetParameters} $0
  ClearErrors
  ${GetOptions} $0 "/UNINSTALL" $1
  ${IfNot} ${Errors}
    StrCpy $Mode "uninstall"
  ${EndIf}
  ClearErrors
  ${GetOptions} $0 "/RESETSETTINGS" $1
  ${IfNot} ${Errors}
    StrCpy $ResetData 1
  ${EndIf}
  ${If} $Mode == "uninstall"
    ${If} $ExistingDir == ""
      MessageBox MB_OK|MB_ICONINFORMATION "Anemorph is not installed." /SD IDOK
      SetErrorLevel 0
      Quit
    ${EndIf}
    IfSilent 0 +2
      Call RunInstalledUninstaller
  ${EndIf}
FunctionEnd

Function UpdateCreate
  ${If} $Mode != "update"
    Abort
  ${EndIf}
  !insertmacro MUI_HEADER_TEXT "Update Anemorph" "Install this version and keep your settings."
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}
  ${NSD_CreateLabel} 0 0 100% 28u "Installed version: $ExistingVersion$\r$\nInstaller version: ${APP_VERSION}"
  Pop $0
  ${NSD_CreateLabel} 0 40u 100% 34u "A different version or build was found. Continue to replace the app files. Your settings, undo history, and videos are kept."
  Pop $0
  ${NSD_CreateButton} 0 94u 125u 24u "Repair or uninstall instead"
  Pop $UpdateRepairButton
  ${NSD_OnClick} $UpdateRepairButton UpdateChooseRepair
  GetDlgItem $0 $HWNDPARENT 1
  ${NSD_SetText} $0 "Update"
  nsDialogs::Show
FunctionEnd
Function UpdateChooseRepair
  Pop $0
  StrCpy $Mode "repair"
  GetDlgItem $0 $HWNDPARENT 1
  SendMessage $HWNDPARENT ${WM_COMMAND} 1 $0
FunctionEnd
Function UpdateLeave
  ; The separate maintenance page is shown only if Repair was chosen.
  GetDlgItem $0 $HWNDPARENT 1
  ${NSD_SetText} $0 "Next >"
FunctionEnd

Function MaintenanceCreate
  ${If} $ExistingDir == ""
    Abort
  ${EndIf}
  ${If} $Mode == "update"
    Abort
  ${EndIf}
  !insertmacro MUI_HEADER_TEXT "Anemorph is already installed" "Repair the installation or remove Anemorph."
  nsDialogs::Create 1018
  Pop $0
  ${If} $0 == error
    Abort
  ${EndIf}
  ${NSD_CreateLabel} 0 0 100% 28u "Installed in:$\r$\n$ExistingDir"
  Pop $0
  ${NSD_CreateRadioButton} 0 38u 100% 14u "Repair / reinstall"
  Pop $RepairRadio
  ${NSD_CreateLabel} 12u 55u 95% 20u "Replace app files and restore shortcuts and Explorer integration."
  Pop $0
  ${NSD_CreateCheckbox} 12u 79u 95% 14u "Reset settings for a fresh start"
  Pop $ResetCheck
  ${NSD_SetState} $ResetCheck $ResetData
  ${NSD_CreateRadioButton} 0 108u 100% 14u "Uninstall"
  Pop $UninstallRadio
  ${NSD_OnClick} $RepairRadio MaintenanceModeChanged
  ${NSD_OnClick} $UninstallRadio MaintenanceModeChanged
  ${NSD_CreateLabel} 12u 125u 95% 28u "Remove the app and all saved settings, undo history, markers, and integration."
  Pop $0
  ${If} $Mode == "uninstall"
    ${NSD_Check} $UninstallRadio
    EnableWindow $ResetCheck 0
  ${Else}
    ${NSD_Check} $RepairRadio
  ${EndIf}
  nsDialogs::Show
FunctionEnd
Function MaintenanceModeChanged
  Pop $0
  ${NSD_GetState} $UninstallRadio $0
  ${If} $0 == ${BST_CHECKED}
    EnableWindow $ResetCheck 0
  ${Else}
    EnableWindow $ResetCheck 1
  ${EndIf}
FunctionEnd
Function MaintenanceLeave
  ${NSD_GetState} $UninstallRadio $0
  ${If} $0 == ${BST_CHECKED}
    Call RunInstalledUninstaller
  ${EndIf}
  ${NSD_GetState} $ResetCheck $ResetData
FunctionEnd

Function RunInstalledUninstaller
  Call RequireClosed
  Call UnregisterModern
  Call CleanIntegrationTrust
  ; Use bundled cleanup rather than relying on a possibly broken or older uninstaller.
  Push $ExistingDir
  Call RemoveProgramFiles
  Call CleanData
  Call RemoveRegistration
  MessageBox MB_OK|MB_ICONINFORMATION "Anemorph has been removed." /SD IDOK
  SetErrorLevel 0
  Quit
FunctionEnd

!macro DefineMaintenance PREFIX
Function ${PREFIX}CleanIntegrationTrust
  nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" clean-integration-trust'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Uninstall was stopped before removing app files or settings.$\r$\nWindows approval is needed to remove the integration certificate Anemorph added.$\r$\n$1" /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
FunctionEnd
Function ${PREFIX}UnregisterModern
  nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" unregister-menu'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Windows could not remove Anemorph's menu identity.$\r$\n$1" /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
FunctionEnd
Function ${PREFIX}RequireClosed
  retry_close:
  nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" check-running'
  Pop $0
  Pop $1
  ${If} $0 != 0
    IfSilent close_abort
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Close Anemorph before continuing, then click Retry." IDRETRY retry_close
    close_abort:
    SetErrorLevel 1
    Abort
  ${EndIf}
FunctionEnd
Function ${PREFIX}CleanData
  retry_data:
  nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" clean-data'
  Pop $0
  Pop $1
  ${If} $0 != 0
    IfSilent data_abort
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Saved data could not be fully removed.$\r$\n$1$\r$\nClose Anemorph, then click Retry." IDRETRY retry_data
    data_abort:
    SetErrorLevel 1
    Abort
  ${EndIf}
FunctionEnd
Function ${PREFIX}RemoveProgramFiles
  Pop $2
  GetFullPathName $2 "$2"
  ${GetRoot} "$2" $3
  ${If} $2 == ""
  ${OrIf} $2 == "$3\"
  ${OrIf} $2 == "$WINDIR"
  ${OrIf} $2 == "$PROFILE"
  ${OrIf} $2 == "$LOCALAPPDATA"
  ${OrIf} $2 == "$APPDATA"
  ${OrIf} $2 == "$PROGRAMFILES"
  ${OrIf} $2 == "$PROGRAMFILES64"
  ${OrIf} $2 == "$DOCUMENTS"
  ${OrIf} $2 == "$DESKTOP"
    MessageBox MB_OK|MB_ICONSTOP "Refusing to remove files from an invalid installation folder." /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
  retry_files:
  nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" stop-broker $\"$2$\"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "$1" /SD IDOK
    Goto files_abort
  ${EndIf}
  ${If} $KeepShellForRepair != 1
    nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" release-shell $\"$2$\"'
    Pop $0
    Pop $1
    ${If} $0 != 0
      IfSilent files_abort
      MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Menu files could not be removed. No Windows applications were stopped.$\r$\n$1" IDRETRY retry_files
      Goto files_abort
    ${EndIf}
  ${Else}
    nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" prune-legacy $\"$2$\"'
    Pop $0
    Pop $1
    DetailPrint "$1"
  ${EndIf}
  Delete "$2\Anemorph.exe"
  Delete "$2\anemorph-maintenance.exe"
  Delete "$2\anemorph-broker.exe"
  ${If} $KeepShellForRepair != 1
    Delete "$2\anemorph-shell-*.dll"
    ${If} ${FileExists} "$2\anemorph-shell-*.dll"
      Goto files_locked
    ${EndIf}
  ${EndIf}
  Delete "$2\LICENSE"
  Delete "$2\README.md"
  Delete "$2\CHANGELOG.md"
  Delete "$2\THIRD_PARTY_NOTICES.md"
  Delete "$2\DEPENDENCY_LICENSES.md"
  Delete "$2\NSIS-LICENSE.txt"
  Delete "$2\assets\screenshots\home.jpg"
  Delete "$2\assets\screenshots\explorer-menu.png"
  Delete "$2\assets\screenshots\linux.webp"
  RMDir "$2\assets\screenshots"
  RMDir "$2\assets"
  Delete "$2\docs\usage.md"
  Delete "$2\docs\building.md"
  Delete "$2\docs\performance.md"
  Delete "$2\docs\signing.md"
  Delete "$2\docs\contributing.md"
  Delete "$2\docs\changelog.md"
  Delete "$2\docs\licenses\third-party-notices.md"
  Delete "$2\docs\licenses\dependency-licenses.md"
  RMDir "$2\docs\licenses"
  RMDir "$2\docs"
  Delete "$2\Anemorph.Integration\Anemorph.identity.msix"
  Delete "$2\Anemorph.Integration\icon.png"
  RMDir "$2\Anemorph.Integration"
  ${If} ${FileExists} "$2\Anemorph.exe"
  ${OrIf} ${FileExists} "$2\anemorph-maintenance.exe"
  ${OrIf} ${FileExists} "$2\anemorph-broker.exe"
  ${OrIf} ${FileExists} "$2\LICENSE"
  ${OrIf} ${FileExists} "$2\README.md"
  ${OrIf} ${FileExists} "$2\CHANGELOG.md"
  ${OrIf} ${FileExists} "$2\THIRD_PARTY_NOTICES.md"
  ${OrIf} ${FileExists} "$2\DEPENDENCY_LICENSES.md"
  ${OrIf} ${FileExists} "$2\NSIS-LICENSE.txt"
  ${OrIf} ${FileExists} "$2\assets\screenshots\home.jpg"
  ${OrIf} ${FileExists} "$2\assets\screenshots\explorer-menu.png"
  ${OrIf} ${FileExists} "$2\assets\screenshots\linux.webp"
  ${OrIf} ${FileExists} "$2\docs\usage.md"
  ${OrIf} ${FileExists} "$2\docs\building.md"
  ${OrIf} ${FileExists} "$2\docs\performance.md"
  ${OrIf} ${FileExists} "$2\docs\signing.md"
  ${OrIf} ${FileExists} "$2\docs\contributing.md"
  ${OrIf} ${FileExists} "$2\docs\changelog.md"
  ${OrIf} ${FileExists} "$2\docs\licenses\third-party-notices.md"
  ${OrIf} ${FileExists} "$2\docs\licenses\dependency-licenses.md"
  ${OrIf} ${FileExists} "$2\Anemorph.Integration\Anemorph.identity.msix"
  ${OrIf} ${FileExists} "$2\Anemorph.Integration\icon.png"
    files_locked:
    IfSilent files_abort
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Anemorph could not remove all app files in:$\r$\n$2$\r$\nClose any applications using this folder, then retry." IDRETRY retry_files
    files_abort:
    SetErrorLevel 1
    Abort
  ${EndIf}
  Delete "$2\Uninstall.exe"
  RMDir "$2"
FunctionEnd
Function ${PREFIX}RemoveRegistration
  nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" clean-leftovers'
  Pop $0
  Pop $1
  DetailPrint "$1"
  ${If} $0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Some Anemorph leftovers could not be removed.$\r$\n$1$\r$\nClose Explorer windows using Anemorph and retry uninstall." /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
  nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" clean-local-signing'
  Pop $0
  Pop $1
  DetailPrint "$1"
  ${If} $0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Anemorph's local signing identity could not be fully removed.$\r$\n$1" /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
  !include "${UNREGISTER_INCLUDE}"
  DeleteRegKey HKCU "Software\Classes\CLSID\{B8A9BDB0-03E4-4F1A-BDEA-28654A8F7DB4}"
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph"
  DeleteRegKey HKCU "Software\Anemorph"
  Delete "$SMPROGRAMS\Anemorph\Anemorph.lnk"
  Delete "$SMPROGRAMS\Anemorph\Uninstall Anemorph.lnk"
  RMDir "$SMPROGRAMS\Anemorph"
  Delete "$DESKTOP\Anemorph.lnk"
FunctionEnd
!macroend
!insertmacro DefineMaintenance ""
!insertmacro DefineMaintenance "un."

Function .onVerifyInstDir
  ${GetRoot} "$INSTDIR" $0
  ${If} $INSTDIR == "$0\"
  ${OrIf} $INSTDIR == "$WINDIR"
  ${OrIf} $INSTDIR == "$PROFILE"
  ${OrIf} $INSTDIR == "$LOCALAPPDATA"
  ${OrIf} $INSTDIR == "$APPDATA"
  ${OrIf} $INSTDIR == "$PROGRAMFILES"
  ${OrIf} $INSTDIR == "$PROGRAMFILES64"
  ${OrIf} $INSTDIR == "$DOCUMENTS"
  ${OrIf} $INSTDIR == "$DESKTOP"
    Abort
  ${EndIf}
FunctionEnd
