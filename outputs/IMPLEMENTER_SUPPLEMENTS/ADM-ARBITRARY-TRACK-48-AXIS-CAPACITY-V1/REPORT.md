# Arbitrary-track ADM source-axis capacity

**Status:** the current 38-row limit is a JOCForge internal 48-wide source-axis boundary, not a demonstrated external/native ADM ABI limit. Support beyond 38 requires changing all 48-wide consumers and widening some U8 fields; no N>38 end-to-end run exists.

## Count model

Under the selected 10-bed, one-direct-LFE profile, the existing typed graph projection gives each regular non-LFE physical row a global source ID starting at 11. With physical row count `N`, regular rows are `R=N−1`, total active sources are `S=11+R=N+10`, and the largest global ID is `N+9`.

The current Rust spatial axis has 48 source slots, IDs 0–47. Thus `S≤48` is equivalent to `N≤38`. At N=36, the existing receipt binds regular IDs 11–45 and leaves IDs 46–47 absent; do not synthesize, alias, or zero-pad those missing rows. At N=39, ID48 would be the new physical source; at N=256, the model requires 266 slots and global IDs through 265.

The ADM binder accepts 1–256 physical rows, and the source graph uses U16 for total source count and U64 for codec source IDs. But several downstream consumers still store count or destination in U8: an active-source count of 256 overflows U8 at N=246 (`S=256`), while maximum ID255 ceases to fit at N=247 (`max_id=256`). These are Rust type-width/model limits, not external codec limits.

## Required internal redesign

The 48-wide arrays/checks occur in legacy F01E snapshots, Installed48 maps, geometry, flags, point response, post-window, U04 refinement, Meter activity/rendering, spatial control, ReaderTarget and cold-provider schema. Full arbitrary-N support must propagate one typed row identity/presence map and dynamic `S` through all of them, not merely widen the parser or replace one `[48]` array. P11's 11-point geometry is a separate selected-profile prefix; it is not an ADM row count.

The source package finds no proof that a 38-physical-row value is an external ABI limit. Native 1–128 plugin packet-row bounds and a separate DDP open-path per-group capacity of 11 remain different count domains; neither has a typed same-job join to physical ADM row count. Keep those joins unbound. A 36-row internal implementation and output run is still a useful first milestone; it must not be labeled arbitrary-N completion.

## Evidence and scope

Wash reran the package verifier normally and with `python -O`: four package/seal files, 29 current Rust source hashes and 21 existing text-evidence hashes passed. The source package is tied to HEAD `74682d529a6aaf6ec2f6b50d06aad0fb5ab32f7f`. This is a static code/receipt review only; no source changed, CLI/DEE/test/decode/playback ran, and no PCM/data payload/key/license was read.
