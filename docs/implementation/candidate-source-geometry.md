# Candidate source geometry values

`spatial::virtual_prefix_geometry::build_candidate_virtual_prefix` generates the approved eleven-point Q15 candidate recipe and converts each coordinate once to F32 using the selected denominator 32,768. It emits the 11×3 XYZ words, five attribute components `(0,0,0,3,2)`, packed attribute `0xB00`, and the 13 defined metadata words for each row. It does not model the native table owner's pointer alias, call routing consumers, or install full source state.

`spatial::bed_source_points::selected_bed_source_points` maps selected bed IDs 0,1,2,4,5,6,7,8,9 to their normalized rectangular F32 points. Bed ID3 is the direct LFE and has no regular bed point. Each row carries the same defined attribute fields and packed word.

Hash-bound candidate tests compare all 33 constructor XYZ words, 143 metadata words, eight downstream value rows, and 72 bed-literal downstream inputs. These comparisons verify candidate formulas against washed inputs/outputs; the downstream vectors are not independent source-coordinate producers. Both generators remain pure candidate values with no shared installed48/control38 owner. They are not bound to F01E's `input-adm.wav` until a source/range receipt resolves the 6,522,906 versus 13,888,071 sample-count difference.

The complete 48-source install remains open: source field0 is not reconstructed, the skip-at-field4 transition is incomplete, earlier measurement-side consumers are not joined to this constructor, and no shared owner installs the 11 prefix and 37 regular bed/object positions atomically.
