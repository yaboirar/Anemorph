Unicode true
RequestExecutionLevel user
ManifestDPIAware true
SetCompressor /SOLID lzma
!ifdef RELEASE_CERT
!uninstfinalize '"${SIGN_POWERSHELL}" -NoProfile -NonInteractive -File "${SIGN_SCRIPT}" -CertificateThumbprint "${RELEASE_CERT}" -SignTool "${SIGN_TOOL}" -Files "%1"' = 0
!endif
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "Sections.nsh"
!include "WinVer.nsh"
!include "maintenance.nsh"
!ifndef APP_VERSION
!error "Build with installer/build.ps1"
!endif
!define APP_CLSID "{B8A9BDB0-03E4-4F1A-BDEA-28654A8F7DB4}"
Name "Anemorph"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\Anemorph"
InstallDirRegKey HKCU "Software\Anemorph" "InstallDir"
BrandingText "Anemorph ${APP_VERSION}"
VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey "ProductName" "Anemorph"
VIAddVersionKey "FileDescription" "Anemorph installer"
VIAddVersionKey "FileVersion" "${APP_VERSION}"
VIAddVersionKey "ProductVersion" "${APP_VERSION}"
VIAddVersionKey "LegalCopyright" "Anemorph contributors"
!define MUI_ICON "..\assets\icon.ico"
!define MUI_UNICON "..\assets\icon.ico"
!define MUI_ABORTWARNING
Page custom MaintenanceCreate MaintenanceLeave
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_COMPONENTS
!define MUI_DIRECTORYPAGE_TEXT_TOP "Choose a dedicated folder for Anemorph. Click Browse to choose another drive or folder. Repair removes the old app files if you choose a new location."
!define MUI_DIRECTORYPAGE_TEXT_DESTINATION "Installation folder"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\Anemorph.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Open Anemorph"
!insertmacro MUI_PAGE_FINISH
!define MUI_UNCONFIRMPAGE_TEXT_TOP "This removes Anemorph, its settings, undo history, markers, selection files, shortcuts, and Explorer integration. Your video files are kept."
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Function .onInit
  ${IfNot} ${RunningX64}
    MessageBox MB_ICONSTOP "Anemorph requires 64-bit Windows."
    Abort
  ${EndIf}
  SetRegView 64
  SetShellVarContext current
  Call InitializeMaintenance
FunctionEnd
Function un.onInit
  SetRegView 64
  SetShellVarContext current
  !insertmacro PrepareHelper
  SetOutPath "$TEMP"
  ${GetParameters} $0
  ClearErrors
  ${GetOptions} $0 "/FROMAPP=" $1
  ${IfNot} ${Errors}
    ; Let Anemorph finish saving and exit before any cleanup can begin.
    System::Call 'kernel32::OpenProcess(i 0x100000, i 0, i r1) p.r2'
    ${If} $2 != 0
      System::Call 'kernel32::WaitForSingleObject(p r2, i 15000) i.r3'
      System::Call 'kernel32::CloseHandle(p r2)'
      ${If} $3 != 0
        MessageBox MB_OK|MB_ICONEXCLAMATION "Anemorph has not finished closing. Please close it and run uninstall again." /SD IDOK
        SetErrorLevel 1
        Quit
      ${EndIf}
    ${EndIf}
  ${EndIf}
FunctionEnd

Section "Anemorph" Core
  SectionIn RO
  Call .onVerifyInstDir
  Call RequireClosed
  Call UnregisterModern
  ${If} $ExistingDir != ""
    WriteRegStr HKCU "Software\Anemorph\InstallLocations" "$ExistingDir" ""
    ${If} $ExistingDir == $INSTDIR
      StrCpy $KeepShellForRepair 1
    ${EndIf}
    Push $ExistingDir
    Call RemoveProgramFiles
    StrCpy $KeepShellForRepair 0
  ${EndIf}
  ${If} $ResetData == 1
    Call CleanData
  ${EndIf}
  SetOverwrite on
  SetOutPath "$INSTDIR"
  File /oname=Anemorph.exe "${APP_EXE}"
  File /oname=anemorph-maintenance.exe "${MAINTENANCE_EXE}"
  File /oname=anemorph-broker.exe "${BROKER_EXE}"
  File "..\LICENSE"
  File "..\README.md"
  File "NSIS-LICENSE.txt"
  SetOutPath "$INSTDIR\docs"
  File "..\docs\*.md"
  SetOutPath "$INSTDIR\docs\licenses"
  File "..\docs\licenses\*.md"
  SetOutPath "$INSTDIR\assets\screenshots"
  File "..\assets\screenshots\home.jpg"
  File "..\assets\screenshots\explorer-menu.png"
  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\Anemorph"
  CreateShortcut "$SMPROGRAMS\Anemorph\Anemorph.lnk" "$INSTDIR\Anemorph.exe"
  CreateShortcut "$SMPROGRAMS\Anemorph\Uninstall Anemorph.lnk" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Anemorph" "InstallDir" "$INSTDIR"
  DeleteRegValue HKCU "Software\Anemorph" "ShellDll"
  WriteRegStr HKCU "Software\Anemorph\InstallLocations" "$INSTDIR" ""
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "DisplayName" "Anemorph"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "Publisher" "yaboirar"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "DisplayIcon" "$INSTDIR\Anemorph.exe,0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "QuietUninstallString" '$\"$INSTDIR\Uninstall.exe$\" /S'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "URLInfoAbout" "https://github.com/yaboirar/Anemorph"
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "NoRepair" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Anemorph" "EstimatedSize" ${INSTALLED_KB}
SectionEnd

!macro RegisterVideo EXT
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\.${EXT}\shell\Anemorph" "" "Rename with Anemorph"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\.${EXT}\shell\Anemorph" "ExplorerCommandHandler" "${APP_CLSID}"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\.${EXT}\shell\Anemorph" "Icon" "$INSTDIR\Anemorph.exe,0"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\.${EXT}\shell\Anemorph" "MultiSelectModel" "Player"
!macroend

Section -ExplorerIntegration
  SectionIn RO
  SetOutPath "$INSTDIR"
  ; Both menu modes activate the separate broker; no DLL is loaded into Explorer.
!ifdef MODERN_PACKAGE
  SetOutPath "$INSTDIR\Anemorph.Integration"
  File /oname=Anemorph.identity.msix "${MODERN_PACKAGE}"
  File /oname=icon.png "..\assets\icon.png"
  SetOutPath "$INSTDIR"
  ${If} ${AtLeastWin11}
  nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" register-modern $\"$INSTDIR$\"'
  Pop $0
  Pop $1
  DetailPrint "$1"
  ${If} $0 != 0
    DetailPrint "First-menu integration is unavailable. Installing the classic Explorer menu instead."
  ${EndIf}
  ${EndIf}
!endif
  nsExec::ExecToStack '$\"$PLUGINSDIR\anemorph-maintenance.exe$\" sync-menu $\"$INSTDIR$\"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Explorer integration could not be configured.$\r$\n$1" /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
SectionEnd

Section /o "Desktop shortcut" Desktop
  CreateShortcut "$DESKTOP\Anemorph.lnk" "$INSTDIR\Anemorph.exe"
SectionEnd


!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${Core} "Install Anemorph with its built-in File Explorer integration. Reset saved data only if you choose a fresh start."
  !insertmacro MUI_DESCRIPTION_TEXT ${Desktop} "Add an Anemorph shortcut to your desktop."
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
  Call un.RequireClosed
  Call un.UnregisterModern
  Call un.CleanIntegrationTrust
  Push $INSTDIR
  Call un.RemoveProgramFiles
  Call un.CleanData
  Call un.RemoveRegistration
SectionEnd
