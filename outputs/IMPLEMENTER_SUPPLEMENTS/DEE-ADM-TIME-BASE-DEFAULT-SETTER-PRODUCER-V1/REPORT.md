# DEE time_base static default setter producer

## Bound static constructor edge

The time_base member is the StringType at Priv+0x550; its vtable slot 5 targets the StringType setter. The only direct code caller observed for that setter is the member-construction helper used for this property. The property constructor supplies the static default string file_position, copies it to a local string, and calls the setter.

This binds the constructor default producer only. It does not bind a selected job's XML token to the setter argument. The vtable slot reference itself is not a runtime setter call, and the host's generic SetProperty target remains unresolved.

## Relation to sample-range behavior

The downstream setRange/time_base comparison and sample-window contract are inherited from the Wash time-base range supplement; they were not retraced. The selected XML value, runtime Priv instance/generation, setter invocation, and branch remain UNBOUND or NOT_CAPTURED. No DEE job ran and no media/payload/key/license data was read.

The upstream manifest checksum-list hash is 6c8c4d072a0157efdc4ccd8491da38ca095d4e86d9e5b961d8fffb5bc964f4ba. Its checker passed normally and under python -O. No separate package seal is included.
