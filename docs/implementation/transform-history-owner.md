# Transform history owner and bounded workspace initializer

This direct slice models the selected six-source history bank as a typed Rust value: source positions `0..=4` are full-band boundaries and position `5` is the LFE boundary; each owns separate primary and companion 128-word F32-bit histories. A returned history can be stored into exactly one `(source, branch)` bank. The Rust owner does not reproduce native object ABI or pointer layout.

The six input-only recipe sequences replay two calls per branch recipe from zero and compare all 1,536 expected history words. The owner-container isolation test stores independent primary/companion snapshots per source slot and verifies that changing one bank does not change another. Its synthetic slot assignment is not a speaker-lane or native per-recipe mapping.

The bounded workspace initializer returns a typed zero workspace for the selected extents: 30 POWER rows of 253 U32, five bias rows of six I16, 30 scratch-exponent rows of 253 I16, six LFE scratch rows of seven I16, one shared work row of 253 I16, and unbound indicators for 30 full-band plus six LFE current rows. Descriptor/arena tail bytes are carried as caller-supplied canary evidence and returned unchanged. The initializer does not expose raw pointers, native offsets, or an ABI-compatible descriptor.

Package manifest SHA-256: `d21eef82d1b135c0219c7e6cb9b40da8052016f907e27813eb2de823af275327`.

Independent-check SHA-256: `19d81e5342e46910d83b358e60de513c3a7de084417ec8434c95b6da6833544b`.

The controlled cold-zero owner value and 420 observed carry checks do not close owner reset after fatal/EOF, history retirement, absolute ADM sample timing, current-row binding, or full product initialization. Those remain open.
