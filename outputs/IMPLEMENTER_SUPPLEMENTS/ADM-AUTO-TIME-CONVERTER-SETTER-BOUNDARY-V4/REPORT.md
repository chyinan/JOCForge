# ADM auto-time converter setter boundary

## Bound interface facts

The host normalizer stores per-target ADM timing rows in `AtmosMezz+0x130`; vtable `+0x10` (`FUN_1400efd60`) answers `dee_audio_input_target` by serializing those rows into a host property-response string. A distinct `dee_audio_input_source` request reads a separate path vector.

In the separate converter image, the bounded factory/config path references filter keys `dee_audio_filter_source` and `dee_audio_filter_target:0`; the bounded string-reference scan found no `dee_audio_input_target` or `dee_audio_input_source` literal. The converter's separately allocated `PrivContext+0xa8` map is read by `FUN_18007fba0` for `offset` and `first_frame_of_action`.

## Remaining edge

No typed parser/copy/call/constructor edge from the host target-property response to `PrivContext+0xa8` was found in this scope; same owner/generation also remains unbound. The literal scan is a limited boundary, not a global absence proof. The default `file_position` MVP remains unblocked; `auto` time behavior is not established.

R3 is still tracing this one adapter edge. No DEE runtime, audio payload, PCM, key/license, or implementation code was accessed in the source package.
