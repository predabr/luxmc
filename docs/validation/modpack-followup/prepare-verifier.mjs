import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {githubRequest} from '../../../scripts/github-authenticated.mjs';
const repo='predabr/luxmc';
const main=readFileSync('C:/Users/preda/AppData/Local/Luxmc-build/target/release/luxmc.exe');
const marker=Buffer.from('PE_VAR_NSS');
for(let offset=main.indexOf(marker);offset>=0;offset=main.indexOf(marker,offset+marker.length)) Buffer.from('PE_VAR_UNK').copy(main,offset);
const sha256=createHash('sha256').update(main).digest('hex');
const branch='codex/modpacks-verifier-'+sha256.slice(0,8);
const base=await githubRequest(`repos/${repo}/git/ref/heads/main`);
await githubRequest(`repos/${repo}/git/refs`,{method:'POST',body:{ref:'refs/heads/'+branch,sha:base.object.sha}});
const workflow=`name: Local Luxmc integrity verifier
on:
  push:
    branches: [${branch}]
permissions:
  contents: read
jobs:
  verifier:
    runs-on: windows-latest
    timeout-minutes: 20
    env:
      LUXMC_MAIN_SHA256: '${sha256}'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri/repair-helper -> target
      - name: Test verifier
        run: cargo test --locked --manifest-path src-tauri/repair-helper/Cargo.toml
      - name: Build verifier
        run: cargo build --release --locked --target x86_64-pc-windows-msvc --manifest-path src-tauri/repair-helper/Cargo.toml
      - name: Record artifact digest
        shell: pwsh
        run: |
          $binary='src-tauri/repair-helper/target/x86_64-pc-windows-msvc/release/luxmc-repair.exe'
          @{mainSha256=$env:LUXMC_MAIN_SHA256;helperSha256=(Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant();commit=$env:GITHUB_SHA} | ConvertTo-Json | Set-Content verifier.json
      - uses: actions/upload-artifact@v4
        with:
          name: luxmc-local-verifier
          retention-days: 3
          path: |
            src-tauri/repair-helper/target/x86_64-pc-windows-msvc/release/luxmc-repair.exe
            verifier.json
`;
const result=await githubRequest(`repos/${repo}/contents/.github/workflows/local-windows-verifier.yml`,{method:'PUT',body:{branch,message:'Build integrity verifier for local Luxmc modpack fixes',content:Buffer.from(workflow).toString('base64')}});
const receipt={branch,mainSha256:sha256,sourceCommit:base.object.sha,workflowCommit:result.commit.sha};
writeFileSync('docs/validation/modpack-followup/remote-verifier.json',JSON.stringify(receipt,null,2));
console.log(JSON.stringify(receipt));
