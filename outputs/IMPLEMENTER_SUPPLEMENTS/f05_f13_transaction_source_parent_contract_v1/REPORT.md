# F05/F13 transaction contract: static snapshot only

The failed Human Is Cat run note records Meter prepass completion, commits at callback g=0 and g=1, then F05/F13 fails with Joc(ProducerTransactionMismatch), exit 2. No output or partial was reported. The failing run ID, executable hash, and expected/actual producer transaction values were not captured; they remain UNBOUND.

The current source snapshot was independently rehashed for all nine relevant Rust files listed in SOURCE-IDENTITIES.json. Its static clock projection for the first source callback is g=2, s=0, d=1. The parent callback token is jocforge-main-callback-g2; the producer token expected by JOC is jocforge-source-0. Current source passes the source token separately from the parent callback token, and producer/writer generations are both 0 while the JOC owner proposes generation 0 to 1. Therefore the inspected source route is internally consistent; it does not explain the older failed executable without a build/runtime join.

The error is emitted at programme_state.rs:275 when ID14 provenance producer_transaction_id differs from the supplied producer_transaction_id. The enum does not report the actual values. The approved GTC contract says the parent callback token and nested source token are distinct scopes. The comparison distinguishes three cases: a callback token used in place of the source token is a label mismatch; a stale source/writer generation is a cross-generation failure; absent provenance is a missing producer token. These cases require different handling and must not be papered over by weakening equality.

All values in INPUT-ONLY-VECTORS.json are synthetic controls, not observed runtime IDs. No implementation change is recommended from this package alone. The failing executable identity and runtime actual value are still required before changing code.

No encoder, ADM, PCM, media, payload, bitstream, native image or private key/license material was used.
