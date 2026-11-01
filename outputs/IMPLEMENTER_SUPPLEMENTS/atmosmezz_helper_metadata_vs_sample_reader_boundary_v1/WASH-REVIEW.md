# Wash review: helper metadata validation vs sample reading

In the hash-bound helper, master open returns an `AdmValidationResult` shared object, and the first post-open consumer reads only its one-byte status. That branch has no PCM row-table/buffer argument. A separate master vtable sample-read candidate exists: it uses a 40-byte `MultiChannelBuffer` wrapper with borrowed caller row pointers, no backing allocation and `owns_storage=false`, and returns actual frame count through an out parameter. Static evidence does not connect the validation consumer to that sample-read call.

Therefore ADM BWF metadata validation capability and sample-read capability are separate. The R3 full-file sample-6 receipt identifies the job and output, but does not include helper PID, master/vtable, row table, or reader identity. It remains unknown which sample-read object and row lineage the full DEE job used.

This is a reader/runtime ownership boundary only. No audio algorithm or data mapping is supplied. Do not use FFmpeg/OpenJOC or labels to fill it.
