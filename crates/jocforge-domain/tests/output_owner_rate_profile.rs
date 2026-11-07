use jocforge_domain::{
    codec::frame_size::Eac3FrameSize,
    integrity::crc16::eac3_crc16,
    profile::CodecRateProfile,
    publication::{
        OutputConfiguration, OutputOwner, OutputProvider, OutputSourceIdentity,
        OutputSourceProposal,
    },
    timeline::{CodecSourceIndex, JocDispatchIndex},
};
use sha2::{Digest, Sha256};

fn frame_for_rate(codec_rate: CodecRateProfile) -> Vec<u8> {
    let size = Eac3FrameSize::from_codec_rate(codec_rate).unwrap();
    let mut frame = vec![0_u8; size.bytes()];
    frame[0..2].copy_from_slice(&[0x0b, 0x77]);
    for offset in 0..11 {
        let value = (size.code().value() >> (10 - offset)) & 1;
        if value != 0 {
            let bit = 21 + offset;
            frame[bit / 8] |= 1 << (7 - (bit % 8));
        }
    }
    let crc_offset = frame.len() - 2;
    let crc = eac3_crc16(&frame[2..crc_offset]);
    frame[crc_offset..].copy_from_slice(&crc.to_be_bytes());
    assert_eq!(eac3_crc16(&frame[2..]), 0);
    frame
}

fn stage_suppressed_source(owner: OutputOwner, source: u64, frame: &[u8]) -> OutputOwner {
    let identity = OutputSourceIdentity::new(
        CodecSourceIndex::new(source),
        JocDispatchIndex::new(source + 1),
        None,
    );
    match owner
        .prepare_source(identity, frame, &Sha256::digest(frame).into())
        .unwrap()
    {
        OutputSourceProposal::Suppressed(proposal) => proposal.into_after().into_after_owner(),
        OutputSourceProposal::Append(_) => panic!("the two leading sources are suppressed"),
    }
}

#[test]
fn output_owner_keeps_each_selected_rate_frame_extent_through_append_and_finalize() {
    let rows = [
        (CodecRateProfile::Kbps384, 1_536, 50_688),
        (CodecRateProfile::Kbps448, 1_792, 59_136),
        (CodecRateProfile::Kbps576, 2_304, 76_032),
        (CodecRateProfile::Kbps640, 2_560, 84_480),
        (CodecRateProfile::Kbps768, 3_072, 101_376),
        (CodecRateProfile::Kbps1024, 4_096, 135_168),
    ];

    for (codec_rate, frame_bytes, expected_product_bytes) in rows {
        let frame = frame_for_rate(codec_rate);
        let configuration = OutputConfiguration::new(
            OutputProvider {
                profile_id: codec_rate.identity_label().to_owned(),
                provider_sha256: Sha256::digest(codec_rate.identity_label().as_bytes()).into(),
                frame_bytes: u32::try_from(frame_bytes).unwrap(),
                suppressed_sources: 2,
                staging_name_template: "attempt-{attempt_id}.partial".to_owned(),
                intent_name_template: "attempt-{attempt_id}.intent".to_owned(),
            },
            "rate-test",
            "selected.ec3",
            1_536 * 33,
        )
        .unwrap();
        assert_eq!(
            configuration.expected_product_bytes(),
            expected_product_bytes
        );
        let owner = OutputOwner::cold(configuration);
        let owner = stage_suppressed_source(owner, 0, &frame);
        let owner = stage_suppressed_source(owner, 1, &frame);
        let identity = OutputSourceIdentity::new(
            CodecSourceIndex::new(2),
            JocDispatchIndex::new(3),
            Some(jocforge_domain::timeline::PublishedAuIndex::new(0)),
        );
        let proposal = owner
            .prepare_source(identity, &frame, &Sha256::digest(&frame).into())
            .unwrap();
        let OutputSourceProposal::Append(append) = proposal else {
            panic!("the third source publishes AU zero");
        };
        assert_eq!(append.frame_bytes().len(), frame_bytes);
        assert_eq!(append.expected_offset(), 0);
        assert_eq!(append.as_json()["frame_bytes"], frame_bytes as u64);
    }
}
