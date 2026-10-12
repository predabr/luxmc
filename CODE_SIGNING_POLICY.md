# Code signing policy — Luxmc

Status: the maintainer supplied SignPath Foundation's rejection email. The project currently lacks the public adoption and independent references required by that program. No Foundation certificate has been approved or issued for Luxmc. Windows releases remain without Authenticode signing until a trusted provider is configured and all components are verified. The release's Ed25519 signatures do not establish Windows publisher trust.

## Intended signing provider

If accepted, the intended acknowledgement is: "Free code signing provided by SignPath.io, certificate by SignPath Foundation". The Foundation would apply signatures using its own publisher identity; it does not issue a private signing key to the Luxmc maintainer.

## Team and approval

- Author, committer and reviewer: [Pedro / predabr](https://github.com/predabr).
- Release signing approver: [Pedro / predabr](https://github.com/predabr).
- External contributions are reviewed before inclusion in a signed build.
- Maintainers must use multi-factor authentication for GitHub and SignPath. Current enforcement has not been verified by this application preparation.
- Each production signing request requires manual approval. Signing credentials must not be exposed to pull requests or forks.

## Build origin and scope

The source repository is [predabr/luxmc](https://github.com/predabr/luxmc). Builds use GitHub Actions with locked package and Cargo dependencies. The installer must be traceable to the exact commit and successful build run that produced it.

The intended Windows scope includes the main executable, Luxmc recovery helper, uninstaller and final NSIS installer. Bundled third-party binaries and installer plugins must be reviewed with the provider and tested under Smart App Control; they cannot simply be signed as original Luxmc code. The helper's integrity binding must be validated after Authenticode signing, since signing changes executable bytes.

Releases advertised as Authenticode-signed must require successful publisher signature verification, timestamp verification, installer lifecycle checks and signed SHA-256 release manifests. Current unsigned releases must disclose that status and possible Windows policy blocks. A missing or failed production signature must not be silently represented as a signed release. Provider credentials, artifact configurations and production policy approval remain pending.

## Privacy

See the [Luxmc privacy policy](https://luxmc-r92.pages.dev/privacidade). Application contact data is provided by the maintainer to SignPath for account administration and review under the [SignPath privacy policy](https://signpath.io/privacy-policy).

Signing submissions are limited to public project build artifacts and build-origin metadata. They must not contain users' Microsoft or Luxmc tokens, passwords, worlds, private invitations, installed instances, or local account data. No runtime user telemetry is sent to SignPath by the launcher.

## Download verification

The current updater verifies Ed25519-signed release checksums and the installer SHA-256. This verifies distribution integrity and does not establish Authenticode trust in Windows. After provider approval and integration, this page will identify the signed release and the verification evidence.

[Program conditions](https://signpath.org/terms.html) and [application](https://signpath.org/apply.html).
