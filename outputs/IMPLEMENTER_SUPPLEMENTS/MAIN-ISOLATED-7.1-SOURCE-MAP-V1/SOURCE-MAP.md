# Main → isolated source map for 7.1 Flat7x / 1664 kbps

## Finding

Wash did reference the main branch, and a sealed architecture matrix already records the useful parts. That matrix is bound to an older isolated snapshot (`9456d4c`); the current source has since gained a D0 PCM-to-frame candidate and additional I0+D0 pairing code. This refresh binds the comparison to isolated HEAD `1acf9c2`.

The main implementation contributes one useful architecture shape: represent I0 and dependent frames as separate roles, define ownership explicitly, then assemble the frames in I0→D0 order. The isolated code already has the stronger target-specific plan, pair candidates, source receipts, and F15/atomic-staging seams. DEE reverse engineering is still relevant to Dolby-specific semantics; it should not be used to rediscover I0→D0 composition or replace the current isolated architecture.

The approved target is one 640-kbps I0 frame (2,560 bytes) followed by one 1,024-kbps D0 frame (4,096 bytes), for 1,664 kbps / 6,656 bytes per AU. The official demo observation has a different component-rate split (640+640 kbps); the user target adopts its I0+D0 structure with the separately selected 640+1,024 split.

## Current code flow and connection status

### CLI route as written

`jocforge-cli/src/encode_request.rs::run_with_data_rate_and_joc_layout` resolves `1024 kbps + Flat7x` to `EncodeRequestRoute::Flat7xI0D0Pair(Flat7xI0D0TargetPlan)`. The route passes the pair target's D0 rate to the worker; the target plan carries the 640-kbps I0 rate separately.

In `jocforge-cli/src/source_execution.rs`, the selected F12 source-frame candidate and Flat7x source binding are prepared first. The current Flat7x branch then calls `require_flat7x_d0_audio_blocks` with `Disabled(NoApprovedAudioProductionRule)` and returns `Flat7xD0AudioProducerUnavailable` before staging F12 integrity receipts or preparing an F15 output proposal. This is the current source-level stop.

The comment on `ProgrammeOutputStaging::append_flat7x_au` says the CLI rejects Flat7x “before source work”; the current source code reaches the F12 candidate and pair-source binding before the D0 producer stop. Treat the executable source path as authoritative and refresh that comment when the route is wired.

### Candidate modules that exist outside the live CLI route

`jocforge-application/src/flat7x_pair_source.rs` has:

1. `bind_flat7x_pair_source`: joins the selected Flat7x target, PCM projection receipt, and numeric-plan identity.
2. `produce_flat7x_pair_d0_pcm_frame_candidate`: calls the domain D0 PCM frame producer and retains the D0 source identity and role-map receipt with the frame/history candidate.
3. `assemble_flat7x_pair_i0_frame_candidate`: calls the target-specific AC-3 I0 frame assembler.
4. `assemble_flat7x_pair_access_unit_candidate`: combines an I0 frame candidate with target BSI and D0 block inputs, then calls the domain I0→D0 AU candidate assembler.

The full D0 PCM candidate route and the D0 audio-span input used by `Flat7xPairAudioBlockCandidate` are currently separate candidate seams. The D0 PCM candidate and complete pair candidate have no production call sites in the CLI. Select one canonical path that carries the completed D0 frame and its history proposal into the AU candidate; avoid rebuilding the same D0 frame from a second unrelated span input.

`jocforge-application/src/flat7x_pair_output.rs::preflight_flat7x_pair_io_stage` validates same-source identity, frame extents, I0→D0 span order, block-two skip-field size, and the disabled F15 owner. Its current callers are tests; the function deliberately rejects absent F13/I0/D0 receipts and does not reach I/O.

`stage_receipted_i0_with_opaque_d0` can create a complete pair F15 proposal when given the explicit source-generation label, D0 frame, F13 metadata receipt, and I0/D0 integrity receipts. `ProgrammeOutputStaging::append_flat7x_pair` and `append_flat7x_au` connect that proposal to the existing atomic output adapter, but the CLI pair append remains uncalled. `jocforge-io/src/atomic_output/mod.rs::append_flat7x_au` and `flat7x_au_append.rs::append_flat7x_au_to_staging_file` provide the lower I/O seam.

## Reuse matrix with exact source locations

| Concern | Main reference | Current isolated seam | Reuse decision |
|---|---|---|---|
| I0→D0 ordering | `writer.rs::package_source_derived_access_unit_with_frame_sizes` validates an external I0 frame, builds D0, then appends I0 and D0 to one in-memory vector (1192–1224). `package_multi_dependent_source_derived_access_unit_with_plan` validates an `OwnershipPlan`, appends I0, then loops over dependent frames (1925–1999). | `codec/access_unit_composer.rs::assemble_flat7x_i0_d0_au_candidate[_with_tail_policy]` (179–285); application `Flat7xPairAccessUnitCandidate` and `assemble_flat7x_pair_access_unit_candidate` (498–582; 547–582). | Reuse explicit frame roles and order. Keep the isolated typed candidate and its source receipts. Main's returned `Vec<u8>` establishes in-memory concatenation only. |
| Role ownership | `conformance.rs::DependentPlan { id, channel_map, lfe_policy }` and `OwnershipPlan { profile, independent_lfe, dependents, final_joc_owner }` (201–244). | `Flat7xFrameTarget`, `Flat7xI0D0TargetPlan`, `Flat7xPairSourceBinding`, `Flat7xD0SourceIdentity`, `Flat7xSourceRoleMapReceipt`, `Flat7xAuOutputOwner`, and typed F13/I0/D0 receipts. | Reuse role/ownership separation. Keep isolated typed identities and explicit generation; do not copy the legacy plan type or infer source ownership from equal values or frame adjacency. |
| Profile and rate allocation | `profile.rs` old `Flat-7.X` row selects `channel_map=0x0200` and source indices `[5, 6]` (135–143). `rate.rs::rate_budget(1664)` chooses 1024+640 kbps (38–61). | `profile/flat7x_access_unit_target.rs::Flat7xI0D0TargetPlan::approved_contract` stores I0 AC-3/BSID 6 at 640/2,560 and D0 E-AC-3/BSID 16 at 1,024/4,096 with `chanmap=0x1A00` (239–347). `aggregate_rate_kbps` derives 1,664 (385–395). | Keep the target plan's per-substream values. The old map and old split are incompatible. Resolve the 7.1 pair through an explicit pair profile/route; do not make the existing 5.1 scalar-rate row pretend to own an I0+D0 topology. |
| Existing 5.1 profile | No main code is needed to define the 5.1 preservation rule. | `profile/production.rs::EncodingProfile` keeps the selected 5.1 profile, scalar `CodecRateProfile`, and `SelectedSingleIndependentZeroDependent`; the Flat7x target is a separate design-only plan. `rate_profile.rs` and `substream_topology.rs` remain module-contract stubs at this snapshot. | Preserve the existing 5.1 resolver and audio path. Avoid changing five-lane/JOC geometry globally. Keep one authoritative per-substream plan for the Flat7x branch. |
| I0 frame production | Main packaging receives I0 bytes as an input; it does not define the isolated target's native I0 source path. | `codec/i0_frame_assembler.rs::assemble_flat7x_i0_ac3_frame_from_audio_series` (944 onward) and `flat7x_pair_source.rs::assemble_flat7x_pair_i0_frame_candidate` (428 onward). | Reuse the isolated I0 audio-series and frame assembler at its typed seam. Preserve the selected 5.1 E-AC-3 path as a separate profile. |
| D0 BSI/audio | Main `write_source_dependent_frame` is documented as a source-derived conventional mantissa bridge, not a perceptual E-AC-3 encoder (`writer.rs` 801–820). Its profile map and audio path are legacy product logic. | Target BSI writers live in `codec/frame_builder.rs` (108, 196, 242, 311). D0 PCM-to-frame candidate production lives in `codec/flat7x_d0_pcm_source.rs::produce_flat7x_d0_frame_from_pcm_source` (160–188); the application source wrapper is in `flat7x_pair_source.rs` (281–333). | Do not port the main D0 writer, channel map, FFmpeg path, or audio algorithm. Continue the isolated D0-specific path from its current candidate seam. Native DEE behavior and production ownership remain separate evidence gates. |
| Source-row mapping | Main `d0_source_indices` describe its own old profile row; they do not establish the target ADM role map. | `profile/flat7x_source_role_map.rs` and `application/source_frame_bridge.rs::build_flat7x_source_pcm_projection` (209 onward) construct a typed, replaceable projection receipt. The receipt explicitly labels its row-to-role policy provisional (59–64). | Keep physical ADM rows, I0 coded channels, D0 `chanmap` locations, and seven JOC reconstruction carriers as separate domains. Wash must close the role join before production authorization. |
| Output publication | Main packaging returns concatenated memory bytes; it does not prove durable pair publication. | `flat7x_pair_output.rs::stage_receipted_i0_with_opaque_d0` (301–435), `cli/output_publication.rs::append_flat7x_pair` (222 onward), and `io/atomic_output/mod.rs::append_flat7x_au` (317–352). | Reuse the isolated F15 receipt and atomic-staging seam. Wire one validated pair proposal and one ordered append only after same-generation, F13, and both frame-integrity receipts exist. |

## Maintainable implementation order

1. **Make the pair plan authoritative.** Use `Flat7xI0D0TargetPlan` (or one successor with the same responsibility) as the source for I0/D0 roles, per-frame rates and byte extents, topology, channel map, reconstruction layout, and EMDF destination. The current `EncodingProfile` still encodes a single-independent/zero-dependent topology; do not pass that profile as though it described the pair. Keep the selected 5.1 profile unchanged.
2. **Select one completed D0 frame seam.** The code has a D0 PCM-frame candidate and a separate caller-owned D0-span frame-assembly route. Join the D0 PCM/history candidate to the pair AU candidate once, preserving the role-map receipt and source identity. The role-map policy is provisional and remains a Wash gate.
3. **Assemble one complete AU candidate.** Reuse `access_unit_composer` and existing I0/D0 assemblers. Validate 2,560-byte I0, 4,096-byte D0, I0→D0 byte order, one block-index-2 D0 skip-field with complete container `C<=511`, and whole-AU rejection before append.
4. **Bind the real owner receipts.** Produce F13 metadata and I0/D0 integrity receipts for the same programme, attempt, callback transaction, source index, published AU, and explicit pair source generation. Keep source generation as its own typed value.
5. **Wire the CLI branch to the pair staging seam.** Branch on the resolved pair target before the ordinary single-frame F12 output route is treated as the final I0. Feed the validated pair proposal to `append_flat7x_pair` and the existing atomic pair adapter exactly once. Preserve the 5.1 route and its byte behavior.
6. **Keep external evidence gates honest.** The target still records open gates for the ADM role join, downmix controls, runtime generation, native writer owner, and all-AU EMDF fit. The local D0 frame candidate is implementation progress; it is not a real-output or DEE-equivalence result. Production and output remain `NOT_RUN` here.

## DEE reverse engineering that remains useful

Keep reverse work focused on unresolved Dolby-specific semantics that the source map cannot answer: native D0 BSI/writer ownership, any required mode/phase behavior, the ADM row-to-D0 role relation, and any native state/generation relationship needed to validate a claimed DEE-equivalent path. Main-branch inspection already answers the architecture question for explicit I0→D0 assembly and ownership shape.

`production_allowed=false`. This package makes no output, decoder, or listening claim.
