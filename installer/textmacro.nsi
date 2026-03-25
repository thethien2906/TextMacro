; ============================================================
;  TextMacro – NSIS Installer Script
;  Produces: TextMacro_Setup.exe
;
;  Requirements:
;    - NSIS 3.x (https://nsis.sourceforge.io/)
;    - Modern UI 2 plugin (bundled with NSIS)
; ============================================================

!define APP_NAME        "TextMacro"
!define APP_VERSION     "0.1.0"
!define APP_PUBLISHER   "TextMacro"
!define APP_URL         "https://github.com/thethien2906/TextMacro"
!define APP_EXE         "textmacro.exe"
!define INSTALL_DIR     "$PROGRAMFILES64\${APP_NAME}"
!define UNINSTALL_KEY   "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"

; ---------- Modern UI 2 ----------
!include "MUI2.nsh"

Name          "${APP_NAME} ${APP_VERSION}"
OutFile       "..\dist\TextMacro_Setup.exe"
InstallDir    "${INSTALL_DIR}"
InstallDirRegKey HKLM "${UNINSTALL_KEY}" "InstallLocation"

; Request admin rights so we can write to Program Files
RequestExecutionLevel admin

; ---------- UI Settings ----------
!define MUI_ABORTWARNING
!define MUI_ICON      "..\assets\logo.ico"
!define MUI_UNICON    "..\assets\logo.ico"

; Welcome page
!insertmacro MUI_PAGE_WELCOME
; License page (optional – comment out if you have no license file)
; !insertmacro MUI_PAGE_LICENSE "..\LICENSE"
; Directory page
!insertmacro MUI_PAGE_DIRECTORY
; Install-files page
!insertmacro MUI_PAGE_INSTFILES
; Finish page
!define MUI_FINISHPAGE_RUN        "$INSTDIR\${APP_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT   "Launch ${APP_NAME}"
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; ============================================================
;  Installer Section
; ============================================================
Section "Main Application" SEC_MAIN

    SetOutPath "$INSTDIR"

    ; --- Core binary ---
    File "..\target\release\${APP_EXE}"

    ; --- Assets ---
    SetOutPath "$INSTDIR\assets"
    File "..\assets\logo.png"
    ; Copy .ico if it exists (created by build script)
    IfFileExists "..\assets\logo.ico" 0 +2
        File "..\assets\logo.ico"

    ; --- Write uninstaller ---
    SetOutPath "$INSTDIR"
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; --- Start Menu shortcut ---
    CreateDirectory "$SMPROGRAMS\${APP_NAME}"
    CreateShortcut  "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" \
                    "$INSTDIR\${APP_EXE}" "" "$INSTDIR\${APP_EXE}" 0
    CreateShortcut  "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk" \
                    "$INSTDIR\Uninstall.exe"

    ; --- Desktop shortcut ---
    CreateShortcut "$DESKTOP\${APP_NAME}.lnk" \
                   "$INSTDIR\${APP_EXE}" "" "$INSTDIR\${APP_EXE}" 0

    ; --- Add/Remove Programs registry ---
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "DisplayName"      "${APP_NAME}"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "DisplayVersion"   "${APP_VERSION}"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "Publisher"        "${APP_PUBLISHER}"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "URLInfoAbout"     "${APP_URL}"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "InstallLocation"  "$INSTDIR"
    WriteRegStr   HKLM "${UNINSTALL_KEY}" "UninstallString"  "$INSTDIR\Uninstall.exe"
    WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify"         1
    WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair"         1

SectionEnd

; ============================================================
;  Uninstaller Section
; ============================================================
Section "Uninstall"

    ; Remove files
    Delete "$INSTDIR\${APP_EXE}"
    Delete "$INSTDIR\Uninstall.exe"
    Delete "$INSTDIR\assets\logo.png"
    Delete "$INSTDIR\assets\logo.ico"
    RMDir  "$INSTDIR\assets"
    RMDir  "$INSTDIR"

    ; Remove shortcuts
    Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
    Delete "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk"
    RMDir  "$SMPROGRAMS\${APP_NAME}"
    Delete "$DESKTOP\${APP_NAME}.lnk"

    ; Remove registry entries
    DeleteRegKey HKLM "${UNINSTALL_KEY}"

SectionEnd
