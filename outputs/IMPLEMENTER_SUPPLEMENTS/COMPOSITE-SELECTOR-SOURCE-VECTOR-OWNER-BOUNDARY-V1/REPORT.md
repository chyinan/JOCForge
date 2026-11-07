# Composite per-item source-vector owner boundary

## Bound static consumer

The upstream Ghidra receipt binds the per-item source-vector consumer's param_2 as a begin/end-backed vector of 32-byte source-entry records. It computes the count as (end - begin) >> 5 and passes the same vector pointer to the reader builder for each index. The prior nested-path receipt identifies each record's offset-zero field as a string-like path field that is carried through a per-item row to storage-open.

## First break

The inspected consumer does not allocate or assign the source vector. Its param_2 owner, named job/config field, and source generation remain UNBOUND/NOT_CAPTURED. The actual path bytes and suffix are not captured, so the selected storage handler remains unbound. A separate U32 value 1536 is a per-item frame sample-count argument and is not source-vector identity.

This closes the static vector shape and forwarding edge only. It does not select a WAV/ADM handler or make the candidate K-count rule applicable to J+0x08. No runtime/path value, media, PCM, payload, key, or license was read.
