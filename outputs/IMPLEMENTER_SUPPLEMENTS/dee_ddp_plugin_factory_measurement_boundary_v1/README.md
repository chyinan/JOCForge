# DDP plugin factory and measurement boundary

Hash-bound Ghidra static evidence closes a DLL-local factory chain: the DDP plugin factory creates a FilterDDPJocEnc stage, whose backend is Priv. A conditional measurement branch can construct AtmosProcMeasure, and its process methods forward a frame descriptor into AtmosProcessor::process. This adds the internal factory-to-measurement path to the earlier host-only summaries.

The live host registry callback for pcm_to_ddp is still not joined to this DLL-local factory table. The measurement branch values and instantiated object were not observed. The descriptor source-row table writer at offset 0 and its owner/generation bridge from the converter row view also remain UNBOUND.

This is a static integration boundary. It does not prove the selected run used this plugin or establish an audio algorithm result. No source, disassembly, binary, PCM, media payload, or private key is included.
