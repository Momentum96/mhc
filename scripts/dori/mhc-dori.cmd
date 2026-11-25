@echo off
rem Windows counterpart of mhc-dori.sh: runs mhc.exe with HOME=%USERPROFILE%\.maho\home (mhc reads HOME before
rem USERPROFILE). Durable memory is a junction to the real %USERPROFILE%\.omo\memory; omo.jsonc and agent\AGENTS.md
rem are copied from the real ones on every start because file symlinks need a privilege this account may lack.
rem Directories of the real home are junctioned into the shadow home once by mhc-dori-setup.ps1.
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
set "MAHO_CLAUDE_ACCOUNT_ROOT=%SHADOW%\.omo\agent\anthropic-subscription-claude"
if "%MHC_BIN%"=="" set "MHC_BIN=%REAL%\.maho\dist\mhc\mhc.exe"
"%MHC_BIN%" %*
exit /b %ERRORLEVEL%
