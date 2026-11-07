# Rate writer child pointer reset boundary

The bounded callback path gets a writer receiver from a rate-handle field. A separate slot-2 reset path reads the same structural handle field and calls a reset helper, but the helper writes only writer-receiver +0x248. It does not read, initialize, or store the child pointer at +0x38.

Thus this reset edge does not provide a constructor/store source for child +0x38. The child pointer's typed parent and generation token remain UNBOUND. The shared handle-field displacement does not prove the slot-2 and slot-4 callbacks used the same runtime object or generation. No codec/BSI identity is inferred; no broader reader search or runtime was performed.
