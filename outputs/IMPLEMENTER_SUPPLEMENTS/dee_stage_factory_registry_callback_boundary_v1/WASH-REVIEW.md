# DEE stage-factory registry callback boundary

This is an addendum to the earlier washed generic stage-factory boundary. The corrected v3 source package has nine current Ghidra evidence-file hashes; Wash independently recomputed all nine and each matches.

The host statically enumerates stage registry entries, calls an indirect callback from each selected registry entry, and appends returned config objects to the StageRunner list. Build then calls the selected config object's factory virtual slot, retains both the config and returned generic ProcessingStage object, and uses capability methods for source/target matching.

A separate registration maps the string pcm_to_ddp to the filter:audio category. That mapping is not the StageRunner registry entry's callback pointer or the config object's dynamic type. The specific callback, config vtable override, concrete stage module, and any DDP descriptor construction remain UNBOUND/NO. No DEE job or runtime capture ran.

The next minimum edge is the callback/function pointer stored in the registry entry corresponding to the selected pcm_to_ddp config object, followed by its config vtable factory override. Do not infer the concrete stage from its name or capability strings.
