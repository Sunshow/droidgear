@echo off
setlocal EnableExtensions DisableDelayedExpansion
if not defined NODE_EXE set "NODE_EXE=node"
"%NODE_EXE%" "%~dp0scripts\copilot-local.js" %*
exit /b %ERRORLEVEL%
