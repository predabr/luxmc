# GLib compatibility patch

Source: glib 0.18.5 from crates.io. MIT license and upstream attribution are preserved.

This copy backports gtk-rs/gtk-rs-core PR #1343 (commit b5a4071), fixing RUSTSEC-2024-0429 without changing the GTK3 ABI required by Tauri on Linux.

The security change is in src/variant_iter.rs: the C out-pointer is mutable and passed as &mut p. Additional cargo fix changes only clarify elided lifetimes and redundant parentheses for current Rust diagnostics. The package version remains truthful. Cargo audit does not certify path dependencies against published registry metadata. The patch is verified against upstream and covered by tests/glib_variant_regression.rs; see docs/evidencias/glib-backport.diff. Do not suppress the advisory or alter the version to hide it.

Upstream: https://github.com/gtk-rs/gtk-rs-core/pull/1343
Advisory: https://rustsec.org/advisories/RUSTSEC-2024-0429.html

Remove this patch when the launcher runtime supports a fixed upstream GLib version.
