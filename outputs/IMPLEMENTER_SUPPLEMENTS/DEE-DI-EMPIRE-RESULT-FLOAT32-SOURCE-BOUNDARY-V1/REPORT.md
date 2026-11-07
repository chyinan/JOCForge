# EmpireLoudness Float32 result source

The bounded result path copies a Float32 from a result block into the owner's result field. The direct producer reads a U32/Float32 from the engine subcontext at +0x3C when engine state +0x1E0 is nonzero. In the zero-state branch this function does not write that result-block member, so no value is inferred for that branch.

The earlier setup Bool path conditionally writes a different subcontext member at +0x164. The direct Float32 producer reads +0x3C and does not read or forward +0x164. Their typed/source-generation relationship remains UNBOUND; Bool-to-result causality is not established. This is one direct edge, not a global absence claim. No DEE/runtime/audio/key/license data was accessed.
