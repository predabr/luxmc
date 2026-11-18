# Skin Studio and NameMC

The generated cape catalog was removed from both the main studio and the model editor. Existing appearances remain valid, including previously selected generated capes. The studio offers the current cape, an imported cape, the account cape when available, and a no-cape control.

NameMC opens in a separate launcher-owned native webview window. Navigating to a NameMC skin detail link selects that skin, downloads its original PNG, saves it in the local collection, and shows the preview. Importing does not upload or apply the skin to the account. A pasted skin link and downloaded-file import are available as alternatives.

The picker window has no launcher capability entry. Its command wrappers additionally reject callers whose window label is not `main`. Navigation is restricted to HTTPS NameMC domains; selected skin IDs must contain exactly 16 hexadecimal characters. The HTTP client identifies itself honestly as `Luxmc/<package version>`. Downloads use the fixed `s.namemc.com/i/<id>.png` endpoint, reject redirects, limit the response to 3 MiB, time out after 12 seconds, and validate PNG format and Minecraft skin dimensions before decoding.

The official NameMC catalog and skin page were inspected:

- [NameMC catalog](https://namemc.com/minecraft-skins)
- [Official skin detail example](https://namemc.com/skin/63455d7069b397c2)
- [Original skin PNG](https://s.namemc.com/i/63455d7069b397c2.png)

The original PNG endpoint returned HTTP 200. A later native-executable check found that requests without a User-Agent were rejected with HTTP 403. Adding the honest Luxmc identifier fixed this: an explicit ignored live Rust test now imports the public skin using the same production function and validates the decoded PNG. Windows PowerShell 5.1 and 7 checks independently confirmed HTTP 403 without an identifier and HTTP 200 with the Luxmc identifier. No shell fallback was needed. A direct automated catalog request returned HTTP 403, so the chooser explains that NameMC may require its own verification and provides paste/file fallbacks. No CAPTCHA bypass or unofficial catalog scraping was added. Browser regression tests use a simulated native selection event; they do not prove that NameMC will admit every desktop webview or network.

Validation:

- `pnpm check`: 0 errors, 0 warnings.
- `cargo check`: passed; four existing warnings outside the NameMC module.
- NameMC Rust unit tests: two passed; the network-dependent test is ignored by default.
- Explicit live `imports_public_skin_with_identified_http_client`: passed using the production importer, including base64 and PNG validation. Logs: `namemc-http-live-test.log`, `namemc-http-unit-tests.log`, `namemc-http-cargo-check.log`.
- `tests/namemc-picker.browser.cjs`: native selection and pasted link, original PNG persistence, saved-skin preservation, no implicit account update, deduplication, event cleanup, retry after failure, generated-cape removal.
- `tests/appearance-microsoft.browser.cjs`: imported cape persistence, combined Microsoft action, local save before upload, retry after Microsoft failure, restore after reload.

Visual result: [skins-studio-design.png](./skins-studio-design.png).
