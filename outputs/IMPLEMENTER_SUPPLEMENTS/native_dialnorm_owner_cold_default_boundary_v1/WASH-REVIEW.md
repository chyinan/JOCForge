# DialNorm owner cold default

## Static owner fact

The current hash-bound DDP state constructor and full reset initializer write U16 27 to the DialNorm owner member. The same codec-state pointer is carried through the wrapper/frame-loop path to the setter and serializer. The setter copies that owner U16 into same-owner staging; the serializer consumes the value as the E-AC-3 DialNorm BSI field.

## Remaining value producer

Run-144 observed 28 at the same semantic field, so the value was changed after cold initialization or belonged to another instance. The writer that produced 28 was not captured. Static analysis found many displacement-only +0x34 stores but none typed to this owner. The F06 Float32 chain and the custom_dialnorm Float32 override do not directly write this U16 in the traced paths.

## Next observation

Use the already running R3 job to watch this exact two-byte field from the same owner’s cold initializer through any later write, setter, and DialNorm serializer. Record owner tokens separately from F06 and custom_dialnorm values. Do not infer a conversion from equal offsets or numeric values.
