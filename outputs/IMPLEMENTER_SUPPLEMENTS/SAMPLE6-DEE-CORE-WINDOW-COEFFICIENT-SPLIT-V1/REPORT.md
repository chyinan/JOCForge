# Human Cat core transform window-table split

## Confirmed static parameter difference

The hash-bound DEE core's 256-word window table used by the first shared core sample transform differs from the current F08-selected Rust window provider at 12 entries by one ULP each. The first difference is at index 21: native F32 bits `3c0ea5aa`, Rust bits `3c0ea5a9`. The long cosine table, long sine table, and spectrum permutation compare word-for-word equal.

A one-bin input-only vector with F32 `+1.0` at input index 234 and zero elsewhere exercises the differing coefficient at index 21. The derived primary/companion F32 results differ by one ULP (bit patterns in `INPUT-ONLY-VECTOR.json`). This confirms a static arithmetic parameter mismatch in the shared core transform candidate.

## Scope limit

The selected matched layout row, input descriptor slot for the user-confirmed low-frequency ADM row, internal LFE lane, and scratch history words 256..381 are still unbound. Therefore this table split is a real native-vs-Rust coefficient discrepancy, but it is not yet proven to affect the matched LFE path or explain the channel-3 residual. The LFE-only recurrence/filter tables also remain unmapped to Rust `lfe_*` parameters. No gain/remap change or root-cause claim follows from this package.

No PCM, encoded payload, private key, license material, native source code, or disassembly is included. The input-only vector is a static arithmetic derivation, not a DEE or Rust runtime test.
