# Huffman table provenance and release status

The user explicitly authorized public-repository use of the six Huffman tables and accepts responsibility for publication. The agent-added `INTERNAL_ONLY` handling label is lifted. This user authorization is not represented as ETSI written permission; third-party license clearance remains `UNKNOWN`.

## Verified identity

The washed package identifies the codebook set as ETSI TS 103 420 V1.2.1 (2018-10), clauses 6.6.3 and Annex A.1. Its six codebooks are materialized byte-for-byte at `crates/jocforge-domain/src/joc/huffman/codebooks.json`; the source identity and authorized scope are recorded in [huffman-table-provenance.json](huffman-table-provenance.json).

I independently checked the washed metadata package:

- Manifest SHA-256: `9fc3a5459885a1f1126e223e238c832c69b9d4df84f5d6179c328ce7a2a4b5a1`.
- Metadata payload SHA-256: `cbad0fa2e8fb26f7ec11bf348c641a0689cf5629a9059cd981721e010515e20d`.
- Independent-check SHA-256: `528308005e3d69d8768fe8c1df8f88b44116abf290b27bddd8b6e5c72c7c63d7`.
- The payload hashes match the manifest, and the independent check binds to the manifest.

The official ETSI PDF says reproduction or use of its contents requires written ETSI permission. ETSI's IPR page says ETSI owns copyright in its standards and that reproduction of parts may require a copyright-licence request. The verified companion archive has no independent redistribution licence. Public availability of that archive is not treated as a redistribution grant. This remains a third-party rights status, not a restriction imposed by the user.

## Release boundary

The repository now contains the six user-authorized codebooks and the washed table/consumer vectors. The vectors do not include native runtime data beyond the package's recorded assertions. The third-party permission status remains `UNKNOWN`; no ETSI written permission is claimed. The user authorization allows this repository use but does not change that provenance fact.

The reported 588-of-588 result is a model post-compare, not native runtime verification. OpenJOC archive import replay remains `NOT_RUN`; F05QH table-scope readiness and whole-product readiness are tracked separately, with overall implementation readiness remaining `NO`.
