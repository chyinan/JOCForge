# DEE synthetic selector-2 exponent comparison

## Washed EmulatorHelper observation

The source package's hash-bound `FUN_14032d580` Ghidra 12.2 EmulatorHelper
run uses synthetic RAM only. Its static lookup selects `stride=2` for selector
code `2`, with `D=4`, `N=2`, one fullband lane, coupling count zero, and block
gate zero. The supplied raw row is
`[10,11,99,9,99,8,99,6]`. The observed selected exponent sequence is
`[10,11,9,8,6]`; the prepack row is `[10,3,0,1,0,2,2,6]`, comprising the
unchanged initial U4 value `10`, four biased differences `[3,0,1,0]`, and two
tail values `[2,2]`. The observed U7 code values are `[76,12]`.

This is a local synthetic emulation of the hash-bound function, not a DEE
process run, native runtime observation, production producer rule, or
same-owner/generation join. Positive `D`/`N` producers and output parity remain
unbound/not run.

## Read-only comparison to Rust HEAD

At implementation HEAD `798b7667ca39f89aba1ec2cad9e5425d886aa990`, the current
Rust candidate maps selector `2` to stride `2` and iterates current indices
`stride, 2*stride, ...`, subtracting the value `stride` positions earlier.
Calculated from that exact loop (the Rust function was not executed by Wash):

| Input passed to the candidate | Selector | Calculated values | Native vector match |
|---|---:|---|---|
| Raw row `[10,11,99,9,99,8,99,6]` | 2 | `[91,2,2,2,2,2]` | No |
| Compact selected sequence `[10,11,9,8,6]` | 2 | `[1,-1,2,2,2,2]` | No |
| Compact selected sequence `[10,11,9,8,6]` | 1 | `[3,0,1,0,2,2]` | Yes for the delta/tail vector only |

The newly added Rust regression at this HEAD uses `[5,7,6,0]` with selector
`1`; it verifies adjacent-difference arithmetic and tail fill, but does not
exercise R2's selector-2/stride-2 vector. Therefore the current candidate has
not yet demonstrated the R2 stride-2 contract. The evidence supports correcting
or explicitly defining the candidate's input/selection contract before
calling the vector matched; it does not by itself authorize a broader native
producer or runtime claim.

## Scope and exclusions

The source package checker passes normally and under `python -O`. Its
`MANIFEST.json` is SHA-256 bound and lists the source payload hashes; no
`package_seal.json` was present in the delivered source directory, so this
supplement binds the manifest and listed files without claiming a source
package seal. The Wash package contains no executable, binary, raw
disassembly, decompilation, media, PCM, payload, key, or license material.
