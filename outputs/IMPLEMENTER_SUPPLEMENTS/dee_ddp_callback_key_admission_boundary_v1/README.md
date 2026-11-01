# DEE DDP callback admission for pcm_to_ddp

Ghidra static evidence binds the host's plugin loader, callback storage, and later StageRunner enumeration to the same manager object. The host obtains callback arrays from loaded modules and appends each pointer to its audio_filter registry.

The hash-identified DDP Atmos DLL exposes three local callback names: encode_to_atmos_ddp, atmos_mezz_to_atmos_ddp, and pcm_to_atmos_ddp. The host stage-name gate compares exact strings against pcm_to_ddp and three legacy _old names. None of the current DDP local names equals pcm_to_ddp; local index 2 is pcm_to_atmos_ddp and is not the selected key.

This proves no direct match between that DDP DLL's current factory names and the exact host key. It does not identify which other loaded module, if any, supplies pcm_to_ddp. The registry's absolute indices depend on module load order. No runtime job, PCM, payload, or private key was used.
