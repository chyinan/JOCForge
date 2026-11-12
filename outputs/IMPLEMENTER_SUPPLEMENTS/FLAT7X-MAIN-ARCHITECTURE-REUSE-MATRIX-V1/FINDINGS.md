# Main-to-isolated architecture reuse matrix for Flat7x

## Decision

The sealed main-branch reference confirms that I0 and D0 can be represented as separate frame roles and assembled in explicit I0-then-D0 order under an ownership plan. Isolated already has a stronger additive typed target plan and AU receipt/staging seams. This architectural pattern can guide the implementation; the legacy profile values, rate split, audio paths and source code do not transfer.

## Reuse matrix

| Concern | Main reference fact | Isolated seam/status | Decision |
|---|---|---|---|
| I0/D0 order | Separate I0 frame and dependent carrier inputs; in-memory output order I0 then D0 | Flat7xI0D0TargetPlan and compose_flat7x_au_byte_sequence; F15 pair owner/staging | Keep explicit ordered roles and pair validation; do not re-open order as a DEE reverse task |
| Ownership | Legacy OwnershipPlan represents frame roles and multi-dependent ownership | Flat7xAuOutputOwner plus F13/I0/D0 integrity receipts | Reuse the role/receipt separation idea, not code or old row values |
| Bit I/O/integrity | Main contains legacy writer helpers | Isolated FrameBitWriter, D0 CRC2 implementation and ETSI I0 AC-3 BSI/CRC supplements | Reimplement only from approved ETSI/Wash contracts |
| Profile/rate | Legacy Flat-7.X row uses chanmap 0x0200 and legacy 1024+640 choices | Target plan is I0=640 + D0=1024, D0 chanmap 0x1A00, Flat7x/index1 | Keep target dimensions independent; never inherit the legacy constants |
| D0 payload/audio | Main legacy D0 writer and its audio path are separate product logic | D0 BSI/skipfld structure is partly implemented; source-role and D0 audio producer remain open | No old writer, FFmpeg carrier, or audio algorithm reuse |
| Publication | Main pair concatenation returns bytes in memory | Isolated F15 owner and temporary staging are partial; final product exposure remains open | Retain isolated transaction gates; legacy vector concatenation is not durable atomic publication |
| Existing 5.1 | Main comparison is design context only | Existing 5.1 multirate writer/resolver remains its own path | Keep 5.1 code path isolated and regression-covered |

## Current target and remaining audio-specific evidence

Target: one 640-kbps I0 followed by one 1024-kbps D0, aggregate 1664 kbps; D0 chanmap 0x1A00; Flat7x index 1; seven JOC coefficient channels; 15 objects. The official sample's observed rate is a separate 640+640=1280-kbps reference.

The additive I0 structural assembler is caller-span based and production-disabled. Legacy I0 audio-block generation, CRC validation on a real frame, D0 source-role mapping, D0 audio behavior, same-source generation join, all-AU EMDF fit, and real output remain open. Those are the next target-specific work; ordered composition and role ownership are already represented in isolated.

No main source code or constants are included. This supplement authorizes architecture direction only; production_allowed=false.
