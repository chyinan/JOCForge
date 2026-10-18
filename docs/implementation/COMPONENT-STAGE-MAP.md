# F04 component consumer slice

Status: `MAPPING_COMPLETE`. This is a scoped consumer mapping for P1.3-E2 only.
It does not mark F04A, F04P, F04PG, or F04C globally complete and does not
change the deferred status of their remaining clauses.

## Boundary

The slice consumes an opaque, linear `SpatialAudioInputPacket` supplied by a
future `spatial_control` producer. That packet carries the selected callback's
`installed48` (`F32[48][3072]`), active matrix rows (`n × 48`, `0 <= n <= 15`),
their distinct destinations, programme/attempt/transaction/callback/profile
identity, the validated provider lineage, and the `spatial_control` before
generation/digest. The future producer alone constructs production packets;
this slice validates and consumes them. A module-private test fixture is the
only packet constructor in this slice.

The producer returns an opaque linear `ComponentStageProposal` containing the
complete component after-state (overlap and source ring), the complete shared
parameter after-state, filter-history after-state, callback-local component
output (`F32[15][1536]`), and a typed receipt. It does not replace any committed
root, emit a programme fatal, or commit locally. There is no local commit.
Matrix and `installed48` are
callback inputs and never become `spatial_audio` root fields.

## Clause map

| Functional scope | Clauses materialized here | Explicitly not materialized |
|---|---|---|
| F04A | F04A.6 matrix accumulation; F04A.7 overlap/ring; F04A.8 64-sample filter consumption | F04A.1-.5 raw38/window/activity/prefix/diffusion/installed48 production; F04A.9 LFE/Type10/receiver |
| F04P | F04P.1-.5 shared 15-row parameter-state validation and one-block transition | Other sample rates, row counts, block formats, full controller, real ADM or independent product qualification |
| F04PG | F04PG.1-.5 selected 48 kHz/64-sample constants and its input-only verification runner contract | Regeneration on a platform math library; expected-state seeding; claims about original DEE tables |
| F04C | F04C.1-.2 component consumer boundary/order; F04C.6 component advancement on normal and cached-drain callbacks; F04C.7 component preflight/error boundary | LFE, Type10, point target, receiver/QMF, full composition receipt, render producer/fanout, parent transaction |
| N00 | N00.1-.4 binary32 rounding, non-fusion, signed-zero, subnormal, and first-argument min/max semantics used by the above clauses | Any numeric behavior not required by the mapped F04 clauses |

## Runtime inputs and ownership

| Input/state | Source or owner | Rules for this slice |
|---|---|---|
| `installed48` | Opaque future producer packet; callback-local | Exactly 48 rows × 3072 finite F32 values. Read-only snapshot. Never copied into committed state. |
| Matrix and destinations | Opaque future producer packet; callback-local | `n × 48`; `0 <= n <= 15`; exactly `n` distinct destinations in `0..14`; nonpositive coefficients are skipped. |
| Component overlap/ring | `SpatialAudioState.root.component_state` | 15 × 1536 overlap and 15 × 1472 source ring. Each is read from the same before-state and replaced only in the returned proposal. |
| Shared component parameters | `SpatialAudioState.root.component_parameter_state` | Exact eight-field schema from F04P.1; selected cold state and F04P block transition. One shared state across all 15 rows. |
| Component filter history | `SpatialAudioState.root.component_filter_history` | 15 × 64 raw F32 words; old values feed the current block, source samples become next history only. |
| Kernel | `ValidatedWindowQ15Provider.component_kernel64()` | Existing identity-validated typed provider only; convert each Q15 integer exactly to F32 as specified. Do not consume `receiver_blend32`. |
| Controller constants | `ValidatedComponentParameterProvider` | Existing typed provider only; preserve the validated eight Q15 integers and eight F32 words. Do not read raw JSON/`Value` or regenerate/hardcode words. Its readiness receipt remains `asset_implementation_ready=false`, `dsp_consumption_status=NOT_YET_VALIDATED`; E2 verifies only this scoped consumption. |
| Profile constants | Selected `EncodingProfile` and the F04P.1 fixed sub-profile | 48 kHz, 15 rows, 64 samples/block, 24 blocks/callback, gain scale `2^-8`, clamp limit `1`, hold limit `1`; other profiles reject. |

## Ordered callback transition

All mapped error codes use `LOCAL_TRANSLATION`; none is claimed to be a
package-native or DEE error code. The implementation runs a private preflight over retirement, supported profile,
callback order, owner generations/digests, the rest of packet/provider lineage,
input shape/domain/finiteness, and current state validity before arithmetic.
The wash-approved local error translation and its precedence are recorded in
`IMPLEMENTATION-MAP.json#phase_one/P1.3-E2/lineage_error_translation`; those
labels are local translations of F04C.7 categories, not native error codes.

After preflight, the single callback transition is:

1. Matrix row, source column, and sample order; each positive coefficient uses
   separate `R32(source * coefficient)` then `R32(old + product)`. Zero,
   negative zero, and negative coefficients do not accumulate.
2. `overlap_output = mixer.head + old_overlap`, with one R32 add per sample;
   next overlap is a raw-bit copy of `mixer.tail`.
3. For block index 0 through 23, compute the parameter next-state once from all
   15 × 64 inputs and the old shared parameter state. The filter reads the old
   block's `output_level` and the next state's `previous_gain`/`smoothed_gain`.
4. For each row/sample, compute `a = K[j] * (history * beta)`,
   `b = K[63-j] * (history * alpha)`, then `((a + b) * 256)` with separate
   binary32 rounding and no FMA. Current source does not enter current output;
   it is copied to the next filter history.
5. Clamp by first `min(+limit, intermediate)`, then `max(-limit, result)`,
   preserving the first operand's bits on equality, and multiply by the
   profile scale. No vectorized or reordered reduction is allowed.
6. Type10 component output is `old_ring || filtered[0..64]`; next ring is a
   raw-bit copy of `filtered[64..1536]`. The 24 blocks advance even for zero
   rows and for both priming callbacks and all cached drains. There is no synthetic flush at EOF.

All work is built in local owned values. A failure returns no proposal or
normal output and cannot mutate the committed input state. The callback parent
remains responsible for retirement/reset and the only eventual root commit.

## Specification and vector identities

The complete approved package manifest SHA-256 is
`c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`.
The mapping gate verifies every source below against that manifest before it
allows this map to pass.

| Artifact | Bytes | SHA-256 | Mapped use |
|---|---:|---|---|
| `specs/04-window-matrix-and-component-audio.md` | 9190 | `350937eadb5da2432ed60723305cdc8de80a139fef803ec3a2aa5a3990ed6334` | F04A.6-.8 |
| `specs/04-component-parameter-state.md` | 5792 | `3e9b777a36ae722aeccd1054d778484e6a96e44d16f2534159dfe49afc7d1861` | F04P.1-.5 |
| `specs/04-component-parameter-generation.md` | 4910 | `2564879656a5b0e73975d99ef7a55217440131c5d4e2d6b4dc8117adecef76d8` | F04PG.1-.5 |
| `specs/04-whole-spatial-chain-composition.md` | 9103 | `92325b12128023001e1f3fd4696dcec68bbd47a46acdb7886781b411a10c5d59` | F04C.1-.2, .6-.7 |
| `specs/00-numeric-semantics.md` | 3369 | `53cb67d6c9ddced1fb909dda806bb478a2926a66f2e38cecaada6f6e95d4d135` | N00.1-.4 |
| `parameters/window-q15.json` | 2704 | `a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130` | `component_kernel64` typed input |
| `parameters/component-parameter-mathematical-v1.json` | 1077 | `5d0fec6f62a33eeef5c64016760fcbe94a1872a564b6d0e03f959c95c37eb0f5` | eight typed controller constants |

The mapped package vectors are:

| Vector | Bytes / SHA-256 | Exact input pointer | Exact expected pointer used |
|---|---|---|---|
| `test-vectors/unit/vectors.json` | 1,503,142 / `a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8` | `cases[id=U-FRONT-MATRIX].input`; `cases[id=U-FRONT-OVERLAP].input`; `cases[id=U-FRONT-FILTER64].input`; `cases[id=U-FRONT-PARAMETER64].input` | respective `.expected.row_sha256`/`.expected.probes_u32`; `.expected.calls`/terminal overlap and ring; `.expected.blocks`/terminal history; `.expected.records`/terminal state |
| `test-vectors/COMPONENT-PARAMETER-GENERATION-VERIFICATION.json` | 55,848 / `819e7cede6f54689a87aad72c8ec3d8a5cb4b14fe41fc6a5556a03973f01ec85` | each `cases[id].execution` and `cases[id].input`; execution order follows `execution_plan` | each `cases[id].expected_before_words`, `.expected_after_words`, `.expected_state`, attenuation, peak, witness and branch fields; expected fields are assertions only, never seeds |
| `test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json` | 260,481 / `ffb1dea3b1e69383312da4059b9d8ddabb4366ed8fc22e7537f750ba041f2d63` | `cases[id=F04-WHOLE-0..5].input` plus the corresponding `recipe_definitions` | `.expected.matrix15x3072_f32le_sha256`, `.expected.overlap15x1536_f32le_sha256`, `.expected.component15x1536_f32le_sha256`, `.expected.component_parameter_blocks_after`; unrelated LFE/receiver/QMF fields are not consumed |

F04C.8's six callback input recipes are `input_only=true`; they qualify only
the consumer boundary. F04A as a whole remains partial, F04P/F04PG remain
model-subset specifications, and upstream installed48/matrix production,
render fanout, the rest of F04, and parent transaction are deferred.

## Planned implementation files

- `crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs`
- `crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage_tests.rs`
- `crates/jocforge-domain/src/spatial/spatial_audio_state.rs`
- `crates/jocforge-domain/src/spatial/mod.rs`
- `crates/jocforge-domain/tests/fixtures/f04_component/unit-vectors.json`
- `crates/jocforge-domain/tests/fixtures/f04_component/COMPONENT-PARAMETER-GENERATION-VERIFICATION.json`
- `crates/jocforge-domain/tests/fixtures/f04_component/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json`

The three vector fixture files are exact package copies and retain their
package-manifest identities. They are test-only inputs; production code has no
file, process, or environment access. The module-private test constructor is
not a production packet factory.
