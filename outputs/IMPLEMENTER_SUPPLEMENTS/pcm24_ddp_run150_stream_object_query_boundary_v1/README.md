# Run150 host stream-object query and DDP receiver boundary

Run150 observed, on one thread, the host RunInput interface, its stream_object query, the DDP source receiver, and the descriptor offset-0 store. The host query returned false and left its output pointer null, so no stream-object token was available. The host RunInput.param2 token differed from the DDP receiver token. The DDP receiver was typed as an internal atmos_storage_composite_master, and its row-table token equaled descriptor[0].

This does not bind a selected ADM master or input generation to the DDP internal owner. No row contents were read. The output was removed after capture; bitstream parsing and playback were not run.

Run149 converter-reader NOT_HIT is limited to the module identity observed by WinDbg (module name, SizeOfImage and PE timestamp); its mapped SHA is UNBOUND. R4's 9972... value was a disk-file hash. The current E installation now hashes to BDB5... with the same file size; it is not established as the Run149 loaded image. Do not extrapolate the breakpoint miss to the current converter DLL.

No source, binary, decompilation, media, PCM, payload, bitstream, license contents or absolute path is included.
