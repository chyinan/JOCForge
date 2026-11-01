# Run145 payload 11 group-presence observation

Wash independently rehashed the source package and all payloads; its checker passes in normal and optimized Python modes. The observation is limited to the first AU Type 11 configuration prefix.

For the hash-identified reference EC-3, AU0 begins at byte offset 0 and contains one 4096-byte frame. The cursor matched payload ID 11, then observed group_id_present=1 at file bit offset 9156 and OpenJOC metadata group_id=0. The cursor stopped at bit 9163, before payload length/body. No raw bitstream, payload body, or PCM was persisted or read past that configuration boundary.

The prior static Ghidra image has a default producer descriptor with group_id_present=0, but the Run145 loaded module is unbound to that image. This is a scoped discrepancy, not a product-defect finding or universal producer rule. No DEE job or playback was run.
