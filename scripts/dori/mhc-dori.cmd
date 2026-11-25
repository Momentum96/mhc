@echo off
rem Windows counterpart of mhc-dori.sh: runs mhc.exe with HOME=%USERPROFILE%\.maho\home (mhc reads HOME before
rem USERPROFILE). Durable memory is a junction to the real %USERPROFILE%\.omo\memory; omo.jsonc and agent\AGENTS.md
rem are copied from the real ones on every start because file symlinks need a privilege this account may lack.
rem Directories of the real home are junctioned into the shadow home once by mhc-dori-setup.ps1.
rem Claude runs on omo's claude CLI account folders and omo's SDK claude.exe (owner decision 2026-11-25);
rem mhc-dori-sync-claude.ps1 copies omo's account entry into mhc's auth.json on every start.
setlocal
set "REAL=%USERPROFILE%"
set "SHADOW=%REAL%\.maho\home"
if not exist "%SHADOW%\.omo\agent" mkdir "%SHADOW%\.omo\agent"
if not exist "%SHADOW%\.omo\memory" mklink /J "%SHADOW%\.omo\memory" "%REAL%\.omo\memory" >nul
copy /Y "%REAL%\.omo\omo.jsonc" "%SHADOW%\.omo\omo.jsonc" >nul 2>&1
copy /Y "%REAL%\.omo\agent\AGENTS.md" "%SHADOW%\.omo\agent\AGENTS.md" >nul 2>&1
for %%p in (OMO_ SENPI_ PI_ MAHO_) do for /f "delims==" %%v in ('set %%p 2^>nul') do set "%%v="
set "HOME=%SHADOW%"
set "GIT_CONFIG_GLOBAL=%REAL%\.gitconfig"
set "PATH=%REAL%\.local\bin;%REAL%\.bun\bin;%PATH%"
set "MAHO_CLAUDE_ACCOUNT_ROOT=%REAL%\.omo\agent\anthropic-subscription-claude"
if "%CLAUDE_CODE_EXECUTABLE%"=="" for /d %%d in ("%REAL%\.bun\install\cache\@anthropic-ai\claude-agent-sdk-win32-x64@*") do if exist "%%d\claude.exe" set "SDK_CLAUDE=%%d\claude.exe"
if "%CLAUDE_CODE_EXECUTABLE%"=="" for /d %%d in ("%REAL%\.bun\install\global\node_modules\@anthropic-ai\claude-agent-sdk-win32-*") do if exist "%%d\claude.exe" set "SDK_CLAUDE=%%d\claude.exe"
if "%CLAUDE_CODE_EXECUTABLE%"=="" if not "%SDK_CLAUDE%"=="" set "CLAUDE_CODE_EXECUTABLE=%SDK_CLAUDE%"
rem The rpc host forwards only allow-listed variables (Path, not CLAUDE_CODE_EXECUTABLE), so its sessions find claude on Path.
if not "%SDK_CLAUDE%"=="" for %%F in ("%SDK_CLAUDE%") do set "PATH=%%~dpF;%PATH%"
powershell -NoProfile -ExecutionPolicy Bypass -File "%REAL%\.maho\dori-scripts\mhc-dori-sync-claude.ps1" >nul 2>&1
if "%MHC_BIN%"=="" set "MHC_BIN=%REAL%\.maho\dist\mhc\mhc.exe"
"%MHC_BIN%" %*
exit /b %ERRORLEVEL%
