# DEE host registry and processing-stage boundary

Hash-bound Ghidra 12.2 static evidence closes the host-level sequence: the registry supplies stage-config objects; the host retains a selected config, calls its factory slot to obtain a ProcessingStage, then queries source/target capability methods. These calls set capability properties; they do not carry the PCM row pointer or DDP frame descriptor.

The separate pcm_to_ddp to filter:audio category mapping does not identify the registry callback. The callback-table writer, mapping from the pcm_to_ddp name to a registry entry/callback, concrete factory override/module, and DDP descriptor+0 transfer remain UNBOUND/NO. The exact next edge is the store that populates the per-entry callback slot and binds that entry to the selected name.

This is an integration boundary, not an E-AC-3/JOC audio algorithm rule. It includes no source, disassembly, binary, PCM, or media payload. Ghidra static analysis only; DEE runtime and live attachment NOT_RUN.
