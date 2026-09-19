# SQLite-only application dependency resolution

Source: SQLx 0.8.6 from crates.io; upstream licenses retained.

Luxmc uses SQLite exclusively. This copy preserves the source and SQLite API and removes the unused optional sqlx-mysql dependency and feature forwarding from Cargo.toml. This keeps RSA's RUSTSEC-2023-0071 out of the distributable lockfile without changing application queries or suppressing an advisory. The mysql feature is not supported by this application patch.

Do not use this copy to build a MySQL client. Revisit this patch on the next SQLx update; remove it when the upstream dependency graph no longer resolves vulnerable RSA. PostgreSQL and other upstream source modules are untouched, and Luxmc does not enable them.

https://rustsec.org/advisories/RUSTSEC-2023-0071.html
https://github.com/launchbadge/sqlx
