# SignPath preparation

The configurations in `artifact-configurations/` are drafts for provider review, not evidence of an active certificate or working production signing policy. The project has not been approved yet.

GitHub Actions uploads each artifact as a ZIP. Both configurations therefore use a `zip-file` root and restrict the product name and version. The caller must supply the exact package version as the `version` parameter. Third-party executables and DLLs are not included in the project's own signing scope.

The final integration needs approved organization, project, artifact configuration and signing policy identifiers, a protected API token, and manual release approval. The official submit-signing-request action can be pinned to v3 commit `f6d04783b4569d051e0c80105fe66e82819d0092`; secrets must never be provided to untrusted pull requests.

Required order: build the main program, establish its final signed bytes, bind and build the recovery helper against the correct payload, sign the helper, build a signed uninstaller and NSIS package from those verified components, sign the installer, validate the complete installation lifecycle and Windows trust, then generate and sign the release checksums. Signing only the outer installer is insufficient.

The current recovery hash and NSIS signing hooks require this ordering to be implemented and tested before a signed release can be published. The draft component configuration is not a substitute for that integration. Existing published binaries and release tags must remain immutable.

See [CODE_SIGNING_POLICY.md](../CODE_SIGNING_POLICY.md) and [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).
