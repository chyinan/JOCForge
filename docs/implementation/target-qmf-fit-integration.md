# Target, QMF and fit integration

TARGET-QMF-FIT-INTEGRATION-V1 has manifest SHA-256 `fe4e3fe2bcf1fcfced00e618196e692617edf696e6f3f786bec67549a117cf19`. Its CONTRACT.md and EDGES.json payloads match the distributed manifest. The upstream source/provider audit and normal/optimized checker results are coordinator evidence; the local implementation did not read research material.

The production receiver now retains preconditioned source rows 1..15 as the actual target-QMF input. It keeps these 15x1536 F32 audio rows distinct from the 16x6 target-gain update values. `stage_mode0_fitting_from_receiver` consumes one already executed receiver proposal, its same-parent before state and the fitting-QMF before state. It validates typed callback/source/dispatch lineage and uses the receiver's actual s09 values, target rows, analysis after state and descriptor result. It does not execute the receiver or descriptor stages again.

`SelectedJocQmfProjection` owns the same-generation s09 return, final-six, target windows and carrier windows. The target storage uses the 15x48x64 object view. Its packed accessor implements the approved 5x48x192 mapping `object = group*3 + packed_bin/64`, `bin = packed_bin%64` without copying, changing complex component order or permuting R/C. Carrier storage remains 5x48x64 in L,R,C,Ls,Rs order.

The retained receiver and fitting after states stay owner-private beside the immutable downstream projection. This boundary contains no 180-cell coefficient fitting, eligibility computation, temporal detector, score kernel, representation retry or writer execution. Those stages must consume this projection with their own approved state and providers. Runtime coefficient-value comparison, later generations, whole-song execution and audio acceptance remain NOT_RUN.

The complete `selected-j05-owner-receipt.v1` still needs the parent component/LFE/target owner tokens and before/after root hashes. The actual buffers and typed g/s/d clocks now exist; the remaining work is parent receipt assembly and F05JL consumption, not a missing selected shape or conversion rule.
