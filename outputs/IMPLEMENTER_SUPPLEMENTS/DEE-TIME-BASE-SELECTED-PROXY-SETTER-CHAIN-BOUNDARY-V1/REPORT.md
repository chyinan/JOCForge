# DDP time_base selected proxy setter chain

The static host path binds the registered audio-filter callback to the StageRunner factory descriptor and the descriptor's returned processing proxy. The selected key encode_to_atmos_ddp is bound to its DDP factory/class. That proxy's property slot +0x08 forwards to the nested stage setter.

The concrete setter body has no direct +0x550 operand in the checked body. It does call helpers and indirect targets; those callees were not joined to a +0x550 writer. Thus the selected callback → descriptor → proxy → setter chain is statically bound, while XML time_base value/omission transfer to the descriptor and its eventual writer remain UNBOUND. Runtime object identity and selected invocation were not observed.

Ghidra 12.2 read-only/no-analysis only. No DEE job, media, PCM, encoded payload, key, or license was read. No implementation access occurred.
