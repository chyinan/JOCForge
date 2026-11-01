# Field-offset correction

The prior supplement id14_row_member_classification_boundary_v1 incorrectly described byte +3 as the normalizer classifier input. The new Ghidra static formula shows the signed classifier input is row +2; row +3 is not read by this normalizer loop. Do not use the superseded +3 claim. Semantic row name and producer edge remain UNBOUND.
