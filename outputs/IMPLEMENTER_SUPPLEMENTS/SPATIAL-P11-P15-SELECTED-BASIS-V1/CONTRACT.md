# Selected P11/P15 basis provider

This package materializes the selected local mathematical generator from the approved geometry contract and its authorized Y_front parameter. It closes selected P11/P15 generation and cold values for the implementation model; it does not claim a dynamic capture of a native runtime table or a real ADM callback.

Only layout 0x111B and 0x159B with profile_enabled=true are supported by this selected provider. P11 emits lower bed points 0..6, upper-front pair, and upper-rear pair. P15 emits lower bed points 0..6, front-side pair using Y_front, upper-front pair, upper-middle-side pair, and upper-rear pair. Stable source indices, Q15 values, and F32 bit words are in SELECTED-P15-INPUT-RECIPE.json and the approved geometry spec.

Coordinates are signed I32 Q15. Convert numerically to binary32 with R32(I32), then multiply separately by exact 2^-15 and round to binary32 again. Preserve row order; do not sort by coordinates. Y_front is required only for P15 and must satisfy 0<Y_front<16384. The approved parameter asset supplies 5285. Missing or invalid P15 parameter rejects the profile.

Source identities: approved package MANIFEST SHA-256 c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a; specs/01-selected-geometry-layout-and-bed-routing.md SHA-256 8ea5ff070ebfb77ecfe86a7e87ce846e05de2a1b3575bf834b5ead7414a51830; parameters/geometry-front-side-y-v1.json SHA-256 5d2346dd39671aae7f7d3b56396fb1dea05da363a218a0efbd926dbab046c185. Wash decision SPATIAL-PRODUCTION-PARAMETER-COLD-PROVIDERS-V1 manifest SHA-256 369d20fb6242ce8fd6835d6c75484304c97f21b0cbc3c228d47af21802efba6 (selected parameter and cold constructor scope).

P11/P15 native runtime table identity and order, real ADM invocation, and spatial product acceptance remain NOT_RUN. Do not infer point coordinates from the observed 15-slot target capacity or attributes.