$ErrorActionPreference='Stop'
Set-Location -LiteralPath 'C:/Users/preda/OneDrive/Documenti/Luxmc'
$env:CARGO_TARGET_DIR='C:/Users/preda/AppData/Local/Luxmc-build/target'
$taskNode='C:/Users/preda/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node.exe'
& 'C:/Users/preda/.cargo/bin/cargo.exe' check -j 1 --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $taskNode node_modules/typescript/bin/tsc -p site-server/tsconfig.json
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $taskNode node_modules/typescript/bin/tsc -p site-client/tsconfig.json
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $taskNode scripts/build-site-server.mjs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $taskNode scripts/build-site-client.mjs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $taskNode scripts/build-website-visuals.mjs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $taskNode --test tests/social.test.mjs tests/accounts.test.mjs tests/protocol.test.mjs tests/catalog.test.mjs tests/website-ui.test.mjs tests/downloads.test.mjs tests/profile-media.test.mjs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
'passed' | Set-Content -LiteralPath 'docs/validation/catalog-profile-groups/light-validation.status'
