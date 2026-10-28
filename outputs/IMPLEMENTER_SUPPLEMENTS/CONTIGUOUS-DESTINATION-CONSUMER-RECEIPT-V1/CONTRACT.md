# Contiguous destination to direct consumer receipt

Native/static evidence binds one post-consumer finalized contiguous destination to a heap vector and then to a direct downstream consumer in the same outer processing invocation. The producer range contains 2048 words / 4096 bytes. A read watch armed after the copy reports the first downstream read as the complete 4096-byte vector construction from that exact contiguous source.

The same call chain exports vector begin/end and, when non-empty, passes the vector begin, 4096-byte length, owner counter zero and an observed owner-selected token to the direct consumer. The consumer data pointer equals the heap-vector begin; mode is 1. This closes the payload-buffer identity edge only.

The semantic mapping from the observed owner-selected token/counter slot to a selected transport access-unit identity remains UNBOUND. Source generation also remains UNBOUND. The minimum next step is one token equality receipt at the owner slot-selection boundary and the first transport enqueue/AU-descriptor consumer. No payload bytes, native code/addresses or key material are included.
