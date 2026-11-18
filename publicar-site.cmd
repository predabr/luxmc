@echo off
title Luxmc - Publicar site (comprovantes)
cd /d "C:\Users\preda\OneDrive\Documenti\Luxmc"
set "PATH=C:\Users\preda\AppData\Local\Luxmc-build\tools\node-v22.23.4-win-x64;%PATH%"
"C:\Users\preda\AppData\Local\Luxmc-build\tools\node-v22.23.4-win-x64\node.exe" "C:\Users\preda\AppData\Local\Luxmc-build\tools\node_modules\pnpm\bin\pnpm.cjs" deploy:website
echo.
echo ================================================
echo Se o navegador pediu autorizacao do Cloudflare,
echo clique em "Allow" / "Permitir" para publicar.
echo ================================================
echo.
pause
