# Helper source-time normalization chain (v3 supersedes v1/v2)

## Final source-time rule

The helper's source-info object contains binary64 start_time and absolute FFOA seconds. Their constructor defaults are 0.0 and -1.0. During master open, the getters read both from the same source-info object and write the master start_time and relative FFOA. If absolute FFOA is negative, relative FFOA is the -1.0 missing sentinel; otherwise it is max(0, absolute FFOA - start_time). If FFOA precedes start_time, the helper reports it and uses relative zero. The helper reports offset=start_time and absolute ffoa=start_time+max(relative,0). Host output slots default to zero and retain that value if a corresponding line is absent.

The earlier three-signed32 conversion is only an intermediate source-info field write; the later absolute-FFOA normalization overwrites it.

## Sample-6 observation

The input SHA is the same as the prior native sample-6 receipt. Its AXML start is 3610 seconds, BEXT time reference and explicit AXML offset/FFOA are absent, and a 580-byte DBMD chunk is not interpreted. The native receipt reports offset=ffoa=3610, so the full-file relative start is zero under the formula.

This value match does not prove AXML start wrote source-info start_time or absolute FFOA. Their first writer and helper runtime image identity remain unbound. Do not generalize the sample-specific result to other ADMs.

## Provenance

Source package manifest SHA-256: 4b3d24c9dd0910fdd9a53d16987cffa2ae9e64334e28a9bb1e84f7d5ca35d1f6
Normal and optimized source package validators passed. The supplement contains no source code, binary, disassembly, audio, private key or absolute paths.
