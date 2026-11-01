# ADM/BWF setup record boundary

## Closed static edges

The current converter statically registers separate input routes: ADM/BWF uses selector 3 and has open/factory plus setup callbacks; DAMF uses selector 2 with a separate callback pair.

The setup callback forwards a caller-provided table to the master setup. That consumer reads a U16 record count and six-byte per-channel records, copies them in order, uses the first U16 as TrackIndex, the signed byte at offset 2 for an object-like/bed-like branch, and the final U16 in bed grouping. Some destinations can later be rewritten.

## Remaining input identity

The setup record producer has not been joined to the CHNA TrackUID/TrackIndex map. The R138 audio.adm task's selected callback, parent generation and record-table owner have not been captured together. This package does not establish a physical-row permutation or change the approved TrackIndex-minus-one input rule.

The minimum next observation is to use the already authorized R3 job and capture the selected setup callback/caller, record-table pointer token and count, then trace that exact caller to the CHNA map. No PCM rows need to be read.

## Provenance

Source package manifest SHA-256: cf9e93fcf64b1a3cc89e3a51ebd08b0c0de3bab50a282218ad0ff3bb3d374848
Parent CHNA and DDP locator package manifests are recorded in the source evidence. Normal and optimized source validators passed. This Wash package contains no source code, disassembly, binary, PCM, private key or absolute path.
