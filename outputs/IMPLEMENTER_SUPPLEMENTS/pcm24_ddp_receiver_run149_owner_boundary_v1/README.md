# Run149 DDP receiver: bounded pointer and owner result

Run149 observed three same-thread stops: host RunInput, DDP source call, and descriptor offset-0 store. The host RunInput param2 token differed from the DDP source receiver token; the DDP row-table token equaled descriptor[0]. Ghidra types the DDP receiver at vtable slot +0xc8 as an internally constructed atmos_storage_composite_master.

This rules out direct object identity between the host RunInput interface and the DDP internal receiver in this run. It does not rule out a typed composition/source relationship through other host objects, nor bind the DDP receiver to the selected ADM master or the same generation. No row contents or raw pointer tokens were read into this package.

The converter reader NOT_HIT result is not used to exclude the current converter DLL: R4's declared hash 9972be4f... differs from the currently installed six-DLL inventory hash bdb502fe.... The actual converter image loaded in Run149 is UNBOUND. The selected ADM reader identity therefore remains open.

The encoder-done marker and 32,964,608-byte output size are context only. No bitstream parse or playback occurred; the output and temporary files were removed. The run used the existing DEE configuration; no license content or path is included.
