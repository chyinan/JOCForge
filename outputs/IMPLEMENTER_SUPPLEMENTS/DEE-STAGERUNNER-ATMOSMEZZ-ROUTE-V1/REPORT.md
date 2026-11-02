# DEE StageRunner selected AtmosMezz route

Recipient: implementation evidence supplement  
Disposition: the approved job selects an ADM/Atmos stage-type route; the first 36-track count gate remains unbound.

## Bound host stage-type path

Ghidra 12.2 static evidence in `dee.exe` (SHA-256 `65485b6d35fa534b25109ab6f238971aaded73fc99c7f7fb119d1a5d5299234e`) shows StageRunner Build resolving the configured filter factory, checking source/filter and filter/output type compatibility, then running the constructed input and filter stages. The selected input class is `dee::input::AtmosMezz` with type `atmos_mezz`; the configured filter key is `encode_to_atmos_ddp`; the output class advertises `atmos_mezz_to_atmos_ddp` / `encode_to_atmos_ddp` and `audio.ec3.atmos` output types.

This is not the standalone raw-PCM `pcm_to_ddp` route in `dee_ddp_encoder.exe`. R1's raw-PCM max-eight-channel gate cannot be applied to the selected ADM/StageRunner path without a typed edge showing that the AtmosMezz path reaches it.

## Remaining boundary

The exact external filter DLL-to-stage-key selection, the reader's 36/38 count transfer, and the first native count gate inside the selected AtmosMezz/converter path remain `UNBOUND`. Sibling DLL hashes are inventory identities only, not proof that a particular DLL was selected. No runtime job or sample data was read.

This static route finding supports targeting the AtmosMezz input/profile chain for variable ADM track counts. It does not prove 36-track end-to-end support, assign a PCM row to a core lane, or authorize an audio-algorithm change.

## Upstream package

- Package: `DEE-STAGERUNNER-ATMOSMEZZ-ROUTE-V1`
- MANIFEST.json SHA-256: `c4698113b15fa1524e7161c79ce8ffd841e73db6eb268c627fb66bbd2e6be2e2`
- Two payloads independently matched their manifest lengths and hashes.
- Checker disposition: `PASS_STAGE_ROUTE_BOUND_COUNT_GATE_UNBOUND`.
- No DEE run, ADM/PCM payload, or key access occurred.
