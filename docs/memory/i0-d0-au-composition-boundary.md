# I0/D0 AU composition boundary

## Verified

- Isolated package `JOCFORGE-I0-D0-AU-COMPOSITION-BOUNDARY-V1` has manifest SHA-256 `0cd5b5e6a1a37961225bfd71bcbf68619baea76c48800449078919acd3499f43` and seal SHA-256 `584fdd652ce61102f182f34a133fd083e325cfcde2e9674e5cdaf2c36a576e1a`. Its checker passes normally and under `python -O`.
- The architecture target uses separate typed I0 and D0 frame inputs, ordered I0 then D0, with complete-AU publication from one source generation; an I0-only partial result is rejected.
- The target rates 640 kbps I0 and 1024 kbps D0, totaling 1664 kbps for two frames, come from the separately approved flat7x profile contract (`JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1`, manifest `7876e3c53e7bda56f60c8d3a691f9082925c6ef14a7899cff21c09f8b1416714`). They are target values, not legacy writer observations.
- The new package carries only an abstract legacy composition shape. It does not prove the legacy implementation durably published I0+D0 atomically.

## Pitfalls to avoid

- Do not merge the abstract target with `DEE-ATMOS71-I0-D0-AU-TRANSACTION-BOUNDARY-PARTIAL-V1`. That earlier package does not identify its observed writer paths as I0 or D0 and does not prove AU order, source generation, or atomic commit.
- Do not infer D0 source-row or speaker-role mapping from the chanmap bit count, ADM row count, or JOC carrier count.
- Do not treat target atomic publication as runtime evidence or as permission to enable production.

## Open gates and handoff

- Runtime source-generation join: `NOT_CAPTURED`.
- D0 source-row-to-semantic-role mapping and native D0 audio algorithm: `UNBOUND`.
- Real output and W04 acceptance: `NOT_RUN`; `production_allowed=false`.
- Next specification work must independently bind D0 source roles and its processing contract. This note authorizes no codec, audio, or publication implementation.
- The maintained chain handoff is [adm-to-eac3joc-chain-status.md](../architecture/adm-to-eac3joc-chain-status.md).
