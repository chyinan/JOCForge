# Composite source-vector child-open metadata boundary

The selected caller builds CompositeMaster items from input descriptors, then copies each item's first two qwords into a contiguous 0x58-byte source-record vector. The second qword is the child-open result metadata view. The vector and its independent record count are then passed into the composite builder.

This binds the immediate source of the vector's metadata qword. The inspected caller does not copy the scalar ADM physical N field into that qword, and it does not join the child-open metadata view to the layout object holding physical N. Whether that metadata view ultimately derives from the same ADM rows, and whether source-vector count equals N, remain UNBOUND.

The result is limited to one selected caller and reused source identities. Runtime selection/count was not captured. No DEE run or PCM, payload, key, or license was accessed.
