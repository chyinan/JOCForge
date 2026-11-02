# F01E 36-row cold-owner boundary

**Status:** `APPROVED_COLD_V1_FIXED_28; WIP_DYNAMIC_PROJECTION_PRESENT; NORMAL_CONTROL_AND_ZERO_DRAIN_STILL_38`

## Approved state contract

The approved F01E cold provider has 28 object-owner records. The normal-control snapshot uses fixed dimensions: 38 physical rows, ten bed rows, 28 object rows, and a 37-row regular view. The 36-row graph has ten bed rows and 26 objects. The approved reset rule discards previous event/sampler/binding/snapshot/counter state and reconstructs fresh selected owners for a new programme/attempt; it does not define an in-place resize.

`NormalControlPreparedSnapshot::from_validated` still rejects a profile that is not 38 channels / 28 objects before building its fixed arrays. The fresh-zero callback also still requires 38 physical rows. These are separate gates after XML parsing and input preparation.

## Current worktree note

The current WIP `ValidatedF01EColdProvider::for_admitted_adm_input` projects graph physical/bed/object counts onto a cloned cold provider and creates zeroed object-owner records from the prepared graph bindings. It preserves selected output/profile fields and resets the callback index. This is code under active development, not an approved native F01E schema or a DEE-validated audio rule. It also restricts Line/RF DRC values to `none` or `film_light`; the effective XML default remains unbound.

The WIP method does not yet make the downstream normal-control snapshot or fresh-zero callback dynamic. A 36-row input therefore still cannot be called end-to-end. Keep the cold schema/version, owner generation, input physical rows, meter/control shape, and DDP frame-view shape distinct. The original 28-owner state must not be relabeled as a 26-owner native contract.

## Evidence limits

The source audit and the approved F01E package are static/model evidence. No native 36-row F01E execution or playable 36-row JOC output was observed. No DSP formula was changed in this Wash pass.
