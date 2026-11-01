# Payload 11 group-presence branch candidates

Ghidra 12.2 static evidence for the exact target image shows the typed Type-11 descriptor producer zero-seeds group_id_present and has no branch that sets this typed field to one. A different same-layout OAMDI/EVO helper can write presence=1 and group_id=0 for one caller condition, but that helper does not write the descriptor ID; its association with Type 11 is UNBOUND. A copy path can propagate the field from an unbound source descriptor.

A separate descriptor builder selects presence from a negative-sentinel condition, but its descriptor has leading ID 1, so it is not the Type-11 producer. The Run145 AU0 reference cursor independently observed payload ID 11 with group_id_present=1. No retained run record binds the loaded module or the invoked producer to that output.

This narrows the candidate branches without choosing one as the Run145 producer. It is static/reference evidence, not a universal encoder rule or defect finding. The package contains no source, disassembly, binary, raw payload, PCM, or private key. No DEE job or playback was run.
