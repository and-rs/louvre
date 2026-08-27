# Upstream

This crate vendors the merge-only source from [`tw_merge` 0.1.21](https://crates.io/crates/tw_merge/0.1.21), published by Max Wells as a Tailwind CSS v4 port of Gaucho Labs' tailwind-fuse.

- Crates.io checksum: `19b094572cbc1a5c8d82590da2aea5acfe671924ac221fc205a7276f0755f6fa`
- Source repository: <https://github.com/rust-ui/ui/tree/main/crates/tw_merge>
- License: MIT; see `LICENSE` for required attribution.

The optional `tw_merge_variants` proc-macro integration and its public variants API were intentionally removed. Louvre exposes `merge_classes` as its stable local API.
