# Selected configuration record import

## Scope

This slice imports the selected configuration fields into a caller-owned encoder owner and metadata buffer. It covers the active source-record-to-owner path only; it does not define the external job writer, every possible configuration value, or whole F09/ADM lifecycle.

## Buffers and field mapping

- Source record: 2,752 bytes. U16 at byte 0 is the record size. Signed I16 at byte 2 is a layout/version tag, not a byte count.
- Owner buffer: 4,096 bytes, independently initialized by the caller.
- Metadata buffer: 64 bytes, independently initialized by the caller.
- Fields copied into the owner include `mode_code` (source byte 12 → owner byte 4, DWORD), `format` (52 → 44, DWORD), `lfe_present` (56 → 48, DWORD), and `rate` (24 → 16, WORD). The complete ordered basic and advanced copies are enumerated in `CONTRACT.json`.
- The selected profile is mode 0, format 7, LFE present, rate 1024. Other tested combinations are input-domain examples, not a claim of universal native enum coverage.

The import first checks the signed tag. Tags below 512 return 140 before owner or metadata writes. For admitted records it performs the listed byte copies in order, applies the documented normalization and derived selector/metadata rules, and then returns the corresponding result. The rate tables and floating-point scale values are exact in `PARAMETERS.json`.

## Partial failure behavior

The observed compatibility-class rate failure returns 59 after copying mode/format/LFE/rate and writing earlier metadata fields. The target digest changes, so this path is partial mutation rather than abort/no-commit. Prefix tag failures return 140 with owner and metadata unchanged. The contract also specifies the later channel-map validation formula and return 161, but the seven controlled vectors do not include a rejecting 161 case; do not claim that rejection as runtime-tested.

## Evidence and validation

Seven synthetic input-only recipes reconstruct their source bytes from explicit byte patches. `EXPECTED.json` contains full post-import owner and metadata bytes, return values, before/after hashes, and canaries. Wash independently rebuilt every source record and replayed the ordered import/derived fields, matching all 4,096 owner bytes and 64 metadata bytes for all seven cases. The six naturally observed owner imports are recorded separately from the seven controlled cases; those counts are not interchangeable.

The implementation contract does not claim untested generic metadata validation, every version/tag, the external job-to-config writer, or the whole F09/ADM-to-JOC pipeline. Whole implementation readiness and product acceptance remain `NO` / `NOT_RUN`.
