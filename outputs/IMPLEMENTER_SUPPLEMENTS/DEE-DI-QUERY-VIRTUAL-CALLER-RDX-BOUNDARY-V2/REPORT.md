# Dialogue Intelligence query virtual-caller RDX boundary — V2

## Bound static receiver path

The direct code edge into the ParametersContainer getter dispatcher uses a tail thunk installed at FilterDDPJocEnc vtable slot +0x8. The thunk loads Priv from FilterDDPJocEnc+0xb8 and branches to the dispatcher without modifying RDX. The receiver is therefore statically tied to Priv+0xec0.

## First break

The exact Ghidra reference query found a vtable data reference to the thunk but no code reference to that slot address. Thus the virtual caller that supplies RDX is unresolved; the owner and contents of the dispatcher’s param_2 remain UNBOUND. A prior receipt shows param_2 enters a dynamic record flattener whose records provide the query string, but no selected record name/value is joined to the exact dialogue_intelligence key.

The static contract “if that exact key is supplied, the registered Bool candidate is returned” remains valid. This V2 supersedes V1 only with the corrected/revised source package hash and its exact thunk-reference boundary; no runtime/key/audio behavior is inferred.

The upstream manifest-list SHA-256 is 8a9157bbcc50468a853cff8582c7844329fe1b876140dea98e6abe1ce08d4063 and its package-seal file SHA-256 is a9b155ece5dd14eed9d7e272fd8e8c2599275cc3baac63c84dc95d53cfbfc930. Its 13-file checker passed normally and under python -O.
