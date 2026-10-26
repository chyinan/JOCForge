use super::*;
use crate::profile::{EncodingProfile, resolve_selected_profile};
use crate::spatial::source_flags::{
    HostId, project_normal_control_snapshot_source_controls_from_prepared,
};

const SNAPSHOT_INPUTS: &str = include_str!(
    "../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/FRONTEND-NORMAL-CONTROL-SNAPSHOT-BINDING-V1/INPUT-ONLY-VECTORS.json"
);
const OWNER_INPUTS: &str = include_str!(
    "../../../../../outputs/IMPLEMENTER_SUPPLEMENTS/OBJECT-TRACK-SAMPLER-OWNER-BINDING-V1/INPUT-ONLY-VECTORS.json"
);

#[test]
fn proposal_emits_physical_rows_and_uid_indexed_sampler_state() {
    let prepared = synthetic_prepared_snapshot();
    let before = cold_before_state(&prepared);
    let before_copy = before.clone();

    let proposal =
        propose_normal_control_snapshot(&before, &prepared, transaction_id(&prepared, 0)).unwrap();

    assert_eq!(before, before_copy);
    assert_eq!(proposal.value_generation().value(), 0);
    assert_eq!(proposal.normal_origin_generation().value(), 0);
    assert_eq!(proposal.query_sample(), 0);
    assert_eq!(proposal.actual_samples(), 1_536);
    assert_eq!(proposal.control38_rows().len(), PHYSICAL_TRACK_COUNT);
    assert_eq!(proposal.regular37_rows().len(), REGULAR_TRACK_COUNT);
    assert_eq!(
        proposal
            .control38_rows()
            .iter()
            .map(NormalControlRow::physical_host)
            .collect::<Vec<_>>(),
        (0..38).collect::<Vec<_>>()
    );
    assert_eq!(
        proposal
            .regular37_rows()
            .iter()
            .map(NormalControlRow::physical_host)
            .collect::<Vec<_>>(),
        (0..38).filter(|host| *host != 12).collect::<Vec<_>>()
    );
    let expected_invocation_hosts: [u8; OBJECT_TRACK_COUNT] = prepared
        .physical_tracks
        .iter()
        .filter(|track| matches!(track.kind, NormalControlTrackKind::Object { .. }))
        .map(|track| track.physical_host)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    assert_eq!(proposal.invocation_hosts(), &expected_invocation_hosts);

    let bed = proposal
        .control38_rows()
        .iter()
        .find(|row| row.physical_host() == 12)
        .unwrap();
    let NormalControlRow::Bed6 {
        words,
        unproduced_tail,
        dbmd,
        ..
    } = bed
    else {
        panic!("host 12 is the semantic LFE bed in the input recipe");
    };
    assert_eq!(*words, [0, 0, 3, 2, 1, 3]);
    assert_eq!(unproduced_tail.offset_bytes(), 24);
    assert_eq!(unproduced_tail.byte_count(), 28);
    assert_eq!(dbmd.track_index(), 12);
    assert_eq!(bed.source_flags_view().mute_word(), 0);
    assert_eq!(bed.source_flags_view().spread_word(), 0.0_f32.to_bits());

    let object = proposal
        .control38_rows()
        .iter()
        .find_map(|row| match row {
            NormalControlRow::Object13 { control, .. } => Some((row, control.words())),
            NormalControlRow::Bed6 { .. } => None,
        })
        .unwrap();
    assert_eq!(object.0.source_flags_view().mute_word(), object.1[0]);
    assert_eq!(object.0.source_flags_view().spread_word(), object.1[9]);
    for row in proposal.control38_rows() {
        if let NormalControlRow::Object13 { control, dbmd, .. } = row {
            assert_eq!(control.words()[11], 0);
            assert_eq!(control.words()[12], 1);
            assert_eq!(dbmd.track_index(), u16::from(row.physical_host()));
        }
    }

    for (ordinal, binding) in prepared.owner_bindings.iter().enumerate() {
        let staged = &proposal.after_state().objects()[ordinal];
        assert_eq!(staged.object_ordinal(), u32::try_from(ordinal).unwrap());
        assert_eq!(
            staged.host_channel_index(),
            Some(u32::from(binding.physical_host))
        );
        assert!(staged.sampler_previous_present());
        assert_eq!(
            proposal.sampler_before()[ordinal].local_owner_ordinal(),
            u8::try_from(ordinal).unwrap()
        );
    }

    assert_eq!(
        proposal.control38_carrier_words().len(),
        CONTROL38_WORD_COUNT
    );
    assert_eq!(
        proposal.regular37_carrier_words().len(),
        REGULAR37_WORD_COUNT
    );
    let bed_tail = 12 * CONTROL_WORDS_PER_TRACK + 6;
    assert!(
        proposal.control38_carrier_words()[bed_tail..13 * 13]
            .iter()
            .all(|word| *word == 0)
    );
    for (regular_index, host) in prepared.regular37_hosts.iter().enumerate() {
        let row = &proposal.regular37_rows()[regular_index];
        assert_ne!(*host, 12);
        if let NormalControlRow::Bed6 { .. } = row {
            let tail_start = regular_index * CONTROL_WORDS_PER_TRACK + 6;
            let tail_end = (regular_index + 1) * CONTROL_WORDS_PER_TRACK;
            assert!(
                proposal.regular37_carrier_words()[tail_start..tail_end]
                    .iter()
                    .all(|word| *word == 0)
            );
        }
    }
}

#[test]
fn proposal_uses_carried_xyz_and_short_last_normal_frame() {
    let prepared = synthetic_prepared_snapshot();
    let cold = cold_before_state(&prepared);
    let first =
        propose_normal_control_snapshot(&cold, &prepared, transaction_id(&prepared, 0)).unwrap();
    let before_last = before_from_proposal(&first);

    let last =
        propose_normal_control_snapshot(&before_last, &prepared, transaction_id(&prepared, 1))
            .unwrap();

    assert_eq!(last.value_generation().value(), 1);
    assert_eq!(last.normal_origin_generation().value(), 1);
    assert_eq!(last.query_sample(), 1_536);
    assert_eq!(last.actual_samples(), 1);
    for sampler in last.sampler_before() {
        assert!(sampler.sampler_previous_present());
        assert_eq!(
            sampler.sampler_previous_xyz_u32(),
            first.after_state().objects()[usize::from(sampler.local_owner_ordinal())]
                .sampler_previous_xyz_u32()
        );
    }

    let carry_recipe: serde_json::Value = serde_json::from_str(SNAPSHOT_INPUTS).unwrap();
    let carry_words = carry_recipe["carry_only_case"]["change"]["sampler_previous_xyz_u32"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let mut alternate_before = before_last.clone();
    alternate_before.objects[0].sampler_previous_present = true;
    alternate_before.objects[0].sampler_previous_xyz_u32 = carry_words;
    let alternate =
        propose_normal_control_snapshot(&alternate_before, &prepared, transaction_id(&prepared, 1))
            .unwrap();
    let uid = &prepared.owner_bindings[0].track_uid;
    let row_for_uid = |proposal: &NormalControlSnapshotProposal| {
        proposal
            .control38_rows()
            .iter()
            .find(|row| row.track_uid() == uid)
            .and_then(|row| match row {
                NormalControlRow::Object13 { control, .. } => Some(control.words()),
                NormalControlRow::Bed6 { .. } => None,
            })
            .unwrap()
    };
    assert_ne!(row_for_uid(&last)[5..8], row_for_uid(&alternate)[5..8]);
}

#[test]
fn source_flags_projection_keeps_snapshot_clock_route_and_tagged_controls() {
    let prepared = synthetic_prepared_snapshot();
    let cold = cold_before_state(&prepared);
    let first =
        propose_normal_control_snapshot(&cold, &prepared, transaction_id(&prepared, 0)).unwrap();
    let first_projection = project_normal_control_snapshot_source_controls_from_prepared(
        &first,
        &prepared,
        prepared.profile_identity(),
    )
    .unwrap();

    assert_eq!(first_projection.clock().callback_index().value(), 0);
    assert_eq!(first_projection.clock().normal_callbacks().value(), 1);
    assert_eq!(first_projection.clock().source_origin().value(), 0);
    assert!(!first_projection.clock().is_cached_drain());
    assert_eq!(
        first_projection
            .route()
            .regular_hosts()
            .iter()
            .map(|host| host.value())
            .collect::<Vec<_>>(),
        prepared.regular37_hosts.to_vec()
    );
    assert_eq!(
        first_projection.controls().rows().len(),
        REGULAR_TRACK_COUNT
    );
    for (rank, (row, control)) in first
        .regular37_rows()
        .iter()
        .zip(first_projection.controls().rows())
        .enumerate()
    {
        let host = row.physical_host();
        let source_view = row.source_flags_view();
        assert_eq!(control.host().value(), host);
        assert_eq!(control.is_muted(), source_view.mute_word() == 1);
        assert_eq!(control.width().to_bits(), source_view.spread_word());
        let route_binding = prepared.regular_source_bindings[rank];
        assert_eq!(route_binding.physical_host(), host);
        assert_eq!(
            first_projection
                .route()
                .destination_for_host(HostId::new(host).unwrap()),
            Some(u8::try_from(route_binding.global_source()).unwrap())
        );
        assert_ne!(host, prepared.lfe_host);
    }

    let before_last = before_from_proposal(&first);
    let last =
        propose_normal_control_snapshot(&before_last, &prepared, transaction_id(&prepared, 1))
            .unwrap();
    let last_projection = project_normal_control_snapshot_source_controls_from_prepared(
        &last,
        &prepared,
        prepared.profile_identity(),
    )
    .unwrap();
    assert_eq!(last_projection.clock().callback_index().value(), 1);
    assert_eq!(last_projection.clock().normal_callbacks().value(), 2);
    assert_eq!(last_projection.clock().source_origin().value(), 1);
    assert!(!last_projection.clock().is_cached_drain());
}

#[test]
fn negative_zero_and_sentinel_inputs_reach_the_writer_conversion() {
    let mut negative_zero = (*synthetic_prepared_snapshot()).clone();
    let ordinal_zero_uid = negative_zero.owner_bindings[0].track_uid.clone();
    let event = find_object_event_mut(&mut negative_zero.object_events, &ordinal_zero_uid).unwrap();
    event.control_event = SelectedObjectEvent::new(
        event.physical_host,
        0,
        1_537,
        1_537,
        Some(0.0),
        Some(0.0),
        Some(-0.0),
        Some(-0.0),
        Some(-0.0),
        Some(-0.0),
        Some(0),
    );
    let writer_xyz = writer_coordinates(&event.control_event).unwrap();
    assert_eq!(writer_xyz[0], 0.5_f32.to_bits());
    assert_eq!(writer_xyz[1], 0.5_f32.to_bits());
    assert_eq!(writer_xyz[2], (-0.0_f32).to_bits());
    let _width = writer_uniform_width(&event.control_event).unwrap();

    let mut sentinel = (*synthetic_prepared_snapshot()).clone();
    let event = find_object_event_mut(&mut sentinel.object_events, &ordinal_zero_uid).unwrap();
    event.control_event = SelectedObjectEvent::new(
        event.physical_host,
        0,
        1_537,
        1_537,
        Some(-1_000.0),
        Some(-1_000.0),
        Some(-1.0),
        Some(0.1),
        None,
        None,
        Some(0),
    );
    let writer_xyz = writer_coordinates(&event.control_event).unwrap();
    assert_eq!(
        writer_xyz,
        [0.0_f32.to_bits(), 1.0_f32.to_bits(), (-1.0_f32).to_bits()]
    );
    let _width = writer_uniform_width(&event.control_event).unwrap();
}

#[test]
fn late_preflight_failures_return_no_proposal_and_leave_before_values_unchanged() {
    let prepared = synthetic_prepared_snapshot();
    let before = cold_before_state(&prepared);
    let before_copy = before.clone();

    let mut duplicate_host = (*prepared).clone();
    duplicate_host.physical_tracks[37].physical_host = 0;
    let duplicate_host = Arc::new(duplicate_host);
    assert_eq!(
        propose_normal_control_snapshot(&before, &duplicate_host, transaction_id(&prepared, 0))
            .unwrap_err(),
        NormalControlSnapshotError::PhysicalIdentityMismatch
    );

    let mut short_last_event = (*prepared).clone();
    short_last_event.object_events[27].object_duration = 1;
    let short_last_event = Arc::new(short_last_event);
    assert_eq!(
        propose_normal_control_snapshot(&before, &short_last_event, transaction_id(&prepared, 0))
            .unwrap_err(),
        NormalControlSnapshotError::ObjectEventMismatch
    );
    assert_eq!(before, before_copy);

    let first =
        propose_normal_control_snapshot(&before, &prepared, transaction_id(&prepared, 0)).unwrap();
    let before_one = before_from_proposal(&first);
    assert_eq!(
        propose_normal_control_snapshot(&before_one, &prepared, transaction_id(&prepared, 0))
            .unwrap_err(),
        NormalControlSnapshotError::CallbackClockMismatch
    );

    let mut changed_prepared = (*prepared).clone();
    changed_prepared.source_identity.job_projection_sha256[0] ^= 1;
    let changed_prepared = Arc::new(changed_prepared);
    assert_eq!(
        propose_normal_control_snapshot(
            &before_one,
            &changed_prepared,
            transaction_id(&prepared, 1)
        )
        .unwrap_err(),
        NormalControlSnapshotError::PreparedIdentityMismatch
    );

    let mut wrong_state_owner = before_one.clone();
    wrong_state_owner.objects[0].host_channel_index = Some(0);
    assert_eq!(
        propose_normal_control_snapshot(
            &wrong_state_owner,
            &prepared,
            transaction_id(&prepared, 1)
        )
        .unwrap_err(),
        NormalControlSnapshotError::StateOwnerMismatch
    );

    let mut foreign_programme = cold_before_state(&prepared);
    foreign_programme.programme_id = ProgrammeId::new("foreign-programme").unwrap();
    assert_eq!(
        propose_normal_control_snapshot(
            &foreign_programme,
            &prepared,
            CallbackLineageIdentity::new(
                foreign_programme.programme_id.clone(),
                foreign_programme.attempt_id,
                FrontendCallbackIndex::new(0),
            )
        )
        .unwrap_err(),
        NormalControlSnapshotError::CallbackIdentityMismatch
    );
}

#[test]
fn discarding_proposal_or_mutating_a_carrier_copy_does_not_change_the_proposal() {
    let prepared = synthetic_prepared_snapshot();
    let before = cold_before_state(&prepared);
    let before_copy = before.clone();
    let proposal =
        propose_normal_control_snapshot(&before, &prepared, transaction_id(&prepared, 0)).unwrap();
    let first_bed = proposal.control38_rows()[0].clone();
    let mut copied_carrier = proposal.control38_carrier_words().to_vec();
    copied_carrier[0] = u32::MAX;
    assert_eq!(proposal.control38_rows()[0], first_bed);
    assert_ne!(proposal.control38_carrier_words()[0], u32::MAX);
    drop(proposal);
    assert_eq!(before, before_copy);
}

fn synthetic_prepared_snapshot() -> Arc<NormalControlPreparedSnapshot> {
    let snapshot: serde_json::Value = serde_json::from_str(SNAPSHOT_INPUTS).unwrap();
    let owner_inputs: serde_json::Value = serde_json::from_str(OWNER_INPUTS).unwrap();
    let selected_profile = resolve_selected_profile(&EncodingProfile::selected()).unwrap();
    let metadata_hashes = PrimaryMetadataChunkHashes::new([1; 32], [2; 32], [3; 32]);
    let source_identity = NormalControlSourceIdentity {
        selected_profile,
        input_kind: SelectedInputKind::Adm,
        source_file_name: "self-owned-input.wav".to_owned(),
        source_file_bytes: 0,
        data_bytes: 0,
        wave: SelectedWaveDescriptor::new(48_000, 38, 24),
        source_samples: 1_537,
        metadata_hashes,
        job_projection_sha256: [4; 32],
        source_receipt_sha256: [5; 32],
        original_job_sha256_declaration: [6; 32],
        original_job_bytes_declaration: 0,
        original_job_bytes_available: false,
        full_pcm_identity_rehashed: false,
        runtime_resource_bound: false,
    };
    let profile = NormalControlProfileIdentity {
        sample_rate_hz: 48_000,
        channels: 38,
        pcm_bits: 24,
        source_samples: 1_537,
        bit_rate_kbps: 1_024,
        bit_rate_enum: 5,
        layout_enum: 8,
        target_capacity: 15,
        frame_samples: 1_536,
        spatial_config_mode: 0,
        dispatch_mode: 2,
        virtual_prefix_count: 11,
        bed_count: 10,
        object_count: 28,
        event_start_samples: 0,
        line_drc: "off".to_owned(),
        rf_drc: "off".to_owned(),
        time_base: "media".to_owned(),
    };

    let bed_rows = snapshot["prepared"]["beds"].as_array().unwrap();
    let bed_by_format = bed_rows
        .iter()
        .map(|bed| {
            (
                bed["channel_format_id"].as_str().unwrap().to_owned(),
                (
                    u8::try_from(bed["host_channel_index"].as_u64().unwrap()).unwrap(),
                    u8::try_from(bed["bed_id"].as_u64().unwrap()).unwrap(),
                    bed["semantic_role"].as_str().unwrap().to_owned(),
                ),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let pack_ids = snapshot["prepared"]["pack_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|format| format.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let pack_ids: [String; BED_TRACK_COUNT] = pack_ids.try_into().unwrap();
    let bed_pack_hosts: [u8; BED_TRACK_COUNT] = pack_ids
        .iter()
        .map(|format| bed_by_format[format].0)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let lfe_host = bed_by_format
        .values()
        .find(|(_, _, role)| role == "lfe")
        .map(|(host, _, _)| *host)
        .unwrap();

    let mut owner_rows = owner_inputs["base_object_tracks"]
        .as_array()
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    owner_rows.sort_by(|left, right| {
        left["audio_track_uid"]
            .as_str()
            .unwrap()
            .as_bytes()
            .cmp(right["audio_track_uid"].as_str().unwrap().as_bytes())
    });
    let object_rows = snapshot["prepared"]["objects"].as_array().unwrap();
    assert_eq!(owner_rows.len(), OBJECT_TRACK_COUNT);
    assert_eq!(object_rows.len(), OBJECT_TRACK_COUNT);
    let mut owner_bindings = Vec::with_capacity(OBJECT_TRACK_COUNT);
    let mut object_events = Vec::with_capacity(OBJECT_TRACK_COUNT);
    let mut physical_tracks = Vec::with_capacity(PHYSICAL_TRACK_COUNT);
    for (ordinal, owner) in owner_rows.iter().enumerate() {
        let recipe = &object_rows[ordinal];
        assert_eq!(recipe["object_ordinal"].as_u64(), Some(ordinal as u64));
        let track_uid = owner["audio_track_uid"].as_str().unwrap().to_owned();
        let semantic_key = u32::try_from(owner["semantic_key"].as_u64().unwrap()).unwrap();
        let channel_format_id = owner["channel_format_id"].as_str().unwrap().to_owned();
        let object_id = owner["audio_objects"][0]["id"].as_str().unwrap().to_owned();
        let host = u8::try_from(recipe["host_channel_index"].as_u64().unwrap()).unwrap();
        let local_owner_ordinal = u8::try_from(ordinal).unwrap();
        let dbmd = synthetic_dbmd(host);
        owner_bindings.push(NormalControlObjectOwnerBinding {
            track_uid: track_uid.clone(),
            local_owner_ordinal,
            semantic_key,
            physical_host: host,
            channel_format_id: channel_format_id.clone(),
        });
        physical_tracks.push(NormalControlPhysicalTrack {
            track_uid: track_uid.clone(),
            object_id: object_id.clone(),
            physical_host: host,
            channel_format_id,
            kind: NormalControlTrackKind::Object {
                semantic_key,
                local_owner_ordinal,
            },
            dbmd,
        });
        let input = &recipe["event"];
        let duration = input["duration"].as_u64().unwrap();
        let event = SelectedObjectEvent::new(
            host,
            input["start"].as_u64().unwrap(),
            duration,
            1_537,
            input.get("x").and_then(serde_json::Value::as_f64),
            input.get("y").and_then(serde_json::Value::as_f64),
            input.get("z").and_then(serde_json::Value::as_f64),
            input.get("width").and_then(serde_json::Value::as_f64),
            input.get("depth").and_then(serde_json::Value::as_f64),
            input.get("height").and_then(serde_json::Value::as_f64),
            input["diffuse"]
                .as_i64()
                .map(|value| i32::try_from(value).unwrap()),
        );
        object_events.push(NormalControlObjectEvent {
            track_uid,
            object_id,
            physical_host: host,
            object_start: input["start"].as_u64().unwrap(),
            block_rtime: input["rtime"].as_u64().unwrap(),
            interpolation_length: input["interpolation"].as_u64().unwrap(),
            object_duration: duration,
            block_duration: input["block_duration"].as_u64().unwrap(),
            control_event: event,
        });
    }
    for (channel_format_id, (host, bed_id, semantic_role)) in &bed_by_format {
        physical_tracks.push(NormalControlPhysicalTrack {
            track_uid: format!("BED_{channel_format_id}"),
            object_id: format!("BED_OBJECT_{bed_id}"),
            physical_host: *host,
            channel_format_id: channel_format_id.clone(),
            kind: NormalControlTrackKind::Bed {
                bed_id: *bed_id,
                bed_pack_ordinal: *bed_id,
                semantic_role: semantic_role.clone(),
                bed_input_id: u32::from(*bed_id),
            },
            dbmd: synthetic_dbmd(*host),
        });
    }
    physical_tracks.sort_by_key(|track| track.physical_host);
    let physical_tracks = physical_tracks.try_into().unwrap();
    let owner_bindings = owner_bindings.try_into().unwrap();
    let object_events = object_events.try_into().unwrap();
    let regular37_hosts: [u8; REGULAR_TRACK_COUNT] = (0_u8..38)
        .filter(|host| *host != lfe_host)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let regular_source_bindings = regular37_hosts
        .iter()
        .enumerate()
        .map(|(rank, host)| NormalControlRegularSourceBinding {
            physical_host: *host,
            global_source: 11 + u64::try_from(rank).unwrap(),
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    Arc::new(NormalControlPreparedSnapshot {
        programme_id: ProgrammeId::new("own-programme").unwrap(),
        attempt_id: AttemptId::new(0),
        profile,
        source_identity,
        graph_identity: NormalControlGraphIdentity {
            axml_sha256: [1; 32],
            chna_sha256: [2; 32],
            physical_track_count: 38,
            bed_track_count: 10,
            object_track_count: 28,
            direct_lfe_host: lfe_host,
            virtual_prefix_source_count: 11,
            total_source_count: 48,
        },
        frontend_state_digest: None,
        dbmd_payload_sha256: [3; 32],
        physical_tracks,
        owner_bindings,
        object_events,
        bed_pack_channel_format_ids: pack_ids,
        bed_pack_hosts,
        regular_source_bindings,
        lfe_host,
        regular37_hosts,
    })
}

fn synthetic_dbmd(host: u8) -> NormalControlDbmdFields {
    NormalControlDbmdFields {
        track_index: u16::from(host),
        enum48: i8::try_from(host).unwrap(),
        enum52: -1,
        flag64: 1,
        raw_flag_low2: 2,
        raw_packed: host,
    }
}

fn cold_before_state(prepared: &NormalControlPreparedSnapshot) -> NormalControlBeforeState {
    NormalControlBeforeState {
        programme_id: prepared.programme_id.clone(),
        attempt_id: prepared.attempt_id,
        callback_index: FrontendCallbackIndex::new(0),
        normal_callbacks: NormalCallbackCount::new(0),
        owner_generation: 0,
        remaining_cached_drains: 4,
        phase: FrontendControlPhase::BeforeFirstNormal,
        profile: prepared.profile.clone(),
        state_digest: None,
        objects: std::array::from_fn(|ordinal| NormalControlObjectState {
            object_ordinal: u32::try_from(ordinal).unwrap(),
            event_cache: None,
            host_channel_index: None,
            sampler_previous_present: false,
            sampler_previous_xyz_u32: [0; 3],
        }),
        last_control38_snapshot: None,
        last_regular37_snapshot: None,
        physical_binding: None,
        dbmd_binding: None,
        previous_prepared_identity: None,
    }
}

fn transaction_id(
    prepared: &NormalControlPreparedSnapshot,
    callback_index: u32,
) -> CallbackLineageIdentity {
    CallbackLineageIdentity::new(
        prepared.programme_id.clone(),
        prepared.attempt_id,
        FrontendCallbackIndex::new(callback_index),
    )
}

fn before_from_proposal(proposal: &NormalControlSnapshotProposal) -> NormalControlBeforeState {
    let after = proposal.after_state();
    NormalControlBeforeState {
        programme_id: after.programme_id.clone(),
        attempt_id: after.attempt_id,
        callback_index: after.next_callback_index,
        normal_callbacks: after.next_normal_callbacks,
        owner_generation: after.next_owner_generation,
        remaining_cached_drains: after.remaining_cached_drains,
        phase: after.phase,
        profile: proposal.after_state.prepared_identity.profile.clone(),
        state_digest: None,
        objects: after.objects.clone(),
        last_control38_snapshot: Some(Arc::clone(&after.control38_snapshot)),
        last_regular37_snapshot: Some(Arc::clone(&after.regular37_snapshot)),
        physical_binding: after.physical_binding_before.clone(),
        dbmd_binding: after.dbmd_binding_before.clone(),
        previous_prepared_identity: Some(Arc::clone(&after.prepared_identity)),
    }
}

fn find_object_event_mut<'a>(
    events: &'a mut [NormalControlObjectEvent; OBJECT_TRACK_COUNT],
    uid: &str,
) -> Option<&'a mut NormalControlObjectEvent> {
    events.iter_mut().find(|event| event.track_uid == uid)
}
