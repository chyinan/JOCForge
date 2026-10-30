# Selected-profile F13 producer length bound

## Scope

This is a model-side arithmetic bound from the approved selected producer domains. It is not a native encoder observation and does not prove that the run-100 writer field is the F13 container length.

## Bound from approved producer inputs

The selected producer domains give these conservative payload bounds: ID11 is 69 bytes for the pinned DBMD; an allowed default-trim variation can add at most one byte, so use 70 bytes. ID14 is capped at 372 bytes, selected ID2 is 3 bytes, and the ID1 FIFO limit is conservatively 10 bytes. Applying the selected payload-length encodings, carriage bits, terminator, protection codes and byte padding gives maximum internal F13 body sizes of 477 bytes for `deployed_compat` and 480 bytes for `etsi_emdf`. Adding the complete four-byte container header yields 481 and 484 bytes.

If the run-100 same-parent `body_bytes + 4` writer operand is the complete F13 container length, that relation would cap the operand at 481/484 for these selected model profiles. Under that conditional identity, model producer inputs cannot reach operands 507, 508, 511, or 512.

## Native identity still missing

Run-100 did not bind its counted/body-length value to the F13 EMDF container type or serialized F13 payload. The F13 length bound is therefore conditional and cannot be promoted to a native DEE reachability limit. The next useful native evidence is a same-parent typed join from the observed body-length field to the F13 container length/prefix, or evidence that the field is produced by another stage.

No fixture or native execution was run for this calculation. Do not assign target payload lengths directly, predict 512 overflow behavior, or claim global unreachability. No PCM, bitstream, private key, binary, or decompiled source is included.
