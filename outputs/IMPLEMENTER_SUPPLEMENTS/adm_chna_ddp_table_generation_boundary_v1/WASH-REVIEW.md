# ADM CHNA to DDP table boundary

## Closed static segments

For this inspected 38-channel ADM BWF, the CHNA records have TrackIndex values 1 through 38 exactly once, and record order matches TrackIndex minus one. The converter metadata loader reads the ADM TrackUID mapping and uses TrackIndex minus one as its zero-based per-track slot. A separate converter setup method copies its incoming per-channel metadata records in order into the master table, and the reader wrapper passes the borrowed row-pointer table without reordering.

Within one DDP worker invocation, its first consumer copies input row i to internal row i. The same invocation also sends a separate DDP-owned six-byte channel-map vector to a storage operation. That vector is not the PCM row-pointer table.

## Remaining join

The converter setup record source is not typed-joined to the CHNA map. The converter reader row table was not captured in the prior DDP locator run, so its identity and generation are not joined to the DDP input row table. The end-to-end ADM physical track to DDP row permutation remains unbound.

## Wash disposition

The selected input contract already defines CHNA TrackIndex minus one as the JOCForge zero-based physical slot. This static observation supports that bounded input rule and does not establish an additional DEE reader-to-DDP permutation. Do not infer track identity from row count, ordinal copy, or pointer adjacency.

The next minimal closure is one same-job pointer-token capture across converter setup/read and DDP row/map processing. It records table identities and row ordinals only; it does not retain PCM.

## Provenance

Source package manifest SHA-256: 7ac71e0be4666c2fe9f50b8432257968b12fe526ed7f7cf1961062bcf293614d
Parent R3 DDP locator manifest SHA-256: a8751ffce6b110d7f05e352198db354db87145d75b7ccb3b372394064de33d38
The source checker passed in normal and optimized Python modes. This Wash package contains no source code, disassembly, binary, PCM, private key, or absolute path.
