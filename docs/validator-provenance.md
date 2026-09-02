# OpenJOC validator provenance

JOCForge uses one external validator identity for strict inspection and
rendering. The machine-readable pin is
`validation/openjoc-validator.json`; `scripts/openjoc-validator-pin.ps1`
hard-fails before external validation when either the executable version or
SHA256 differs.

The pinned executable reports OpenJOC semantic version `0.16.0`, was built from
source revision `ad6556babf42566f1a09820b01dc333703c8b1da`, and has SHA256
`b4fb4593be8dfe3bbeceb4ba4e3eb9c97f89ebe043555e8af615cb4eea289bee` on
Windows x86-64. OpenJOC remains an external validator/decoder/renderer, not a
JOCForge implementation dependency or the sole evidence authority.
