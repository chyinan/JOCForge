# F13C ID14 complexity and carrier boundary

The selected model treats ID14 as the JOC object-representation payload. Its selected body carries a downmix index and object-count-minus-one; the validated 15-object profile derives complexity 16 as object_count+1. The model parses the complete EMDF and computes complexity before the six-block frame encoder. Its separate carrier_block value only chooses the audio block that receives the opaque skip-field bytes; it is not part of ID14 and does not affect complexity.

Ghidra static evidence binds a selected native ID14 descriptor through queue copy and generic byte serialization. In the reviewed selected dispatch, ID11 has the semantic branch; ID14 follows the generic opaque queue path. A global claim that native code never semantically parses ID14 is not made.

The native complexity BSI byte is configuration-count sourced and serialized before outgoing EMDF. Existing controlled count cases emitted 1 and 16 without ID14 storage, but no same-parent/generation join connects ID14 object count to that native field. Native carrier placement is length-planned: under the approved complete-container cap, positive lengths are compatible with block 2, while the typed carrier_block-to-native-plan owner transfer remains UNBOUND.

This wash package retains model facts, selected static edges, and the exact open joins. It contains no reverse-engineering source, decompilation, binary, payload, PCM, or private key. Static and prior controlled evidence does not establish full audio output parity.
