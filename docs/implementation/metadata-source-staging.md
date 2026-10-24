# Metadata source staging

Status: `LOCALLY_VERIFIED_NONCOMMITTING_AFTER_PROPOSAL__PARENT_INSTALL_NOT_RUN`.

The body barrier follows `F13-PRODUCER-BARRIER-REFERENCES-V1`. The staged-parent boundary follows washed `METADATA-STAGED-PARENT-INSTALL-V1` (manifest SHA-256 `e00e12523b542884ef8931d0756eb6acf440f3ed314c88c33858d064f55ab92c`); both payload hashes were verified before use.

`assemble_metadata_bodies` takes the ID11 bytes produced by the admitted reader receipt, builds ID2 from that same body and the selected job line/RF profiles, and accepts a completed ID14 body only when programme/transaction/g/s/d and writer lineage match the J05 projection. It returns pure values and owns no queue state.

`MetadataOwner::stage_source` consumes that proposal plus a typed ID1 configuration. It uses the constructor-pinned wrapper, stages ID1 timer and all four queues on one owned clone, applies push/pop order 11,1,2,14, and serializes published bodies in order 11,14,2,1. Source0/1 still execute timer and queues but emit no EMDF. The result owns a replacement `MetadataOwner`, specialized receipt, body lineage and optional EMDF. It neither installs the owner nor retires the programme.

The existing `process_source` function retains its local fixture behavior and is not the production parent API. Parent installation remains conditional on all other source proposals, F12/frame/integrity checks and any required durable append. EOF adds no source, queue pop or flush.

For source2+, a local integration adapter now passes the actual `MetadataSourceProposal` EMDF through selected carrier admission/reservation/body writing at an explicitly supplied `CarrierBitCursor`. This validates the F13-container-to-carrier edge only. F12 must still produce the real frame template and cursor; diagnostic offsets are never treated as production constants.
