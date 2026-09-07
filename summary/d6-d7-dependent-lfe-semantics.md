# D6/D7 and dependent-LFE semantic closure investigation

Verified 2026-09-08 against JOCForge `9ce7706492e52af523c0569f274b4e9ea26a91f0`, OpenJOC `77e842303d4cb9d95b29ff41a3b9ed230f246b80`, ETSI TS 102 366 V1.4.1 and TS 103 420 V1.2.1.

## Public rules

- TS 102 366 V1.4.1 E.1.3.1.2 permits one I0 followed by up to eight associated Type-1 dependents D0..D7, with sequential dependent IDs and matching sample rate/block count.
- E.1.3.1.7–1.8 define dependent `chanmape`, `chanmap`, `lfeon`, and Table E.1.4 locations. The map includes full-band locations, LFE2, and LFE; map cardinality must equal the dependent's `acmod` full-band channels plus `lfeon`.
- E.2.8.2 defines dependent data as replacement when its location is also in I0, and supplementation when it is not. The explicit `acmod=1/0, lfeon=1` example says dependent center and LFE replace the corresponding I0 center and LFE.
- TS 103 420 V1.2.1 Table 47/48 defines idx0/3 as five full-band JOC inputs and idx1/2/4 as seven. Its Table 47 note says LFE bypasses JOC processing. Clause 8.1 sends I0 and all dependents to the JOC decoder; clause 8.2 places OAMD/JOC EMDF in the final dependent.

## Project semantic boundary

The canonical JOCForge plan admits exactly the Table-47 full-band topology plus at most one LFE(0). LFE is a separate programme plane and never a JOC matrix input. Each dependent must contain at least one full-band channel because every legal `acmod` has at least one full-band channel; therefore a dependent-LFE replacement can be attached to an existing full-band replacement but cannot create an LFE-only slot.

The useful no-LFE dependent maxima are five for idx0/idx3 (five single full-band locations) and six for idx1/idx2/idx4 (five singles plus one public two-channel pair). A single dependent LFE supplement or replacement does not increase those maxima because it does not add a programme channel or a new legal independent ownership slot. D6/D7 are therefore `SEMANTIC_CEILING` for the canonical JOC plan, not `SPEC_LEGAL_NOT_IMPLEMENTED`; raw E-AC-3 syntax still permits D0..D7 for other programme plans.

## OpenJOC boundary

OpenJOC already assembles a dependent LFE replacement over an I0 LFE and retains the result outside full-band channel planes. It rejects LFE2/multiple LFE and duplicate dependent full-band locations. No OpenJOC change is required by this closure.

## Implementation boundary

JOCForge should model dependent plans explicitly, validate sequential IDs, map cardinality, full-band ownership, exactly-one LFE ownership, final JOC/EMDF owner, capacity, and timing before emitting bytes. The current parallel map/carrier API can remain as a compatibility wrapper. Evidence must distinguish `SUPPORTED_PASS`, `SEMANTIC_CEILING`, `REJECTED_CAPACITY`, and `PUBLIC_AMBIGUOUS`.
