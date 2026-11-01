# F06 measurement to U16 serializer field boundary

## Static facts

The F06 integrated-loudness result is a Float32 selected from result records and logged under the integrated/measured loudness names. The same Float32 is copied to a cached owner member and then to a Float32 field in an encode-parameter record. A conditional later Float32 overwrite of that parameter field exists.

A separate DDP path reads a U16 owner field, copies it into serializer staging, and a five-bit writer consumes it. The previously washed run144 observed value 28 through that same-owner setter/staging/serializer chain.

## Open connection

No typed path was found from the F06 Float32 result or parameter record to the DDP U16 owner field. The relationship between these two chains, their generations, units, conversion, clamp and rounding is unbound. An equal numeric value or equal offset in different objects does not join them.

## Minimum next observation

Add captures for the Float32 result/cache/parameter copy and optional overwrite to the already authorized R3 scalar run. In that same run capture the DDP U16 field, setter and serializer consumption. Join them only if the observation provides an explicit owner/record/generation path. Do not start a second DEE job.

This Wash package contains no raw disassembly, source code, binary, PCM, private key, or absolute path.
