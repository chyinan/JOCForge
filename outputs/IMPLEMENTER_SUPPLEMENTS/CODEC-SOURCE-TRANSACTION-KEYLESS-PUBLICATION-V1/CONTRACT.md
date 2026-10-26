# Codec source transaction and keyless publication

Ready scope: connecting the already specified source proposals, candidate scratch, main clocks and durable output boundary. Receipts/root transactions and publication policy here are clean LOCAL_MODEL contracts, not a reconstructed DEE global transaction.

## Source preparation and trials

Advance shared main F07 analysis once per codec source. Main F06ND/none-DRC and F08 consume that same analysis result; the independent meter prepass has separate cold histories/audio and is never a main-analysis consumer. Freeze the F09/F10/F11 plan, spectra, db/gain, dither, BSI and payload values for this source. A candidate changes BAP/quantizer/groupers/bitwriter scratch, not the transform, timer, queues or persistent plan.

Use the approved main search procedure (test16, then low16/high784, at most ten midpoints). Each trial has fresh BAP, per-block mantissa groupers and bounded writer state. A legal candidate infeasibility discards only that scratch. Preflight/plan/provider failure, unclassified error, or no accepted candidate is fatal and retires the attempt. F11 producer/domain errors are not swallowed as main trial rejection. A successful leaf frame is still only a proposal.

Source0/1 build and validate complete 4096-byte codec scratch frames and CRC16 and commit the source once. They use ABSENT published metadata, skip the defined EMDF/complexity/carrier/published-payload checks, and append no AU. They do not skip analysis, codec or queue advancement. Source1 consumes JOC writer0 with suppressed disposition.

All current metadata bodies must be ready before the four staged queue operations. Preserve their approved push/pop and serialization orders and individual writer origins; candidates do not push/pop them again.

## Main clocks and runtime plan

For a validated admitted source duration N>0, derive Q=ceil(N/1536) with checked integer arithmetic. The main plan has Q normal plus four cached callbacks, Q+2 codec sources/dispatches/writers, Q published AUs and Q*4096 output bytes. This algebra does not broaden the selected input/profile admission rules. The primary N=6522906 values are a fixture example, not a fixed duration for all admitted inputs. Runtime finalize compares the current validated plan; primary-only terminal schema constants remain primary validation material.

Callbacks g<2 are frontend priming with no codec source. Thereafter s=g-2, dispatch=s+1 and current writer=s. Source0/1 suppression is a different phase. For s>=2, AU j=s-2 consumes ordinary ID11/ID1/ID2 writer j and ID14 writer j+1. At end, ordinary pending writers are Q and Q+1; JOC pending is Q+1. Finalize records the drop disposition without an extra pop, source, codec flush or AU.

Main cached callbacks reuse the last padded main input/source-control snapshot and advance main states. They are distinct from the independent meter's three fresh-zero callbacks.

## Durable source acceptance

Validate the accepted scratch frame, identity, exact size and CRC before IO. For AU j require the expected staging offset/length j*4096. Write the full frame through an explicit-progress loop, check length/readback, then flush/fsync the data. Only a successful durable receipt permits the single enclosing root swap and logical AU ledger increment. A suppressed source has its explicit no-append path, not a fake durable append.

Partial writes change physical attempt progress only. Error, zero progress, overrun, readback or durability failure retires the attempt and abandons staging; logical committed state is not advanced by that failed source. Abandoned temp bytes may exist. Optional truncation/deletion is cleanup, not rollback or same-attempt resume authority. A crash after durable append but before the root swap also requires a new programme-origin attempt, with cold meter/main owners and no reused pending queues or trusted old measurement receipt.

## Finalize and failure visibility

Check the validated plan's terminal counts, pending ledger and staging length, fsync staging, atomically publish under the existing no-overwrite final-path policy, fsync the parent directory, then record successful final identity and DONE.

If rename itself did not succeed, use its bounded atomic failure guarantee. If rename succeeded and parent-directory fsync then fails, the final path may already be visible. Report the existing DURABLE_OUTPUT_ERROR with finalize/parent-sync phase, retire the attempt and return no DONE/successful publication receipt. Do not invent rollback of completed AU ledger entries or promise that the previous namespace is unchanged. This clarification grants no overwrite, automatic final-path deletion/re-rename, mandatory quarantine or new marker/recovery protocol.

## Architecture and verification boundary

Domain/application carry typed proposals and outcomes. Actual file operations remain in io, composed by the permitted CLI shell. This contract does not authorize application->io, external encoders as a production core, private keys or a dummy authentication fallback. Keyless checksum/tail and real CRC/frame math remain their approved numerical contracts.

The supplied transactions are abstract synthetic control outcomes. Their feasibility predicate is not a BAP/codec implementation. Normal and optimized checks exercised 24 cases, 20 fatal/restart cases and a 4249-step primary queue clock, with no actual DSP, frame generation, filesystem durability or native execution.
