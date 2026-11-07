# EmpireLoudness Bool to Float32 result boundary

The same EmpireLoudness owner carries the setup Bool from +0x94 into owner +0x2c. A typed read packs that Bool into engine config +0x44. In one conditional mode, the engine copies the selected flag to a subcontext at +0x164 and attaches that subcontext to the same engine owner.

The result path writes a Float32 value to owner +0x88. Its first processing consumer receives the Bool-configured subcontext but reads only fields +0x08, +0x10, +0x18, and +0x20. It does not read or forward the Bool-derived +0x164 field. The numeric dependency from the Bool-controlled subcontext to the Float32 result remains UNBOUND. Runtime mode and final effect were not captured.
