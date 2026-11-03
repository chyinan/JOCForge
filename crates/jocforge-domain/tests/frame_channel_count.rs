// pattern: Functional Core
use jocforge_domain::frontend::frame_channel_count::{
    FrameChannelCountError, project_frame_channel_count_candidates, static_oamdi_source_count,
};

#[test]
fn static_oamdi_source_count_observes_the_reported_guard_without_claiming_runtime_acceptance() {
    assert_eq!(static_oamdi_source_count(36).unwrap().value(), 36);
    assert_eq!(static_oamdi_source_count(38).unwrap().value(), 38);
    assert_eq!(static_oamdi_source_count(159).unwrap().value(), 159);
    assert_eq!(
        static_oamdi_source_count(0),
        Err(FrameChannelCountError::StaticOamdiCountOutOfRange)
    );
    assert_eq!(
        static_oamdi_source_count(160),
        Err(FrameChannelCountError::StaticOamdiCountOutOfRange)
    );
}

#[test]
fn ordinary_and_extended_reference_vectors_preserve_36_and_38_frame_rows() {
    for (physical_rows, object_rows) in [(36_u16, 26_u16), (38, 28)] {
        let candidates =
            project_frame_channel_count_candidates(physical_rows, physical_rows, &[object_rows])
                .unwrap();
        assert_eq!(candidates.ordinary_mode().value(), physical_rows);
        assert_eq!(candidates.extended_mode().value(), physical_rows);
    }
}

#[test]
fn extended_reference_uses_the_maximum_object_tagged_count_across_adapters() {
    let candidates = project_frame_channel_count_candidates(36, 36, &[20, 26, 23]).unwrap();
    assert_eq!(candidates.extended_mode().value(), 36);
}

#[test]
fn reference_projection_rejects_counts_outside_its_vectors() {
    assert_eq!(
        project_frame_channel_count_candidates(36, 38, &[26]),
        Err(FrameChannelCountError::ReferenceCountMismatch)
    );
    assert_eq!(
        project_frame_channel_count_candidates(38, 38, &[26]),
        Err(FrameChannelCountError::ReferenceCountMismatch)
    );
}
