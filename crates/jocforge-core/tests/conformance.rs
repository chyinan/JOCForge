// pattern: Functional Core

use jocforge_core::{
    BlockPartition, ConformancePlanError, FfmpegInteroperabilityStatus, JocProfile,
    canonical_dependent_channel_maps, channel_map_channel_count, resolve_dependent_topology,
    resolve_short_block_plan,
};

#[test]
fn short_block_plan_accumulates_public_partitions_to_one_joc_interval() {
    let plan = resolve_short_block_plan(&[
        BlockPartition::One,
        BlockPartition::Two,
        BlockPartition::Three,
    ])
    .expect("1+2+3 short-block plan");

    assert_eq!(plan.audio_blocks(), 6);
    assert_eq!(plan.samples(), 1536);
    assert_eq!(plan.numblkscod(), &[0, 1, 2]);
    assert_eq!(plan.convsync(), &[true, false, false]);
}

#[test]
fn short_block_plan_rejects_incomplete_or_overfull_programme_groups() {
    assert_eq!(
        resolve_short_block_plan(&[BlockPartition::Three]),
        Err(ConformancePlanError::TotalBlocks { actual: 3 })
    );
    assert_eq!(
        resolve_short_block_plan(&[BlockPartition::Six, BlockPartition::One]),
        Err(ConformancePlanError::TotalBlocks { actual: 7 })
    );
}

#[test]
fn short_block_plan_covers_the_canonical_public_partition_families() {
    let plans = [
        vec![BlockPartition::Six],
        vec![BlockPartition::Three, BlockPartition::Three],
        vec![
            BlockPartition::Two,
            BlockPartition::Two,
            BlockPartition::Two,
        ],
        vec![BlockPartition::One; 6],
    ];
    for partitions in plans {
        let plan = resolve_short_block_plan(&partitions).expect("canonical short-block family");
        assert_eq!(plan.samples(), 1536);
        assert_eq!(plan.audio_blocks(), 6);
    }
}

#[test]
fn dependent_topology_is_sequential_and_bounded_by_public_substream_ids() {
    let topology = resolve_dependent_topology(8).expect("D0..D7");
    assert_eq!(topology.ids(), &[0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(
        resolve_dependent_topology(9),
        Err(ConformancePlanError::DependentCount { actual: 9 })
    );
}

#[test]
fn canonical_multi_dependent_maps_close_profile_ownership_without_overlap() {
    assert_eq!(
        canonical_dependent_channel_maps(JocProfile::Idx0, 2).expect("5.X D0+D1"),
        vec![0xc000, 0x3000]
    );
    assert_eq!(
        canonical_dependent_channel_maps(JocProfile::Idx1, 6).expect("Flat-7.X six-dependent plan"),
        vec![0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0200]
    );
    assert!(canonical_dependent_channel_maps(JocProfile::Idx0, 6).is_err());
    assert_eq!(channel_map_channel_count(0x0200), 2);
}

#[test]
fn ffmpeg_interoperability_status_has_explicit_external_only_names() {
    assert_eq!(
        FfmpegInteroperabilityStatus::UnsupportedDecoderFeature.to_string(),
        "UNSUPPORTED_DECODER_FEATURE"
    );
    assert_eq!(
        FfmpegInteroperabilityStatus::FailStreamError.to_string(),
        "FAIL_STREAM_ERROR"
    );
}
