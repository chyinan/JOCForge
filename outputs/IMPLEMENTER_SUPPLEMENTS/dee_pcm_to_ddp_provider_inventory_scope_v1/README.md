# DEE pcm_to_ddp provider: bounded six-DLL inventory

This package washes a hash-bound Ghidra 12.2 inventory of six top-level audio_filter DLLs present at seal time. Within that bounded inventory, the unique exact host key pcm_to_ddp is returned by dee_audio_filter_ddp.dll local factory slot 0. Its callback RVA is 0x1d1540, constructor RVA 0x1deef0, class is dee::proc::FilterDDPEncPlugin, name writer RVA 0x11d7b0, and the vtable +0x10 getter returns the exact name.

The host path is statically bound from LoadLibraryW/GetProcAddress and dlb_phoenix_get_audio_filter_handle through getter-array append to the singleton-backed manager vector and later StageRunner dispatch. The host uses an exact name comparison and also lists three _old aliases. The six module inventory shows no other exact pcm_to_ddp name.

This corrects the scope of the earlier DDP-Atmos-only negative result: its three local aliases did not match, while the standard DDP DLL is the unique provider in this six-image inventory. It does not prove which modules loaded in any particular run, the cumulative host ordinal, the descriptor offset-0 writer, or the converter-source owner/generation bridge. Runtime observation is NOT_RUN.

Only sanitized module identities, local factory names and address tokens are included. No code, decompilation, binary, media, PCM, payload, bitstream, private key/license contents, or absolute paths are included.
