# DI UI input alias correction and cancelled-attempt status

## Corrected path interpretation

The provided form screenshot shows `D:\潜在的なアイ.wav` selected. The provided log screenshot shows the CLI input alias `D:\ADM.wav` and a temporary output path. The user states that the GUI stages a selected ADM BWF under the ASCII alias `D:\ADM.wav` to avoid Chinese path handling and restores the original filename after processing. These observations are consistent with that explanation, so the earlier conclusion that `D:\ADM.wav` proved a stale or unauthorized source is retracted.

However, there is no per-attempt source-file hash, copy/rename receipt, or other artifact binding the alias bytes to the selected ADM. The alias-to-source relation is therefore `USER_ASSERTED/UNBOUND`; this package claims neither that the wrong file was used nor that the correct input identity is cryptographically bound.

## Attempt outcomes

The sealed UI-observation receipt records two attempts for the selected Potential AI input: attempt 1 last observed at 64.6%, attempt 2 at 74.8%. Both later showed `编码已取消`. Neither has a final EC-3 completion/rename receipt; final file existence and stream properties were not checked. They are cancelled attempts, not successful samples. The second authorized sample, Human Cat, remains `NOT_RUN`.

No new UI action or DEE run was performed for this correction. The old `sample6-keyless-full-20261101-08.ec3` structure-error artifact was not used.

## DI static finding remains unchanged

The prior hash-bound static edge still shows `FUN_18014f640` preserving the same `SimpleType<bool>` receiver through its `+0x28` indirect call to `FUN_18016a540`. The generic map caller, typed owner/generation, and first measurement/audio consumer remain `UNBOUND` or `NOT_IDENTIFIED`. The path correction does not establish a DI audio effect.

## Evidence limits

The screenshots establish the visible selected path and CLI alias; attempt times/progress are transcribed from prior UI observations and are explicitly not a raw-log or source-hash attestation. The Wash package includes screenshot hashes and metadata only, not the screenshot images, audio, or encoded output.
