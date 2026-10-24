# Selected configuration record import

This direct slice reconstructs seven sanitized input-only configuration records, then performs the washed byte-copy and derived-field import into caller-owned 4,096-byte owner and 64-byte metadata buffers. Source records are fixed at 2,752 bytes. It does not model a native ABI, external job writer, arbitrary configuration domain, full metadata importer, or whole F09 lifecycle.

The signed I16 layout/version tag at source offset 2 is checked first. Values below 512 return 140 before writes. Admitted records follow the exact ordered basic/advanced copy lists and selected parameter tables. The compatible-class/rate path may return 59 after earlier owner and metadata writes; those mutations are preserved and not rolled back. Prefix-failure and partial-rate-failure results are compared byte-for-byte against the package's expected owner/metadata snapshots, including before/after hashes and tail canaries.

The channel-map formula returning 161 is implemented from the static contract, but no controlled rejecting vector was supplied. A narrow unit test for the table/formula is explicitly static-derived only; runtime/native rejection validation remains unclaimed.

Package manifest SHA-256: `8ad92e6a0372414c4f2319f996399a5167379922ff3ebaa3d29467f339bd61b1`.

Independent-check SHA-256: `ffed91e451eb8eb58a41a46d2412e32a64551f80d149dd2b526d5fc225576ff8`.

The source Wash manifest/check identities and all eight package payloads are recorded and rechecked by the integration tests. Whole F09, external configuration production, current-row binding, ADM scheduling, reset/EOF, and product acceptance remain open or `NOT_RUN`.
