# Wash review: DDP filter callback and private context

The bounded Ghidra chain links a DDP filter instance to its private context: the filter constructor allocates/initializes the context and stores the returned pointer on the filter receiver; the callback reads that same pointer and forwards it to the context initializer. This closes receiver-to-context ownership within that filter instance.

The static factory descriptor exists, but its consumer/dispatch caller is outside the reviewed image or not captured. The additional factory receiver value passed into construction is not proven to be consumed by the context initializer. Neither the filter instance nor its context is joined to a particular full-file job/source generation.

The minimum one-shot runtime observation is at callback entry: typed host job/source generation, filter receiver, context pointer and dispatch callsite. This package provides no codec/audio rule or product state mapping.
