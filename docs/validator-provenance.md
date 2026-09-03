# OpenJOC validator provenance

JOCForge uses one external validator identity for strict inspection and
rendering. The machine-readable pin is
`validation/openjoc-validator.json`; `scripts/openjoc-validator-pin.ps1`
hard-fails before external validation when either the executable version or
SHA256 differs.

The original mutable artifact had SHA256
`b4fb4593be8dfe3bbeceb4ba4e3eb9c97f89ebe043555e8af615cb4eea289bee` and was
unavailable after the active development target was overwritten. The migrated
frozen artifact is OpenJOC semantic version `0.16.0`, built from source
revision `ad6556babf42566f1a09820b01dc333703c8b1da`, with SHA256
`7e761d0c8a23a064c96c1f70f6415e1d95f18b8b88d72eca0bfa6b52d8ee2daf` on
Windows x86-64. Its immutable manifest and executable live at the path
declared in `validation/openjoc-validator.json`. OpenJOC remains an external
validator/decoder/renderer, not a JOCForge implementation dependency or the
sole evidence authority.
