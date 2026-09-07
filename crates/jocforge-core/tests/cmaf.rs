// pattern: Functional Core

use jocforge_core::{EmdfProfile, JocProfile, cmaf_joc_init_and_fragments, write_one_au_proof};

#[test]
fn cmaf_writer_emits_ec3_dec3_and_one_complete_joc_sample() {
    let sample = write_one_au_proof(JocProfile::Idx1).expect("idx1 sample");
    let output =
        cmaf_joc_init_and_fragments(JocProfile::Idx1, EmdfProfile::EtsiStrict, 1664, &[sample])
            .expect("CMAF JOC output");

    assert!(output.windows(4).any(|window| window == b"ec-3"));
    assert!(output.windows(4).any(|window| window == b"dec3"));
    assert!(output.windows(4).any(|window| window == b"moof"));
    assert!(output.windows(4).any(|window| window == b"mdat"));
}

#[test]
fn cmaf_writer_rejects_non_canonical_sample_shape() {
    let error = cmaf_joc_init_and_fragments(
        JocProfile::Idx1,
        EmdfProfile::EtsiStrict,
        1664,
        &[vec![0x0b, 0x77]],
    )
    .expect_err("truncated CMAF sample");
    assert!(error.contains("1536-sample JOC access unit"));
}
