@echo off
setlocal
set "SCRIPT_DIR=%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%FsDeleteJpgRaw.ps1" -Path "%~1" -Mode Bin
endlocal
