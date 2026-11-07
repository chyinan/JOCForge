# R141 and third-U32 plan compatibility boundary

R141 is terminal at the process level, but its exit code and encoder-completion marker were not captured. Its observed U16 value 0x0010 and width-five field belong to serializer record +0x08, which the separate syntax mapping identifies as BSID. They are not the transport-plan third U32 at per-frame descriptor +0x0B1C.

The corrected R4/run11 helper path is a separate codec-loop metadata serializer call: its value 1 is element_count and its scalar field was not captured. The plan has distinct A/B caller paths and required frame/parent/frontend/plan-storage generation tokens. The R141 serializer callsite is neither A nor B, and R141 lacks those tokens. A shared DLL hash does not join the owners or generations.

A candidate store to +0xB18 was observed on a separate constructor path; no typed base or generation link reaches plan descriptor +0xB1C. Third-U32 first writer, semantic owner, profile transfer, and R138 caller applicability remain UNBOUND/NOT_EVALUATED. No capture or DEE job was started by this slice; it makes no claim about the closed BSI presence chain.
