# DEE ATMOS WAV PRIMARY CALLBACK ENTRY BOUNDARY V1

Wash verified the source package identity and recorded the following bounded finding.

- scope: FUN_1803ec660 entry register saves through its first helper call only.
- registration: Provider ID 3 registers FUN_1803ec660 as primary callback; registration does not supply N identity.
- prototype: Ghidra prototype remains undefined FUN_1803ec660(void); RCX/RDX/R8B/R9 are saved into local registers but no pointee field reads occur in the prefix.
- first_call: FUN_18057a410 at 0x1803ec69a with RCX=0x10 and RDX=&DAT_180a53b10; trace stops there.
- n_join: No Reader/ADM layout or physical N argument is typed in this prefix; runtime callback invocation NOT_CAPTURED.

Open status: `PRIMARY_WAV_CALLBACK_ENTRY_BOUND; ARGUMENT_TYPES_AND_PHYSICAL_N_UNBOUND; RUNTIME_NOT_CAPTURED`. Runtime: `NOT_RUN`.

- Callback body beyond the first helper was not inspected.
- No runtime callback, DEE job, or media/key access.
