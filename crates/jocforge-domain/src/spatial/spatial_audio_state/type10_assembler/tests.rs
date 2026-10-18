// pattern: Imperative Shell
use super::{Type10AssemblyError, Type10AssemblyOutcome, assemble_type10};
use crate::{
    numeric::FiniteF32,
    profile::{EmdfCarriageProfile, EncodingProfile},
    spatial::{
        SpatialControlProviderLineage,
        post_window_fanout::CallbackPhase,
        source_flags::{HostId, SourceFlagsAttempt, SourceFlagsProgrammeId},
        spatial_audio_state::{
            LfeDelayRingState, SpatialAudioDigest, SpatialAudioJoinMetadata, SpatialAudioMachine,
            component_stage::{ComponentCallbackPhase, ComponentStageProposal},
            direct_lfe::{LfeStageProposal, Type10LfeRow0Input},
        },
    },
    timeline::FrontendCallbackIndex,
    transaction::CallbackTransactionId,
};
use std::sync::Arc;

const FRAME_SAMPLES: usize = 1536;
const COMPONENT_ROWS: usize = 15;
const TYPE10_ROWS: usize = 16;
const DIRECT_LFE_VECTORS: &[u8] =
    include_bytes!("../../../../tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json");

struct TestContext {
    spatial_audio: SpatialAudioMachine,
    spatial_control: crate::spatial::SpatialControlMachine,
    provider_lineage: [crate::spatial::F04ParameterProviderLineage; 2],
    profile: EncodingProfile,
}

fn context() -> TestContext {
    let (spatial_audio, spatial_control, provider_lineage, profile) =
        crate::spatial::spatial_audio_state::component_stage::tests::type10_test_context();
    TestContext {
        spatial_audio,
        spatial_control,
        provider_lineage,
        profile,
    }
}

fn vectors() -> serde_json::Value {
    serde_json::from_slice(DIRECT_LFE_VECTORS).expect("the checked-in direct-LFE vector is valid")
}

fn vector_phase(document: &serde_json::Value, callback: u64) -> CallbackPhase {
    let case = &document["cases"][usize::try_from(callback).expect("bounded test callback")];
    assert_eq!(
        case["id"].as_str(),
        Some(["g0", "g1", "g2", "g3", "g4", "g5"][callback as usize])
    );
    assert_eq!(case["input"]["generation"].as_u64(), Some(callback));
    match case["input"]["phase"].as_str() {
        Some("NORMAL") => CallbackPhase::Normal,
        Some("CACHED_DRAIN") => CallbackPhase::CachedDrain,
        other => panic!("unexpected fixture phase: {other:?}"),
    }
}

#[allow(clippy::too_many_arguments)]
fn parent_join(
    _context: &TestContext,
    callback: u64,
    programme: &str,
    attempt: u64,
    transaction: &str,
    profile: EncodingProfile,
    before_generation: u64,
    before_digest: SpatialAudioDigest,
) -> SpatialAudioJoinMetadata {
    SpatialAudioJoinMetadata::new(
        SourceFlagsProgrammeId::new(programme).expect("fixture programme id"),
        SourceFlagsAttempt::new(attempt),
        CallbackTransactionId::new(transaction).expect("fixture transaction id"),
        FrontendCallbackIndex::new(callback),
        before_generation,
        before_digest,
        profile,
    )
}

fn selected_parent(context: &TestContext, callback: u64) -> SpatialAudioJoinMetadata {
    parent_join(
        context,
        callback,
        "type10-test-programme",
        7,
        "type10-test-transaction",
        context.profile,
        context.spatial_audio.generation(),
        context.spatial_audio.digest(),
    )
}

fn component_values(callback: u64, length: usize) -> Vec<FiniteF32> {
    (0..length)
        .map(|word| {
            let marker = callback
                .checked_mul(32_768)
                .and_then(|base| base.checked_add(u64::try_from(word).ok()?))
                .expect("bounded test marker");
            FiniteF32::new(f32::from_bits(0x3f00_0000 + u32::try_from(marker).unwrap()))
                .expect("test marker is finite")
        })
        .collect()
}

fn model_samples_after(callback: u64) -> i64 {
    let callbacks = callback.checked_add(1).expect("bounded callback count");
    let samples = callbacks
        .checked_mul(u64::try_from(FRAME_SAMPLES).unwrap())
        .expect("bounded test sample count");
    i64::try_from(samples).expect("test sample count fits the model counter")
}

fn component_proposal(
    context: &TestContext,
    join: &SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    control_provider_lineage: Option<&SpatialControlProviderLineage>,
    output_length: usize,
) -> ComponentStageProposal {
    let component_phase = match phase {
        CallbackPhase::Normal => ComponentCallbackPhase::Normal,
        CallbackPhase::CachedDrain => ComponentCallbackPhase::CachedDrain,
    };
    ComponentStageProposal::fixture_for_type10(
        join,
        component_phase,
        &context.spatial_control,
        context.provider_lineage,
        control_provider_lineage,
        component_values(join.callback_index().value(), output_length),
    )
}

fn lfe_row_bytes(source_generation: u64) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(FRAME_SAMPLES * 4);
    for sample in 0..FRAME_SAMPLES {
        let marker = source_generation
            .checked_mul(FRAME_SAMPLES as u64)
            .and_then(|base| base.checked_add(u64::try_from(sample).ok()?))
            .expect("bounded test marker");
        let bits = 0x3e80_0000 + u32::try_from(marker).unwrap();
        bytes.extend_from_slice(&bits.to_le_bytes());
    }
    bytes
}

fn ring_after(callback: u64, corrupt_source: Option<u64>) -> LfeDelayRingState {
    let mut ring = LfeDelayRingState::cold();
    ring.write_index = i64::try_from((callback + 1) % 3).unwrap();
    ring.read_index = (ring.write_index + 1) % 3;
    ring.model_samples_consumed = model_samples_after(callback);
    let first_source = callback.saturating_sub(2);
    for source in first_source..=callback {
        let slot = usize::try_from((source + 1) % 3).unwrap();
        let mut bytes = lfe_row_bytes(source);
        if corrupt_source == Some(source) {
            bytes[0] ^= 1;
        }
        ring.rows[slot] = Some(Arc::from(bytes.into_boxed_slice()));
        ring.input_frame_indexes[slot] = Some(FrontendCallbackIndex::new(source));
    }
    ring
}

fn lfe_row0(callback: u64, origin: u64, byte_override: Option<Vec<u8>>) -> Type10LfeRow0Input {
    Type10LfeRow0Input::fixture_for_type10(
        FrontendCallbackIndex::new(callback),
        FrontendCallbackIndex::new(origin),
        byte_override.unwrap_or_else(|| lfe_row_bytes(origin)),
    )
}

fn lfe_proposal(
    context: &TestContext,
    join: &SpatialAudioJoinMetadata,
    phase: CallbackPhase,
    row0: Option<Type10LfeRow0Input>,
    after: LfeDelayRingState,
    control_provider_lineage: Option<&SpatialControlProviderLineage>,
) -> LfeStageProposal {
    LfeStageProposal::fixture_for_type10(
        join,
        phase,
        &context.spatial_control,
        HostId::new(2).expect("test host id is in range"),
        after,
        row0,
        control_provider_lineage,
    )
}

fn valid_pair(
    context: &TestContext,
    callback: u64,
) -> (
    SpatialAudioJoinMetadata,
    CallbackPhase,
    ComponentStageProposal,
    LfeStageProposal,
) {
    let document = vectors();
    let phase = vector_phase(&document, callback);
    let parent = selected_parent(context, callback);
    let component = component_proposal(
        context,
        &parent,
        phase,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let after = ring_after(callback, None);
    let row0 = callback
        .checked_sub(2)
        .map(|origin| lfe_row0(callback, origin, None));
    let lfe = lfe_proposal(context, &parent, phase, row0, after, None);
    (parent, phase, component, lfe)
}

#[allow(clippy::needless_pass_by_value)]
fn assert_error(
    result: Result<super::Type10AssemblyProposal, Type10AssemblyError>,
    expected: Type10AssemblyError,
) {
    match result {
        Err(actual) => assert_eq!(actual, expected),
        Ok(_) => panic!("expected Type10 assembly rejection"),
    }
}

#[test]
fn g0_g1_priming_without_packet() {
    let context = context();
    let document = vectors();
    for callback in 0..=1 {
        let phase = vector_phase(&document, callback);
        let parent = selected_parent(&context, callback);
        let component = component_proposal(
            &context,
            &parent,
            phase,
            None,
            COMPONENT_ROWS * FRAME_SAMPLES,
        );
        let lfe = lfe_proposal(
            &context,
            &parent,
            phase,
            None,
            ring_after(callback, None),
            None,
        );
        let stage = assemble_type10(&parent, phase, component, lfe)
            .expect("priming callbacks return a staged no-packet outcome");
        let (_, lfe_after, outcome) = stage.into_parts();
        assert_eq!(
            lfe_after.model_samples_consumed,
            model_samples_after(callback)
        );
        assert!(matches!(outcome, Type10AssemblyOutcome::Priming { .. }));
        assert_eq!(
            document["cases"][callback as usize]["expected_type10"]["available"],
            false
        );
    }
}

#[test]
fn g2_g5_row_order_origins_and_bit_preservation() {
    let context = context();
    let document = vectors();
    for callback in 2..=5 {
        let phase = vector_phase(&document, callback);
        let parent = selected_parent(&context, callback);
        let expected_component = component_values(callback, COMPONENT_ROWS * FRAME_SAMPLES);
        let component = ComponentStageProposal::fixture_for_type10(
            &parent,
            match phase {
                CallbackPhase::Normal => ComponentCallbackPhase::Normal,
                CallbackPhase::CachedDrain => ComponentCallbackPhase::CachedDrain,
            },
            &context.spatial_control,
            context.provider_lineage,
            None,
            expected_component.clone(),
        );
        let origin = callback - 2;
        let row0 = lfe_row0(callback, origin, None);
        let after = ring_after(callback, None);
        let lfe = lfe_proposal(&context, &parent, phase, Some(row0), after, None);
        let stage = assemble_type10(&parent, phase, component, lfe)
            .expect("valid row0 and component rows assemble");
        let (component_after, lfe_after, outcome) = stage.into_parts();
        assert_eq!(
            component_after.audio.overlap_history_rows.len(),
            COMPONENT_ROWS
        );
        assert_eq!(
            lfe_after.model_samples_consumed,
            model_samples_after(callback)
        );
        let Type10AssemblyOutcome::Ready { packet, receipt } = outcome else {
            panic!("g={callback} must produce a packet")
        };
        assert_eq!(packet.shape(), [TYPE10_ROWS, FRAME_SAMPLES]);
        assert_eq!(receipt.shape(), [TYPE10_ROWS, FRAME_SAMPLES]);
        assert_eq!(packet.payload().len(), TYPE10_ROWS * FRAME_SAMPLES);
        assert_eq!(
            packet.callback_index(),
            FrontendCallbackIndex::new(callback)
        );
        assert_eq!(
            receipt.component_origin_generation(),
            FrontendCallbackIndex::new(callback)
        );
        assert_eq!(
            receipt.lfe_origin_generation(),
            Some(FrontendCallbackIndex::new(origin))
        );
        assert_eq!(receipt.programme_id().as_str(), "type10-test-programme");
        assert_eq!(receipt.attempt(), SourceFlagsAttempt::new(7));
        assert_eq!(receipt.transaction_id().as_str(), "type10-test-transaction");
        assert_eq!(receipt.phase(), phase);
        assert_eq!(receipt.profile_identity(), context.profile);
        assert_eq!(
            receipt.spatial_audio_before_generation(),
            parent.before_generation()
        );
        assert_eq!(
            receipt.spatial_audio_before_digest(),
            parent.before_digest()
        );
        assert_eq!(
            receipt.spatial_control_before_generation(),
            context.spatial_control.generation()
        );
        assert_eq!(
            receipt.spatial_control_before_digest(),
            context.spatial_control.digest()
        );
        assert_eq!(
            receipt.spatial_control_provider_lineage(),
            context.spatial_control.provider_lineage()
        );
        assert_eq!(receipt.f04_provider_lineage(), context.provider_lineage);
        assert_eq!(receipt.lfe_host(), HostId::new(2).unwrap());
        assert_eq!(
            document["cases"][callback as usize]["expected_type10"]["row0_origin_generation"]
                .as_u64(),
            Some(origin)
        );
        assert_eq!(
            document["cases"][callback as usize]["expected_type10"]["rows_1_15_origin_generation"]
                .as_u64(),
            Some(callback)
        );
        let expected_lfe = lfe_row_bytes(origin);
        for sample in 0..FRAME_SAMPLES {
            let start = sample * 4;
            let expected_bits =
                u32::from_le_bytes(expected_lfe[start..start + 4].try_into().unwrap());
            assert_eq!(
                packet.payload()[sample].to_bits(),
                expected_bits,
                "row0 sample {sample}"
            );
        }
        for component_row in 0..COMPONENT_ROWS {
            for sample in 0..FRAME_SAMPLES {
                let target = (component_row + 1) * FRAME_SAMPLES + sample;
                let source = component_row * FRAME_SAMPLES + sample;
                assert_eq!(
                    packet.payload()[target].to_bits(),
                    expected_component[source].to_bits(),
                    "Type10 row {} sample {sample}",
                    component_row + 1
                );
            }
        }
    }
}

#[test]
fn component_lfe_programme_attempt_mismatch() {
    let context = context();
    let parent = selected_parent(&context, 0);
    let phase = CallbackPhase::Normal;
    let mut wrong_parent = parent_join(
        &context,
        0,
        "other-programme",
        8,
        "type10-test-transaction",
        context.profile,
        parent.before_generation(),
        parent.before_digest(),
    );
    let component = component_proposal(
        &context,
        &wrong_parent,
        phase,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(&context, &parent, phase, None, ring_after(0, None), None);
    assert_error(
        assemble_type10(&parent, phase, component, lfe),
        Type10AssemblyError::Input,
    );
    wrong_parent = parent_join(
        &context,
        0,
        "type10-test-programme",
        8,
        "type10-test-transaction",
        context.profile,
        parent.before_generation(),
        parent.before_digest(),
    );
    let component = component_proposal(
        &context,
        &wrong_parent,
        phase,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(&context, &parent, phase, None, ring_after(0, None), None);
    assert_error(
        assemble_type10(&parent, phase, component, lfe),
        Type10AssemblyError::Input,
    );
}

#[test]
fn callback_phase_mismatch() {
    let context = context();
    let parent = selected_parent(&context, 0);
    let wrong_callback = parent_join(
        &context,
        1,
        "type10-test-programme",
        7,
        "type10-test-transaction",
        context.profile,
        parent.before_generation(),
        parent.before_digest(),
    );
    let component = component_proposal(
        &context,
        &wrong_callback,
        CallbackPhase::Normal,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::Normal,
        None,
        ring_after(0, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::Normal, component, lfe),
        Type10AssemblyError::CallbackOrder,
    );

    let component = component_proposal(
        &context,
        &parent,
        CallbackPhase::CachedDrain,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::Normal,
        None,
        ring_after(0, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::Normal, component, lfe),
        Type10AssemblyError::CallbackOrder,
    );
}

#[test]
fn transaction_mismatch() {
    let context = context();
    let parent = selected_parent(&context, 0);
    let wrong_transaction = parent_join(
        &context,
        0,
        "type10-test-programme",
        7,
        "different-transaction",
        context.profile,
        parent.before_generation(),
        parent.before_digest(),
    );
    let component = component_proposal(
        &context,
        &wrong_transaction,
        CallbackPhase::Normal,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::Normal,
        None,
        ring_after(0, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::Normal, component, lfe),
        Type10AssemblyError::Input,
    );
}

#[test]
fn profile_mismatch() {
    let context = context();
    let parent = selected_parent(&context, 0);
    let other_profile = EncodingProfile::selected_with_carriage(EmdfCarriageProfile::EtsiEmdf);
    let wrong_profile = parent_join(
        &context,
        0,
        "type10-test-programme",
        7,
        "type10-test-transaction",
        other_profile,
        parent.before_generation(),
        parent.before_digest(),
    );
    let component = component_proposal(
        &context,
        &wrong_profile,
        CallbackPhase::Normal,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::Normal,
        None,
        ring_after(0, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::Normal, component, lfe),
        Type10AssemblyError::ProfileUnsupported,
    );
}

#[test]
fn provider_lineage_mismatch() {
    let context = context();
    let parent = selected_parent(&context, 0);
    let mismatched = SpatialControlProviderLineage::fixture_mismatch_for_type10(
        context.spatial_control.provider_lineage(),
    );
    let component = component_proposal(
        &context,
        &parent,
        CallbackPhase::Normal,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::Normal,
        None,
        ring_after(0, None),
        Some(&mismatched),
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::Normal, component, lfe),
        Type10AssemblyError::Input,
    );
}

#[test]
fn stale_spatial_audio_before_generation() {
    let context = context();
    let parent = selected_parent(&context, 0);
    let stale = parent_join(
        &context,
        0,
        "type10-test-programme",
        7,
        "type10-test-transaction",
        context.profile,
        parent.before_generation() + 1,
        parent.before_digest(),
    );
    let component = component_proposal(
        &context,
        &stale,
        CallbackPhase::Normal,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::Normal,
        None,
        ring_after(0, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::Normal, component, lfe),
        Type10AssemblyError::State,
    );
}

#[test]
fn stale_spatial_audio_before_digest() {
    let context = context();
    let parent = selected_parent(&context, 0);
    let stale_digest = SpatialAudioDigest::from_canonical_bytes(b"stale type10 before root");
    let stale = parent_join(
        &context,
        0,
        "type10-test-programme",
        7,
        "type10-test-transaction",
        context.profile,
        parent.before_generation(),
        stale_digest,
    );
    let component = component_proposal(
        &context,
        &parent,
        CallbackPhase::Normal,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &stale,
        CallbackPhase::Normal,
        None,
        ring_after(0, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::Normal, component, lfe),
        Type10AssemblyError::State,
    );
}

#[test]
fn missing_row0_after_priming() {
    let context = context();
    let parent = selected_parent(&context, 2);
    let component = component_proposal(
        &context,
        &parent,
        CallbackPhase::CachedDrain,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::CachedDrain,
        None,
        ring_after(2, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::CachedDrain, component, lfe),
        Type10AssemblyError::State,
    );
}

#[test]
fn row0_origin_or_ring_slot_mismatch() {
    let context = context();
    let parent = selected_parent(&context, 2);
    let wrong_origin = lfe_row0(2, 1, None);
    let component = component_proposal(
        &context,
        &parent,
        CallbackPhase::CachedDrain,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::CachedDrain,
        Some(wrong_origin),
        ring_after(2, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::CachedDrain, component, lfe),
        Type10AssemblyError::State,
    );

    let mut mismatched_bytes = lfe_row_bytes(0);
    mismatched_bytes[0] ^= 1;
    let expected_origin = lfe_row0(2, 0, Some(mismatched_bytes));
    let component = component_proposal(
        &context,
        &parent,
        CallbackPhase::CachedDrain,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::CachedDrain,
        Some(expected_origin),
        ring_after(2, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::CachedDrain, component, lfe),
        Type10AssemblyError::State,
    );
}

#[test]
fn malformed_component_output_shape() {
    let context = context();
    let parent = selected_parent(&context, 2);
    let component = component_proposal(
        &context,
        &parent,
        CallbackPhase::CachedDrain,
        None,
        COMPONENT_ROWS * FRAME_SAMPLES - 1,
    );
    let lfe = lfe_proposal(
        &context,
        &parent,
        CallbackPhase::CachedDrain,
        Some(lfe_row0(2, 0, None)),
        ring_after(2, None),
        None,
    );
    assert_error(
        assemble_type10(&parent, CallbackPhase::CachedDrain, component, lfe),
        Type10AssemblyError::State,
    );
}

#[test]
fn dropped_stage_preserves_committed_root() {
    let context = context();
    let before_generation = context.spatial_audio.generation();
    let before_digest = context.spatial_audio.digest();
    let before_bytes = context.spatial_audio.committed().canonical_bytes().to_vec();
    let (parent, phase, component, lfe) = valid_pair(&context, 2);
    let stage = assemble_type10(&parent, phase, component, lfe)
        .expect("the callback-local assembly proposal is discardable");
    drop(stage);
    assert_eq!(context.spatial_audio.generation(), before_generation);
    assert_eq!(context.spatial_audio.digest(), before_digest);
    assert_eq!(
        context.spatial_audio.committed().canonical_bytes(),
        before_bytes
    );
}
