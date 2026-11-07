# Atmos frame buffer capacity lifecycle

The selected owner stores its initialization flag at +0x68, a float matrix pointer at +0x80, and a channel pointer table at +0x88. The inspected owner fields do not store capacities for either allocation.

The allocator creates the matrix and pointer table only when +0x68 is zero. Their sizes use the current CompositeFormatCount, but this addendum does not re-derive exact byte extents. It then sets +0x68 to one. The first sample consumer uses the current U16 channel count at metadata +0xA and has no capacity argument or same-owner capacity guard in the inspected body.

The post-process helper refreshes encoder caches at +0x228/+0x230. It does not release or reset the matrix/table or clear the initialization flag in that body. Reuse of the same owner with a larger count was not captured; no overrun was observed. This package identifies a code-path capacity gap, not runtime failure.
