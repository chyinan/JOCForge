# DEE/Rust 5.1 LFE path crosswalk

This Wash supplement connects the authorized ADM's first six speaker rows to a static DEE 5.1 core-input ordering, and compares the pre-core LFE behavior with the current Rust source path.

The authorized ADM metadata labels physical rows 0–5 as L, R, C, LFE, Lss, Rss. The static DEE label/category mapping preserves those identities, and its six-entry core adapter preserves pointer-table order. The traced DEE pre-core mixer uses unity accumulation for internal LFE category 3; its non-unit coefficient branch is restricted to categories 4–7. Later staging also copies sample words unchanged.

Current Rust source explicitly selects external row 3 and routes it through a dedicated seven-bin F08 LFE transform and exponent-input path. This is a useful algorithm-comparison target: no non-unit DEE LFE scale was found before the core encoder call, while the core transforms remain different and are not compared by this package.

The matched full-file PCM evidence shows decoded channel index 3 is the dominant persistent residual (whole-file RMSE 0.00620, correlation 0.818). Static row/layout evidence makes an LFE relation plausible, but no same-invocation pointer/generation capture joins this decoded channel index to a physical ADM row. Do not apply a gain or remap from this package alone.
