# Programme measurement to F06ND/F12 edge

## Approved contracts

The selected meter contract defines one programme-level loudness/dialnorm value. F06ND consumes the dialnorm scalar for each source generation and emits same-generation RF/line control. The F12 BSI path consumes a same-source BSI and writes a five-bit dialnorm field. The F12 wire field itself carries no programme identity.

The whole-composition contract requires the BSI and ID1 source projections to carry the finalized programme measurement digest. That requirement must be joined to the scalar and source-generation inputs consumed by F06ND and to the same-source BSI consumed by F12.

## Observed integration gap

The reviewed F06ND/F12 interface clauses name scalar values and source generation; they do not name a typed finalized-measurement receipt. The reviewed cross-module provider map binds numeric providers but contains no programme measurement/dialnorm receipt binding. The bounded frozen-model source stores a plain measurement dictionary and digest without programme/source identity or measurement generation. Its source is not listed in the cited freeze manifest and remains independently unreviewed; its event strings do not prove typed F06ND/F12 calls.

No verified typed finalized-programme identity or receipt is currently shown as consumed by both the F06ND source input and F12 BSI input. This edge remains `UNBOUND` and `NOT_READY`. Native DLL execution and independent model review are `NOT_RUN`. Do not invent a digest-to-owner join or treat equal dialnorm values as identity.
