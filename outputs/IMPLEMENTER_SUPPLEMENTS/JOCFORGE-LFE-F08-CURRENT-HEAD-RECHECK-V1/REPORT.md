# Current HEAD recheck for the F08 LFE coefficient candidate

## Wash disposition

The R4 coefficient crosswalk originally referenced implementation HEAD `1b4a3417...`; its later current-head addendum referenced `613112b7...`. Wash independently rechecked the implementation repository at the newer HEAD `2155ea31d6c78b5f91b7ba256ce0a506581d1618`.

## Verified current-source identity

- `analysis/owner.rs` hash remains `46a184...2182ab`; the owner still calls `f08_lfe_transform_block` after the LFE window fold.
- `transform/lfe_transform.rs` hash remains `37b20d...4f3052`; the production recurrence still reads the `lfe_recurrence`, cosine, and sine provider arrays.
- The committed parameter JSON at current HEAD has hash `38e9b8...b386f4`; the current working-tree parameter JSON has hash `963a17...e7da5c`. The files differ overall, but Wash compared all three LFE arrays from each and found their F32 words identical to each other and to the Rust arrays in the R4 DEE crosswalk.

Thus the crosswalk's recurrence/cosine/sine word comparison still applies to the selected F08 production callsite at current HEAD. This confirms the parameter delta candidate is not limited to the older source snapshot.

## Scope limits

This is a source/hash recheck only. It does not bind ADM physical row 3 to the native DEE core LFE lane, select the native stride branch, join the native and Rust folded input buffers, or run numeric/audio parity. Those remain open.
