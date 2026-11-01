# Host storage lookup and DDP record address-reuse boundary

Ghidra classifies AtmosMezz storage slot +0x40 as a wrapper to storage vtable +0x38; the resolved operation checks a key and looks up a string in an internal map. It is metadata lookup, not an audio-row read.

Run152 and Run153 observed equal salted numeric tokens for a host stack-local storage key argument and the DDP measurement record destination. Static roles and separate call frames show these are different locals; the host AtmosMezz::Run frame was absent from the DDP writer backtrace. Treat this as stack-address reuse, not shared object, owner, or generation identity.

The DDP internal record-to-reader path and descriptor[0]/row-table relation remain bounded within the DDP component. The selected ADM input to DDP owner/generation link remains UNBOUND. Run inputs used preflight hashes without rehashing during these captures; output was removed without bitstream parsing or playback.
