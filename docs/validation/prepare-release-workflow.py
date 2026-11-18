from pathlib import Path

file = Path(__file__).resolve().parents[2] / ".github/workflows/release.yml"
text = file.read_text(encoding="utf-8")
text = text.replace("v2.0.1", "v3.0.0")
text = text.replace("jobs:\n", "concurrency:\n  group: release-${{ github.event.inputs.tag || github.ref }}\n  cancel-in-progress: false\n\njobs:\n", 1)
text = text.replace("    steps:\n      - name: Get tag", "    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.inputs.tag || github.ref }}\n\n      - name: Get tag", 1)
text = text.replace("      - name: Create or Find GitHub Release", "      - name: Validate release version\n        env:\n          RELEASE_TAG: ${{ steps.get-tag.outputs.tag }}\n        run: node scripts/check-release-version.mjs \"$RELEASE_TAG\"\n\n      - name: Create or Find GitHub Release", 1)
start = text.index('                body: [')
end = text.index('                prerelease:', start)
text = text[:start] + '''                body: require("fs").readFileSync(`docs/releases/${tag}.md`, "utf8"),
                draft: true,
''' + text[end:]
text = text.replace("      - uses: actions/checkout@v4\n\n", "      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.inputs.tag || github.ref }}\n\n")
text = text.replace("version: 10\n", "version: 10.30.3\n")
needle = "          args: --target x86_64-pc-windows-msvc --bundles nsis\n"
if needle not in text:
    raise ValueError("Windows build step missing")
text = text.replace(needle, needle + '''
      - name: Publish Windows installer with unchanged filename
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          RELEASE_TAG: ${{ needs.create-release.outputs.tag }}
        shell: pwsh
        run: |
          $installer = Get-ChildItem -LiteralPath 'src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis' -Filter '*-setup.exe' | Select-Object -First 1
          if (!$installer) { throw 'Windows installer was not built.' }
          Copy-Item -LiteralPath $installer.FullName -Destination 'Lux MC Launcher.exe' -Force
          gh release upload $env:RELEASE_TAG 'Lux MC Launcher.exe' --repo $env:GITHUB_REPOSITORY --clobber
          if ($LASTEXITCODE -ne 0) { throw 'Windows installer upload failed.' }
''')
text = text.replace("    steps:\n      - name: Download and verify every published installer", "    steps:\n      - uses: actions/checkout@v4\n        with:\n          ref: ${{ github.event.inputs.tag || github.ref }}\n\n      - uses: actions/setup-node@v4\n        with:\n          node-version: '22'\n\n      - name: Download and verify every published installer", 1)
text = text.replace('            "Luxmc_${version}_x64-setup.exe"', '            "Luxmc_${version}_x64-setup.exe"\n            "Lux MC Launcher.exe"')
text = text.replace('          sha256sum "${expected[@]}" > SHA256SUMS\n          gh release upload "$RELEASE_TAG" SHA256SUMS --repo "$REPOSITORY" --clobber', '''          gh api "repos/$REPOSITORY/releases/tags/$RELEASE_TAG" > release.json
          node ../scripts/build-release-manifest.mjs release.json .
          sha256sum --check SHA256SUMS
          gh release upload "$RELEASE_TAG" SHA256SUMS latest.json --repo "$REPOSITORY" --clobber
          gh release edit "$RELEASE_TAG" --repo "$REPOSITORY" --draft=false --latest''')
file.write_text(text, encoding="utf-8")
print("Release workflow will publish only after all installers and the updater manifest pass verification.")
